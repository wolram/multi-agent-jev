use jev_core::client::JevClient;
use jev_core::schema::{Agent, EscalateAction};
use jev_core::transport::MockTransport;
use orchestrator::orchestrator::Orchestrator;
use orchestrator::queue::QueuedTask;
use orchestrator::state::TaskState;
use orchestrator::worktree::WorktreeManager;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

fn init_git(dir: &std::path::Path) {
    Command::new("git").current_dir(dir).args(&["init", "-b", "main"]).status().unwrap();
    Command::new("git").current_dir(dir).args(&["config", "user.name", "test"]).status().unwrap();
    Command::new("git").current_dir(dir).args(&["config", "user.email", "test@test"]).status().unwrap();
    std::fs::write(dir.join("README.md"), "test").unwrap();
    Command::new("git").current_dir(dir).args(&["add", "."]).status().unwrap();
    Command::new("git").current_dir(dir).args(&["commit", "-m", "init"]).status().unwrap();
}

#[test]
fn flow_basic() {
    let tmp = std::env::temp_dir().join(format!("jev-test-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).unwrap();
    init_git(&tmp);
    let wts = tmp.join("wts");
    std::fs::create_dir_all(&wts).unwrap();
    let wt = WorktreeManager::new(&tmp, &wts);
    let t = MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.9}))
        .with_response("risk", json!({"critical_path": false, "reasons": [], "confidence": 0.9}))
        .with_response("escalate", json!({"action": "local", "confidence": 0.9}))
        .with_response("brief", json!({
            "objective": "x",
            "constraints": [],
            "inputs": {},
            "done_when": ["done"]
        }));
    let c = JevClient::new(t);
    let mut orch = Orchestrator::new(c, wt);
    orch.enqueue(QueuedTask {
        id: "t1".into(),
        title: "impl".into(),
        body: "".into(),
        paths: vec![],
    });
    let _ = orch.run_once().unwrap();
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::ReadyToMerge);
}
