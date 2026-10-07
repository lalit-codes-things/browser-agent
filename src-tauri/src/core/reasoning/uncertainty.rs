// Uncertainty-triggered reasoning.
//
// C-54: LLM-every-step is replaced by uncertainty-triggered reasoning once
//        Phase 6 validates the shift; deterministic skills/primitives run
//        without LLM calls until defined uncertainty/recovery triggers occur.
//
// C-55: uncertainty triggers enumerated in triggers.rs.
//
// This module makes the decision gate explicit rather than pretending the
// full trigger evaluation exists yet.

use crate::core::reasoning::triggers::{UncertaintyTrigger, UncertaintyEvaluator};

pub struct UncertaintyDecision {
    pub reason: Option<UncertaintyTrigger>,
    pub should_reason: bool,
}

pub struct UncertaintyEngine;

impl UncertaintyEngine {
    pub fn evaluate(trigger: Option<UncertaintyTrigger>) -> UncertaintyDecision {
        let should_reason = UncertaintyEvaluator::should_reason(trigger.clone());
        UncertaintyDecision {
            reason: trigger,
            should_reason,
        }
    }
}
