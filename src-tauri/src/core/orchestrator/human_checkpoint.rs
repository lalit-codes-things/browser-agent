// Human Checkpoint.
//
// Intervention state with distinct epoch semantics so user-caused
// mutations do not burn recovery budgets or trigger livelock (C-90).
//
// Phase 3 prepares the state; Phase 8 completes and operationalizes it
// during human checkpoints.
//
// This module intentionally exposes typed intervention state rather than
// fake "fallback" UX. The frontend renders whatever the backend issues.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum InterventionType {
    Capture,
    TwoFactor,
    Passkey,
    ExplicitUserCheckpoint,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionState {
    pub intervention_type: InterventionType,
    pub reason: String,
    pub expiry: Option<String>,
    pub resume_state: Option<String>,
}

pub struct HumanCheckpoint;

impl HumanCheckpoint {
    pub fn enter(_kind: InterventionType, _reason: &str) -> InterventionState {
        InterventionState {
            intervention_type: _kind,
            reason: _reason.into(),
            expiry: None,
            resume_state: None,
        }
    }
}
