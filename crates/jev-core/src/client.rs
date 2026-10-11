use crate::error::{Error, Result};
use crate::schema::*;
use crate::thresholds::Thresholds;
use crate::transport::{DecideRequest, Transport};
use serde_json::Value;

#[derive(Clone)]
pub struct JevClient<T: Transport> {
    pub transport: T,
    pub thresholds: Thresholds,
}

impl<T: Transport> JevClient<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            thresholds: Thresholds::default(),
        }
    }

    pub fn with_thresholds(mut self, t: Thresholds) -> Self {
        self.thresholds = t;
        self
    }

    fn extract<U: serde::de::DeserializeOwned>(&self, v: &Value, key: &str) -> Result<U> {
        let root = v.get("answers").unwrap_or(v);
        let inner = root.get(key).ok_or_else(|| Error::Validation(format!("missing key {key}")))?;
        Ok(serde_json::from_value(inner.clone())?)
    }

    pub fn route(&self, task: &Task) -> Result<RouteDecision> {
        let req = DecideRequest {
            state: serde_json::to_value(task)?,
            questions: serde_json::json!({ "route": {
                "task_id": task.id,
                "title": task.title,
            }}),
        };
        let resp = self.transport.post_decide(&req).map_err(|e| Error::Transport(e.to_string()))?;
        let d: RouteDecision = self.extract(&resp, "route")?;
        if d.confidence.is_nan() || d.confidence < 0.0 || d.confidence > 1.0 {
            return Err(Error::Validation("route.confidence out of range".into()));
        }
        Ok(d)
    }

    pub fn risk(&self, task: &Task) -> Result<RiskDecision> {
        let req = DecideRequest {
            state: serde_json::to_value(task)?,
            questions: serde_json::json!({ "risk": {
                "paths": task.paths,
            }}),
        };
        let resp = self.transport.post_decide(&req).map_err(|e| Error::Transport(e.to_string()))?;
        let d: RiskDecision = self.extract(&resp, "risk")?;
        if d.confidence.is_nan() || d.confidence < 0.0 || d.confidence > 1.0 {
            return Err(Error::Validation("risk.confidence out of range".into()));
        }
        Ok(d)
    }

    pub fn escalate(&self, _task: &Task, _route: &RouteDecision, _risk: &RiskDecision) -> Result<EscalateDecision> {
        let req = DecideRequest {
            state: serde_json::json!({}),
            questions: serde_json::json!({ "escalate": {} }),
        };
        let resp = self.transport.post_decide(&req).map_err(|e| Error::Transport(e.to_string()))?;
        let d: EscalateDecision = self.extract(&resp, "escalate")?;
        if d.confidence.is_nan() || d.confidence < 0.0 || d.confidence > 1.0 {
            return Err(Error::Validation("escalate.confidence out of range".into()));
        }
        Ok(d)
    }

    pub fn brief(&self, task: &Task, route: &RouteDecision) -> Result<Brief> {
        let req = DecideRequest {
            state: serde_json::to_value(task)?,
            questions: serde_json::json!({ "brief": { "agent": format!("{:?}", route.agent) }}),
        };
        let resp = self.transport.post_decide(&req).map_err(|e| Error::Transport(e.to_string()))?;
        let d: Brief = self.extract(&resp, "brief")?;
        Ok(d)
    }

    pub fn plan(&self, task: &Task) -> Result<TaskPlan> {
        let route = self.route(task)?;
        let risk = self.risk(task)?;
        let escalate = self.escalate(task, &route, &risk)?;
        let brief = self.brief(task, &route)?;

        let needs_review = matches!(
            self.thresholds.classify("route", route.confidence),
            BandResult::NeedsReview | BandResult::Escalate
        ) || matches!(
            self.thresholds.classify("risk", risk.confidence),
            BandResult::NeedsReview | BandResult::Escalate
        ) || matches!(
            self.thresholds.classify("escalate", escalate.confidence),
            BandResult::NeedsReview | BandResult::Escalate
        ) || risk.critical_path;
        Ok(TaskPlan {
            task_id: task.id.clone(),
            route,
            risk,
            escalate,
            brief,
            needs_review,
        })
    }
}
