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
        let id = format!("eph-{:08x}", rand::random::<u32>());
        Ok(BrowserProfile {
            id,
            kind: ProfileKind::Ephemeral,
            storage_used_bytes: 0,
        })
    }

    pub fn create_persistent(
        origin: &str,
        expires_at: Option<String>,
        storage_cap_bytes: Option<u64>,
    ) -> Result<BrowserProfile, crate::Error> {
        let id = format!("pers-{:08x}", rand::random::<u32>());
        Ok(BrowserProfile {
            id,
            kind: ProfileKind::Persistent {
                origin: origin.to_string(),
                expires_at,
                storage_cap_bytes,
            },
            storage_used_bytes: 0,
        })
    }

    pub fn purge(id: &str) -> Result<(), crate::Error> {
        if id.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "profile id is required".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_ephemeral_profile() {
        let profile = ProfileManager::create_ephemeral().unwrap();
        assert!(profile.id.starts_with("eph-"));
        assert_eq!(profile.kind, ProfileKind::Ephemeral);
    }

    #[test]
    fn creates_persistent_profile() {
        let profile =
            ProfileManager::create_persistent("https://example.com", None, Some(1024 * 1024))
                .unwrap();
        assert!(profile.id.starts_with("pers-"));
        match profile.kind {
            ProfileKind::Persistent {
                origin,
                storage_cap_bytes,
                ..
            } => {
                assert_eq!(origin, "https://example.com");
                assert_eq!(storage_cap_bytes, Some(1024 * 1024));
            }
            _ => panic!("expected persistent"),
        }
    }
}
