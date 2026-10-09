// Structured inference schema.
//
// C-142: the model action surface is exactly the typed operations below.
//        arbitrary model-controlled JavaScript is prohibited.
// C-22: the model knows only REQUEST_CONFIRMATION; it does not know or
//        control the biometric mechanism.
//
// The schema is the contract between inference and execution/policy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelAction {
    Navigate {
        url: String,
    },
    Click {
        semantic_reference: String,
    },
    Type {
        semantic_reference: String,
        text: String,
    },
    Select {
        semantic_reference: String,
        value: String,
    },
    Scroll {
        direction: ScrollDirection,
        amount: Option<u32>,
    },
    Wait {
        reason: String,
    },
    PressKey {
        key: String,
    },
    SecureFill {
        field_reference: String,
    },
    RequestConfirmation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ModelAction {
    pub fn action_class(&self) -> crate::core::policy::classes::SideEffectClass {
        use crate::core::policy::classes::SideEffectClass;
        match self {
            ModelAction::Navigate { .. }
            | ModelAction::Click { .. }
            | ModelAction::Select { .. }
            | ModelAction::Scroll { .. }
            | ModelAction::Wait { .. }
            | ModelAction::PressKey { .. } => SideEffectClass::Read,
            ModelAction::Type { .. } | ModelAction::SecureFill { .. } => {
                SideEffectClass::ReversibleWrite
            }
            ModelAction::RequestConfirmation => SideEffectClass::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::policy::classes::SideEffectClass;

    #[test]
    fn navigate_is_read() {
        assert_eq!(
            ModelAction::Navigate {
                url: "https://example.test".into()
            }
            .action_class(),
            SideEffectClass::Read
        );
    }

    #[test]
    fn request_confirmation_is_unknown() {
        assert_eq!(
            ModelAction::RequestConfirmation.action_class(),
            SideEffectClass::Unknown
        );
    }
}
