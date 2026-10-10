// Action classification.
//
// C-06: action classes are exactly READ / REVERSIBLE_WRITE / IRREVERSIBLE /
//        UNKNOWN -> IRREVERSIBLE.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum SideEffectClass {
    #[default]
    Read,
    ReversibleWrite,
    Irreversible,
    Unknown,
}

impl SideEffectClass {
    /// C-09: UNKNOWN consequential actions are treated as IRREVERSIBLE.
    pub fn effective_class(self) -> Self {
        match self {
            Self::Unknown => Self::Irreversible,
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_effective_is_irreversible() {
        assert_eq!(
            SideEffectClass::Unknown.effective_class(),
            SideEffectClass::Irreversible
        );
    }

    #[test]
    fn read_effective_is_read() {
        assert_eq!(
            SideEffectClass::Read.effective_class(),
            SideEffectClass::Read
        );
    }

    #[test]
    fn reversible_write_effective_is_reversible_write() {
        assert_eq!(
            SideEffectClass::ReversibleWrite.effective_class(),
            SideEffectClass::ReversibleWrite
        );
    }
}
