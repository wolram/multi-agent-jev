use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecideRequest {
    pub state: serde_json::Value,
    pub questions: serde_json::Value,
}

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("http error: {0}")]
    Http(String),
    #[error("invalid response: {0}")]
    Invalid(String),
}

pub trait Transport {
    fn post_decide(&self, req: &DecideRequest) -> Result<serde_json::Value, TransportError>;
}

#[derive(Clone)]
pub struct MockTransport {
    pub responses: std::collections::BTreeMap<String, serde_json::Value>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            responses: std::collections::BTreeMap::new(),
        }
    }

    pub fn with_response(mut self, key: impl Into<String>, val: serde_json::Value) -> Self {
        self.responses.insert(key.into(), val);
        self
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for MockTransport {
    fn post_decide(&self, _req: &DecideRequest) -> Result<serde_json::Value, TransportError> {
        Ok(serde_json::json!({ "answers": self.responses.clone() }))
    }
}
