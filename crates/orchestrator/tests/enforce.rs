mod common;

use common::{init_git, scratch, MockRunner};
use jev_core::client::JevClient;
use jev_core::schema::EscalateAction;
use jev_core::transport::MockTransport;
use orchestrator::orchestrator::{Orchestrator, RunOutcome};
use orchestrator::queue::QueuedTask;
use orchestrator::state::TaskState;
use orchestrator::worktree::WorktreeManager;
use serde_json::json;

/// Builds a JevClient whose L0 decisions are fixed for the test.
fn client(escalate: &str, critical: bool) -> JevClient<MockTransport> {
    let t = MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.9}))
        .with_response(
            "risk",
            json!({"critical_path": critical, "reasons": [], "confidence": 0.9}),
        )
        .with_response("escalate", json!({"action": escalate, "confidence": 0.9}))
        .with_response(
            "brief",
            json!({
                "objective": "x",
                "constraints": [],
                "inputs": {},
                "done_when": ["done"]
            }),
        );
    JevClient::new(t)
}

fn orchestrator(
    name: &str,
    client: JevClient<MockTransport>,
    runner: MockRunner,
) -> Orchestrator<JevClient<MockTransport>, MockRunner> {
    let tmp = scratch(name);
    init_git(&tmp);
    let wts = tmp.join("wts");
    std::fs::create_dir_all(&wts).unwrap();
    let wt = WorktreeManager::new(&tmp, &wts);
    let mut orch = Orchestrator::new(client, runner, wt);
    orch.enqueue(QueuedTask {
        id: "t1".into(),
        title: "impl".into(),
        body: "".into(),
        paths: vec![],
    });
    orch
}

#[test]
fn escalate_human_blocks_without_running_locally() {
    let mut orch = orchestrator("esc_human", client("human", false), MockRunner::new());
    let out = orch.run_once().unwrap();
    assert_eq!(
        out,
        RunOutcome::Escalated {
            id: "t1".into(),
            action: EscalateAction::Human
        }
    );
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::Blocked);
    assert!(!orch.current_worktree.contains_key("t1"));
}

#[test]
fn escalate_frontier_llm_blocks() {
    let mut orch = orchestrator("esc_llm", client("frontier_llm", false), MockRunner::new());
    let out = orch.run_once().unwrap();
    assert_eq!(
        out,
        RunOutcome::Escalated {
            id: "t1".into(),
            action: EscalateAction::FrontierLlm
        }
    );
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::Blocked);
}

#[test]
fn unapproved_design_blocks() {
    let mut orch = orchestrator(
        "design_rej",
        client("local", false),
        MockRunner::new().design_approved(false),
    );
    let out = orch.run_once().unwrap();
    assert_eq!(out, RunOutcome::DesignRejected { id: "t1".into() });
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::Blocked);
}

#[test]
fn critical_path_rejected_review_blocks() {
    let mut orch = orchestrator(
        "review_rej",
        client("local", true),
        MockRunner::new().review_approve(false),
    );
    let out = orch.run_once().unwrap();
    assert_eq!(out, RunOutcome::ReviewRejected { id: "t1".into() });
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::Blocked);
}

#[test]
fn critical_path_approved_review_reaches_ready_to_merge() {
    let mut orch = orchestrator(
        "review_ok",
        client("local", true),
        MockRunner::new().review_approve(true),
    );
    let out = orch.run_once().unwrap();
    assert_eq!(out, RunOutcome::ReadyToMerge { id: "t1".into() });
    assert_eq!(*orch.states.get("t1").unwrap(), TaskState::ReadyToMerge);
}

#[test]
fn orchestrator_never_merges() {
    let mut orch = orchestrator(
        "never_merge",
        client("local", false),
        MockRunner::new(),
    );
    let _ = orch.run_once().unwrap();
    assert_ne!(*orch.states.get("t1").unwrap(), TaskState::Merged);
}
