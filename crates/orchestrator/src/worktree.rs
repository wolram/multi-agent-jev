use crate::error::Error;
use crate::error::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone)]
pub struct WorktreeManager {
    pub repo_root: PathBuf,
    pub worktrees_root: PathBuf,
}

impl WorktreeManager {
    pub fn new(repo_root: impl Into<PathBuf>, worktrees_root: impl Into<PathBuf>) -> Self {
        Self {
            repo_root: repo_root.into(),
            worktrees_root: worktrees_root.into(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.repo_root);
        cmd.args(args);
        let out = cmd.output()?;
        if !out.status.success() {
            return Err(Error::Git(format!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            )));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    pub fn ensure_repo(&self) -> Result<()> {
        std::fs::create_dir_all(&self.worktrees_root)?;
        Ok(())
    }

    pub fn create(&self, name: &str, branch: Option<&str>) -> Result<PathBuf> {
        let path = self.worktrees_root.join(name);
        let mut args = vec!["worktree", "add", "--detach"];
        if let Some(b) = branch {
            args.push(b);
        }
        args.push(path.to_str().expect("utf8"));
        let _ = self.run(&args);
        Ok(path)
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let path = self.worktrees_root.join(name);
        let _ = self.run(&["worktree", "remove", "--force", path.to_str().expect("utf8")]);
        Ok(())
    }
}
