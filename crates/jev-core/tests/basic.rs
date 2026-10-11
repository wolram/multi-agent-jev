use jev_core::schema::{Agent, EscalateAction};
use serde_json::json;

#[test]
fn route_risk_escalate_brief() {
    let t = jev_core::transport::MockTransport::new()
        .with_response("route", json!({"agent": "codesmith", "confidence": 0.9}))
        .with_response("risk", json!({"critical_path": false, "reasons": [], "confidence": 0.9}))
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
        paths: vec![],
    };
    let plan = c.plan(&task).unwrap();
    assert_eq!(plan.route.agent, Agent::Codesmith);
    assert_eq!(plan.escalate.action, EscalateAction::Local);
    assert!(!plan.needs_review);
}
