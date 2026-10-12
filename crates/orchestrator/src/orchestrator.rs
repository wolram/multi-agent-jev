use crate::error::{Error, Result};
use crate::queue::{Queue, QueuedTask};
use crate::runner::AgentRunner;
use crate::state::TaskState;
use crate::worktree::WorktreeManager;
use jev_core::client::JevClient;
use jev_core::schema::{EscalateAction, Task};
use jev_core::transport::Transport;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait Router: Send + Sync {
    fn plan(&self, task: &jev_core::schema::Task) -> jev_core::error::Result<jev_core::schema::TaskPlan>;
}

impl<T: Transport + Send + Sync> Router for JevClient<T> {
    fn plan(&self, task: &jev_core::schema::Task) -> jev_core::error::Result<jev_core::schema::TaskPlan> {
        JevClient::plan(self, task)
    }
}

/// Result of a single orchestration step. `Merged` is intentionally absent:
/// only the human (L4) advances a task to `Merged`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Idle,
    Escalated { id: String, action: EscalateAction },
    DesignRejected { id: String },
    ReviewRejected { id: String },
    ReadyToMerge { id: String },
}

pub struct Orchestrator<R: Router, A: AgentRunner> {
    pub router: R,
    pub runner: A,
    pub queue: Queue,
    pub worktrees: WorktreeManager,
    pub states: HashMap<String, TaskState>,
    pub current_worktree: HashMap<String, PathBuf>,
}

impl<R: Router, A: AgentRunner> Orchestrator<R, A> {
    pub fn new(router: R, runner: A, worktrees: WorktreeManager) -> Self {
        Self {
            router,
            runner,
            queue: Queue::new(),
            worktrees,
            states: HashMap::new(),
            current_worktree: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, t: QueuedTask) {
        self.states.insert(t.id.clone(), TaskState::Open);
        self.queue.push(t);
    }

    /// Advances one task, honoring the L0 `TaskPlan`.
    ///
    /// The plan is binding: the orchestrator never runs a task locally when the
    /// plan says to escalate, never skips the L3 review when `needs_review`, and
    /// never transitions to `Merged` (human is the only merge gate).
    pub fn run_once(&mut self) -> Result<RunOutcome> {
        let Some(qt) = self.queue.claim() else {
            return Ok(RunOutcome::Idle);
        };
        let task = Task {
            id: qt.id.clone(),
            title: qt.title,
            body: qt.body,
            paths: qt.paths,
        };
        let plan = self
            .router
            .plan(&task)
            .map_err(|e| Error::Jev(e.to_string()))?;
        self.transition(&qt.id, TaskState::Routed)?;

        // L0 binding: do not execute locally when the plan says to escalate.
        if plan.escalate.action != EscalateAction::Local {
            self.transition(&qt.id, TaskState::Blocked)?;
            return Ok(RunOutcome::Escalated {
                id: qt.id,
                action: plan.escalate.action,
            });
        }

        let worktree = self
            .worktrees
            .create(&format!("jev-{}", qt.id), None)?;
        self.current_worktree.insert(qt.id.clone(), worktree);

        // L1: design before code.
        let design = self.runner.design(&task, &plan)?;
        if !design.approved {
            self.transition(&qt.id, TaskState::Blocked)?;
            return Ok(RunOutcome::DesignRejected { id: qt.id });
        }
        self.transition(&qt.id, TaskState::Designed)?;

        // L2: implementation in the isolated worktree.
        let implementation = self.runner.implement(&task, &design)?;
        self.transition(&qt.id, TaskState::Implemented)?;

        // L3: adversarial review is mandatory on `needs_review` (critical path
        // or low confidence).
        if plan.needs_review {
            let review = self.runner.review(&task, &implementation)?;
            if !review.approve {
                self.transition(&qt.id, TaskState::Blocked)?;
                return Ok(RunOutcome::ReviewRejected { id: qt.id });
            }
        }

        self.transition(&qt.id, TaskState::Reviewed)?;
        self.transition(&qt.id, TaskState::ReadyToMerge)?;
        Ok(RunOutcome::ReadyToMerge { id: qt.id })
    }

    fn transition(&mut self, id: &str, to: TaskState) -> Result<TaskState> {
        let cur = *self
            .states
            .get(id)
            .ok_or_else(|| Error::InvalidTransition(format!("unknown task {id}")))?;
        if !cur.can_transition(to) {
            return Err(Error::InvalidTransition(format!(
                "cannot transition {} -> {}",
                cur.as_str(),
                to.as_str()
            )));
        }
        self.states.insert(id.to_string(), to);
        Ok(to)
    }
}
