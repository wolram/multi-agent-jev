//! Shared test helpers for the orchestrator integration tests.
#![allow(dead_code)]

use agents::contracts::{DesignDoc, ImplementationReport, ReviewReport};
use jev_core::schema::{Task, TaskPlan};
use orchestrator::error::Result;
use orchestrator::runner::AgentRunner;
use std::path::PathBuf;
use std::process::Command;

/// Deterministic, per-test scratch directory.
pub fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("jev-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Initializes a real git repo so `git worktree add` succeeds.
pub fn init_git(dir: &std::path::Path) {
    let git = |args: &[&str]| {
        Command::new("git").current_dir(dir).args(args).status().unwrap();
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.name", "test"]);
    git(&["config", "user.email", "test@test"]);
    std::fs::write(dir.join("README.md"), "test").unwrap();
    git(&["add", "."]);
    git(&["commit", "-m", "init"]);
}

/// Stub agent runner. Defaults to approving design and review.
pub struct MockRunner {
    pub design_approved: bool,
    pub review_approve: bool,
}

impl MockRunner {
    pub fn new() -> Self {
        Self {
            design_approved: true,
            review_approve: true,
        }
    }

    pub fn design_approved(mut self, v: bool) -> Self {
        self.design_approved = v;
        self
    }

    pub fn review_approve(mut self, v: bool) -> Self {
        self.review_approve = v;
        self
    }
}

impl Default for MockRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRunner for MockRunner {
    fn design(&self, task: &Task, _plan: &TaskPlan) -> Result<DesignDoc> {
        Ok(DesignDoc {
            feature: task.id.clone(),
            md_path: format!("docs/designs/{}.md", task.id),
            approved: self.design_approved,
        })
    }

    fn implement(&self, _task: &Task, _design: &DesignDoc) -> Result<ImplementationReport> {
        Ok(ImplementationReport {
            changed_files: vec![],
            pr_url: None,
            tests_passed: true,
        })
    }

    fn review(&self, _task: &Task, _report: &ImplementationReport) -> Result<ReviewReport> {
        Ok(ReviewReport {
            approve: self.review_approve,
            findings: vec![],
        })
    }
}
