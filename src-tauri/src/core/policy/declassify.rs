// Declassification.
//
// C-30: session-surviving taint is preserved until explicit declassification.
//
// Declassification is a trusted runtime operation, not something the model
// or page can trigger.

use crate::core::policy::taint::TaintFlag;

pub struct DeclassificationPolicy;

impl DeclassificationPolicy {
    pub fn may_declassify(_from: TaintFlag, _to: TaintFlag, _justification: &str) -> bool {
        // Placeholder: real declassification is policy-gated and audited.
        false
    }
}
