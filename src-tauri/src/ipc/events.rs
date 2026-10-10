// Typed event union.
//
// Frontend state is a projection of this event stream. Security-relevant
// renderable state must exist in backend state/event definitions (C-18).
// The frontend never invents system truth.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppEvent {
    #[serde(rename = "TASK_STATE")]
    TaskStateChanged(TaskStateEvent),
    #[serde(rename = "BROWSER_RUNTIME_STATE")]
    BrowserRuntimeStateChanged(BrowserRuntimeStateEvent),
    #[serde(rename = "NAVIGATION_STATE")]
    NavigationStateChanged(NavigationStateEvent),
    #[serde(rename = "PERCEPTION_STATE")]
    PerceptionStateChanged(PerceptionStateEvent),
    #[serde(rename = "ACTION_PROPOSAL")]
    ActionProposed(ActionProposalEvent),
    #[serde(rename = "POLICY_DECISION")]
    PolicyDecision(PolicyDecisionEvent),
    #[serde(rename = "EXECUTION_RESULT")]
    ExecutionResult(ExecutionResultEvent),
    #[serde(rename = "VERIFICATION_OUTCOME")]
    VerificationOutcome(VerificationOutcomeEvent),
    #[serde(rename = "AUTHORIZATION_REQUIRED")]
    AuthorizationRequired(AuthorizationRequiredEvent),
    #[serde(rename = "AUTHORIZATION_RESOLVED")]
    AuthorizationResolved(AuthorizationResolvedEvent),
    #[serde(rename = "AGENT_CURSOR_STATE")]
    AgentCursorStateChanged(AgentCursorStateEvent),
    #[serde(rename = "HUMAN_TAKEOVER_STATE")]
    HumanTakeoverStateChanged(HumanTakeoverStateEvent),
    #[serde(rename = "MODEL_STATE")]
    ModelStateChanged(ModelStateEvent),
    #[serde(rename = "EGRESS_STATE")]
    EgressStateChanged(EgressStateEvent),
    #[serde(rename = "ACTION_DURABLE_STATE")]
    ActionDurableStateChanged(ActionDurableStateEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct BrowserRuntimeStateEvent {
    pub available: bool,
    pub state: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigationStateEvent {
    pub task_id: String,
    pub origin: Option<String>,
    pub url: Option<String>,
    pub loader_id: Option<String>,
    pub frame_id: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerceptionStateEvent {
    pub task_id: String,
    pub epoch: u64,
    pub frame_id: String,
    pub loader_id: String,
    pub actionable_count: usize,
    pub bounded_exceeded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionProposalEvent {
    pub task_id: String,
    pub action: String,
    pub action_class: String,
    pub semantic_reference: Option<String>,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyDecisionEvent {
    pub task_id: String,
    pub action_class: String,
    pub tier: String,
    pub verdict: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionResultEvent {
    pub task_id: String,
    pub action: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub post_state: Option<String>,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationOutcomeEvent {
    pub task_id: String,
    pub outcome: String,
    pub evidence_count: u32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationRequiredEvent {
    pub task_id: String,
    pub tier: String,
    pub summary: String,
    /// Backend-issued commitment reference. The frontend echoes it back to
    /// identify the request; it never authors the authorized facts.
    pub commitment_hash: String,
}

/// Resolution of a backend-issued authorization request. Emitted only by
/// the backend after it validated the user's response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationResolvedEvent {
    pub task_id: String,
    /// "APPROVED" or "DENIED".
    pub decision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCursorStateEvent {
    pub active: bool,
    pub state: String,
    pub target_frame_id: Option<String>,
    pub target_loader_id: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub reason: Option<String>,
}

/// Real model artifact state, derived from the pinned manifest and the
/// installed-artifact check. Never claims verification that has not run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStateEvent {
    pub model_id: String,
    pub quantization: String,
    /// "UNLOADED" / "LOADED" / "PARKED".
    pub residency: String,
    /// "VERIFIED_LOADED" / "PRESENT_UNVERIFIED" / "UNAVAILABLE".
    pub availability: String,
    /// Pinned SHA-256 from the manifest (integrity display value).
    pub sha256: String,
    pub reason: Option<String>,
}

/// Real egress enforcement state. The mode comes from runtime config;
/// enforcement_active is only true when the platform enforcement runtime
/// (proxy + pf helper) is actually active.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressStateEvent {
    /// Configured mode: "ENFORCED" / "DISABLED".
    pub mode: String,
    pub enforcement_active: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanTakeoverStateEvent {
    pub task_id: String,
    pub mode: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDurableStateEvent {
    pub task_id: String,
    pub durable_id: String,
    pub durable_state: String,
    pub action_commitment_hash: String,
}
