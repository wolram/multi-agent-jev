mod common;

use common::{init_git, scratch, MockRunner};
use jev_core::client::JevClient;
use jev_core::transport::MockTransport;
use orchestrator::orchestrator::{Orchestrator, RunOutcome};
use orchestrator::queue::QueuedTask;
use orchestrator::state::TaskState;
use orchestrator::worktree::WorktreeManager;
use serde_json::json;

#[test]
fn flow_basic() {
    let tmp = scratch("flow_basic");
    init_git(&tmp);
    let wts = tmp.join("wts");
    std::fs::create_dir_all(&wts).unwrap();
    let wt = WorktreeManager::new(&tmp, &wts);

    let t = MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.9}))
        .with_response("risk", json!({"critical_path": false, "reasons": [], "confidence": 0.9}))
        .with_response("escalate", json!({"action": "local", "confidence": 0.9}))
        .with_response(
            "brief",
            json!({
                "objective": "x",
                "constraints": [],
                "inputs": {},
                "done_when": ["done"]
            }),
        );
    let c = JevClient::new(t);
    let mut orch = Orchestrator::new(c, MockRunner::new(), wt);
    orch.enqueue(QueuedTask {
        id: "t1".into(),
        title: "impl".into(),
        body: "".into(),
        paths: vec![],
    });

    let out = orch.run_once().unwrap();
    assert_eq!(
        out,
        RunOutcome::ReadyToMerge {
            id: "t1".into()
        }
    );
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::ReadyToMerge);
}
