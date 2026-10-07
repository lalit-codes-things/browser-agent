// Task Authority.
//
// Task authority defines what the task is allowed to do and under which
// constraints. It is backend-derived and immutable for the task lifetime
// unless revised through the explicit revise path.
//
// No page-derived content can redefine task authority (C-144).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAuthority {
    pub task_id: String,
    pub bounded_origins: Vec<AuthorityOrigin>,
    pub allowed_actions: Vec<AllowedAction>,
    pub constraints: TaskConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityOrigin {
    pub origin: String,
    pub scope: OriginScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OriginScope {
    ExactOrigin,
    MultiSubdomain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedAction {
    pub action: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConstraints {
    pub max_steps: Option<u32>,
    pub max_llm_calls: Option<u32>,
    pub max_task_duration_ms: Option<u64>,
    pub max_confirmations: Option<u32>,
    pub allowed_network_destination_classes: Vec<String>,
    pub high_stakes_threshold_amount: Option<String>,
}
