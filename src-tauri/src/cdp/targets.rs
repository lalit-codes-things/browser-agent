// Targets.
//
// C-04, C-70: target lifecycle and auto-attach validation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpTarget {
    pub target_id: String,
    pub type_: TargetType,
    pub url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TargetType {
    Page,
    Utility,
    Other,
}

pub struct TargetRegistry;

impl TargetRegistry {
    pub fn register(_target: CdpTarget) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "TargetRegistry::register is scheduled".into(),
        ))
    }
}
