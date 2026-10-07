// Idempotency.
//
// C-06, C-11: retry behavior is class-driven and verification-driven.
//        never auto-retry an irreversible operation merely because the UI
//        did not change or the model thinks it failed.

use crate::core::policy::classes::SideEffectClass;

pub struct Idempotency;

impl Idempotency {
    pub fn may_retry_without_verification(class: SideEffectClass) -> bool {
        matches!(class, SideEffectClass::Read | SideEffectClass::ReversibleWrite)
    }
}
