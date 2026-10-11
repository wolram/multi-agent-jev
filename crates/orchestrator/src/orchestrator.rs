use crate::error::{Error, Result};
use crate::queue::{Queue, QueuedTask};
use crate::state::TaskState;
use crate::worktree::WorktreeManager;
use jev_core::client::JevClient;
use jev_core::schema::Task;
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

pub struct Orchestrator<R: Router> {
    pub router: R,
    pub queue: Queue,
    pub worktrees: WorktreeManager,
    pub states: HashMap<String, TaskState>,
    pub current_worktree: HashMap<String, PathBuf>,
}

impl<R: Router> Orchestrator<R> {
    pub fn new(router: R, worktrees: WorktreeManager) -> Self {
        Self {
            router,
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

    pub fn run_once(&mut self) -> Result<bool> {
        let Some(qt) = self.queue.claim() else { return Ok(false); };
        let task = Task {
            id: qt.id.clone(),
            title: qt.title,
            body: qt.body,
            paths: qt.paths,
        };
        let _plan = self
            .router
            .plan(&task)
            .map_err(|e| Error::Jev(e.to_string()))?;
        self.transition(&qt.id, TaskState::Routed)?;
        let _ = self.worktrees.create(&format!("jev-{}", qt.id), None);
        self.transition(&qt.id, TaskState::Designed)?;
        self.transition(&qt.id, TaskState::Implemented)?;
        self.transition(&qt.id, TaskState::Reviewed)?;
        self.transition(&qt.id, TaskState::ReadyToMerge)?;
        Ok(true)
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
