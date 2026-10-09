use crate::core::reasoning::schema::ModelAction;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionInput {
    pub action: ModelAction,
    pub epoch: u64,
    pub target_frame_id: String,
    pub target_loader_id: String,
    pub policy_tier: crate::core::policy::tiers::AuthorizationTier,
    pub confirmation_commitment_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionResult {
    pub action: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub post_state: Option<String>,
}

pub struct ExecutionEngine;

impl ExecutionEngine {
    pub fn execute(input: ExecutionInput) -> Result<ExecutionResult, crate::Error> {
        if input.target_frame_id.trim().is_empty() || input.target_loader_id.trim().is_empty() {
            return Err(crate::Error::StateMismatch(
                "target frame/loader binding is missing".into(),
            ));
        }
        if input.action.action_class().effective_class()
            != crate::core::policy::classes::SideEffectClass::Read
            && input.policy_tier == crate::core::policy::tiers::AuthorizationTier::None
        {
            return Err(crate::Error::PolicyBlocked(
                "side-effectful action lacks authorization".into(),
            ));
        }
        if matches!(input.action, ModelAction::RequestConfirmation) {
            return Err(crate::Error::PolicyBlocked(
                "confirmation is not a browser execution primitive".into(),
            ));
        }
        Err(crate::Error::NotImplemented(
            "browser transport is unavailable for execution".into(),
        ))
    }
}
