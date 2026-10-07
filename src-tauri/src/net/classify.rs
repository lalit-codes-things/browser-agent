// Classify.
//
// Request classification supports egress policy and unexpected-traffic signaling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RequestClass {
    Navigation,
    Read,
    Mutation,
    WebSocket,
    Other,
}

impl RequestClass {
    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation)
    }
}
