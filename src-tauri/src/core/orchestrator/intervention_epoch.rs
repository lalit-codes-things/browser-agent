// Intervention Epoch.
//
// Intervention epochs are distinct from ordinary execution epochs so that
// human checkpoints produce stable state without burning recovery budgets
// or triggering livelock (C-90). Phase 3 prepares this state; Phase 8
// operationalizes it.
//
// This is a typed boundary, not a UX convenience.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterventionEpoch {
    pub epoch: u64,
    pub kind: crate::core::orchestrator::human_checkpoint::InterventionType,
    pub issued_at: Option<String>,
}
