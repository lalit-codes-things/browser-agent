// Execution primitives.
//
// C-142: the execution engine exposes only the typed operations below.
//        arbitrary model-controlled JavaScript is prohibited in all phases.
//
// This module is the API surface between reasoning and the browser runtime.

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
    pub fn execute(_input: ExecutionInput) -> Result<ExecutionResult, crate::Error> {
        // Placeholder: real execution is wired to the browser runtime.
        Err(crate::Error::NotImplemented("ExecutionEngine::execute is scheduled".into()))
    }
}
