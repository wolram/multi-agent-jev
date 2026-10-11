use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Agent {
    Maestro,
    Codesmith,
    Codex,
    Researcher,
    Designer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDecision {
    pub agent: Agent,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDecision {
    pub critical_path: bool,
    pub reasons: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalateAction {
    Local,
    FrontierLlm,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalateDecision {
    pub action: EscalateAction,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brief {
    pub objective: String,
    pub constraints: Vec<String>,
    pub inputs: serde_json::Value,
    pub done_when: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub body: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TaskPlan {
    pub task_id: String,
    pub route: RouteDecision,
    pub risk: RiskDecision,
    pub escalate: EscalateDecision,
    pub brief: Brief,
    pub needs_review: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum BandResult {
    Follow,
    NeedsReview,
    Escalate,
}

fn validate_conf(conf: f64) -> Result<(), crate::error::Error> {
    if conf.is_nan() || conf < 0.0 || conf > 1.0 {
        return Err(crate::error::Error::Validation(format!(
            "confidence must be in [0,1], got {conf}"
        )));
    }
    Ok(())
}
