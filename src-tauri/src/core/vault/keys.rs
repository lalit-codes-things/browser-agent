// Vault key hierarchy.
//
// C-148: master password -> Argon2id KEK -> Keychain-resident master key
//        wrapping distinct vault-item and audit keys; password change re-wraps
//        keys without breaking HMAC chain verification across rotated segments.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHierarchy {
    pub kek_algorithm: String,
    pub keychain_master_key_wrapped: bool,
    pub vault_item_key_wrapped: bool,
    pub audit_key_wrapped: bool,
}
