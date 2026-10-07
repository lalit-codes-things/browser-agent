// Uncertainty triggers.
//
// C-55: the uncertainty triggers include target-resolution failure,
//        ambiguity, unexpected state, failed invariant, unresolved
//        parameters, contradictory state, policy clarification, and
//        recovery-mode entry.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UncertaintyTrigger {
    TargetResolutionFailure,
    Ambiguity,
    UnexpectedState,
    FailedInvariant,
    UnresolvedParameters,
    ContradictoryState,
    PolicyClarification,
    RecoveryModeEntry,
}

pub struct UncertaintyEvaluator;

impl UncertaintyEvaluator {
    pub fn should_reason(trigger: Option<UncertaintyTrigger>) -> bool {
        // Phase 6: replace LLM-every-step with uncertainty-triggered
        // reasoning once validated. For now, we model the trigger gate
        // explicitly and leave the baseline deterministic path underneath.
        trigger.is_some()
    }
}
