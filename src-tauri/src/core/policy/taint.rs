// Taint / provenance.
//
// C-30: session-surviving taint is preserved until explicit declassification.
//
// Page-derived content and tainted values must not be treated as trusted.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaintFlag {
    #[default]
    None,
    PageDerived,
    UnknownSource,
    SensitiveToPublic,
    CredentialMaterial,
}

impl TaintFlag {
    pub fn is_clean(&self) -> bool {
        matches!(self, Self::None)
    }
}
