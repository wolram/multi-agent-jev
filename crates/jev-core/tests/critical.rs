use jev_core::schema::{Agent, EscalateAction};
use serde_json::json;

#[test]
fn critical_path_sets_needs_review() {
    let t = jev_core::transport::MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.9}))
        .with_response("risk", json!({
            "critical_path": true,
            "reasons": ["crates/jev-core/**"],
            "confidence": 0.95
        }))
        .with_response("escalate", json!({"action": "local", "confidence": 0.9}))
        .with_response("brief", json!({
            "objective": "x",
            "constraints": [],
            "inputs": {},
            "done_when": ["done"]
        }));
    let c = jev_core::client::JevClient::new(t);
    let task = jev_core::schema::Task {
        id: "t1".into(),
        title: "impl".into(),
        body: "".into(),
        paths: vec!["crates/jev-core/src/lib.rs".into()],
    };
    let plan = c.plan(&task).unwrap();
    assert!(plan.needs_review);
    assert!(plan.risk.critical_path);
}

#[test]
fn low_confidence_marks_needs_review() {
    let t = jev_core::transport::MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.7}))
        .with_response("risk", json!({
            "critical_path": false,
            "reasons": [],
            "confidence": 0.9
        }))
        .with_response("escalate", json!({"action": "local", "confidence": 0.9}))
        .with_response("brief", json!({
            "objective": "x",
            "constraints": [],
            "inputs": {},
            "done_when": ["done"]
        }));
    let c = jev_core::client::JevClient::new(t);
    let task = jev_core::schema::Task {
        id: "t2".into(),
        title: "impl".into(),
        body: "".into(),
        paths: vec![],
    };
    assert!(c.plan(&task).unwrap().needs_review);
}
