// Typed event union.
//
// Frontend state is a projection of this event stream. Security-relevant
// renderable state must exist in backend state/event definitions (C-18).
// The frontend never invents system truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum AppEvent {
    TaskStateChanged(TaskStateEvent),
    ModelStateChanged(ModelStateEvent),
    PolicyDecision(PolicyDecisionEvent),
    VerificationOutcome(VerificationOutcomeEvent),
    AuthorizationRequired(AuthorizationRequiredEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStateEvent {
    pub task_id: String,
    pub status: TaskStatus,
    pub step_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    Pending,
    Running,
    Verified,
    LikelySuccess,
    Unknown,
    Failed,
    Aborted,
    Parked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStateEvent {
    pub model_id: String,
    pub residency: ModelResidency,
    pub verified_at_load: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelResidency {
    Loaded,
    Parked,
    Unloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecisionEvent {
    pub action_class: String,
    pub tier: String,
    pub verdict: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationOutcomeEvent {
    pub outcome: String,
    pub evidence_count: u32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationRequiredEvent {
    pub tier: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDurableStateEvent {
    pub durable_id: String,
    pub durable_state: String,
    pub payment_commitment_hash: String,
    pub task_id: String,
}
