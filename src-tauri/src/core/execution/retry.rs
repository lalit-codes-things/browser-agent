// Retry policy.
//
// C-11: retry behavior is class-driven and verification-driven.
// C-92: irreversible actions never auto-retried after LIKELY_SUCCESS or
//        UNKNOWN; VERIFIED_FAILURE retries only when Policy proves
//        repeatability.

use crate::core::policy::classes::SideEffectClass;

#[derive(Debug, Clone)]
pub enum RetryDecision {
    Retry,
    DoNotRetry,
    RetryWithPolicyProof,
}

pub struct RetryPolicy;

impl RetryPolicy {
    pub fn decide(class: SideEffectClass, _verification: Option<&str>) -> RetryDecision {
        match class.effective_class() {
            SideEffectClass::Irreversible => {
                // C-92: never auto-retry irreversible after LIKELY_SUCCESS or UNKNOWN.
                RetryDecision::DoNotRetry
            }
            SideEffectClass::Read | SideEffectClass::ReversibleWrite => RetryDecision::Retry,
            SideEffectClass::Unknown => {
                // C-09: unknown consequential actions treated as irreversible.
                RetryDecision::DoNotRetry
            }
        }
    }
}
