// Profiles.
//
// C-12: high-risk profiles ephemeral by default.
// C-13: persistent per-site high-risk profiles require explicit user opt-in
//        and must have expiry, storage caps, and purge.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProfileKind {
    Ephemeral,
    Persistent {
        origin: String,
        expires_at: Option<String>,
        storage_cap_bytes: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub id: String,
    pub kind: ProfileKind,
    pub storage_used_bytes: u64,
}

pub struct ProfileManager;

impl ProfileManager {
    pub fn create_ephemeral() -> Result<BrowserProfile, crate::Error> {
        Err(crate::Error::NotImplemented(
            "ProfileManager::create_ephemeral is scheduled".into(),
        ))
    }

    pub fn create_persistent(
        _origin: &str,
        _expires_at: Option<String>,
        _storage_cap_bytes: Option<u64>,
    ) -> Result<BrowserProfile, crate::Error> {
        Err(crate::Error::NotImplemented(
            "ProfileManager::create_persistent is scheduled".into(),
        ))
    }

    pub fn purge(_id: &str) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "ProfileManager::purge is scheduled".into(),
        ))
    }
}
