// Action serialization and epoch binding.
//
// C-141: model-proposed actions are bound to the state epoch they were
//        derived from; stale actions are rejected when relevant state
//        changes before execution.
// C-66: every action log records epoch, target/frame/loader identity,
//        semantic reference, policy result including tier, action type,
//        execution result, verification result.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedAction {
    pub epoch: u64,
    pub target_frame_id: String,
    pub target_loader_id: String,
    pub semantic_reference: String,
    pub policy_tier: String,
    pub action_type: String,
    pub execution_result: String,
    pub verification_result: Option<String>,
    pub timestamp: Option<String>,
}
