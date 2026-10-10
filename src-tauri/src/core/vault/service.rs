// Vault service.
//
// Manages vault lifecycle: initialization, authentication, locking,
// credential metadata storage, and AEAD-encrypted secret handling.
// C-143: Raw secrets never cross into the frontend.

use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use parking_lot::Mutex;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::core::policy::scopes::CredentialScope;
use crate::core::vault::auth::VaultAuth;
use crate::core::vault::blind_index::blind_index_for;
use crate::core::vault::header::{Argon2Parameters, VaultHeader};
use crate::core::vault::items::{
    BlindIndexValue, HttpsCheckState, IdnHomographCheckState, VaultItemRef,
};
use crate::core::vault::keys::KeyHierarchy;
use crate::core::vault::secret::VaultPassword;
use crate::error::Error;
use crate::security::clocks::MonotonicClock;
use crate::storage::sqlite::SqliteStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStatus {
    pub is_initialized: bool,
    pub is_unlocked: bool,
    pub item_count: usize,
}

pub struct VaultService {
    unlocked_state: Mutex<Option<UnlockedVaultState>>,
    store: Arc<SqliteStore>,
}

struct UnlockedVaultState {
    hierarchy: KeyHierarchy,
    kek: [u8; 32],
}

impl VaultService {
    pub fn new(store: Arc<SqliteStore>) -> Self {
        Self {
            unlocked_state: Mutex::new(None),
            store,
        }
    }

    pub fn status(&self) -> Result<VaultStatus, Error> {
        let is_initialized = self.get_stored_header()?.is_some();
        let is_unlocked = self.unlocked_state.lock().is_some();
        let item_count = self.store.list_vault_items()?.len();
        Ok(VaultStatus {
            is_initialized,
            is_unlocked,
            item_count,
        })
    }

    pub fn create_vault(&self, password: &str) -> Result<(), Error> {
        if password.trim().is_empty() {
            return Err(Error::InvalidParameter("password cannot be empty".into()));
        }
        if self.get_stored_header()?.is_some() {
            return Err(Error::StateMismatch("vault already initialized".into()));
        }

        let mut header = VaultHeader::new(
            Argon2Parameters::minimum_floor(),
            MonotonicClock::now_nanos(),
        )?;
        let vault_pw = VaultPassword::new(password.as_bytes());
        let kek = KeyHierarchy::derive_kek(password.as_bytes(), &header)?;
        header.header_tag = VaultAuth::compute_auth_tag(&kek, &header.salt);
        let hierarchy = VaultAuth::authenticate(&vault_pw, header.clone())?;

        self.save_stored_header(&header)?;
        *self.unlocked_state.lock() = Some(UnlockedVaultState { hierarchy, kek });
        Ok(())
    }

    pub fn unlock(&self, password: &str) -> Result<(), Error> {
        if password.trim().is_empty() {
            return Err(Error::InvalidParameter("password cannot be empty".into()));
        }
        let header = self
            .get_stored_header()?
            .ok_or_else(|| Error::StateMismatch("vault not initialized".into()))?;

        let vault_pw = VaultPassword::new(password.as_bytes());
        let hierarchy = VaultAuth::authenticate(&vault_pw, header.clone())?;
        let kek = KeyHierarchy::derive_kek(password.as_bytes(), &header)?;

        *self.unlocked_state.lock() = Some(UnlockedVaultState { hierarchy, kek });
        Ok(())
    }

    pub fn lock(&self) {
        *self.unlocked_state.lock() = None;
    }

    pub fn change_password(&self, old_pw: &str, new_pw: &str) -> Result<(), Error> {
        if new_pw.trim().is_empty() {
            return Err(Error::InvalidParameter(
                "new password cannot be empty".into(),
            ));
        }
        let current_header = self
            .get_stored_header()?
            .ok_or_else(|| Error::StateMismatch("vault not initialized".into()))?;

        let old_vault_pw = VaultPassword::new(old_pw.as_bytes());
        let new_vault_pw = VaultPassword::new(new_pw.as_bytes());

        // Re-wraps keys preserving metadata
        let new_hierarchy =
            VaultAuth::change_master_password(&old_vault_pw, &new_vault_pw, &current_header)?;
        let new_kek = KeyHierarchy::derive_kek(new_pw.as_bytes(), &new_hierarchy.header)?;

        self.save_stored_header(&new_hierarchy.header)?;
        *self.unlocked_state.lock() = Some(UnlockedVaultState {
            hierarchy: new_hierarchy,
            kek: new_kek,
        });
        Ok(())
    }

    pub fn add_item(
        &self,
        origin: &str,
        account_label: &str,
        secret: &str,
    ) -> Result<VaultItemRef, Error> {
        let guard = self.unlocked_state.lock();
        let state = guard
            .as_ref()
            .ok_or_else(|| Error::PolicyBlocked("vault is locked".into()))?;

        if origin.trim().is_empty() || account_label.trim().is_empty() || secret.trim().is_empty() {
            return Err(Error::InvalidParameter("all fields are required".into()));
        }

        let blind_idx_bytes = blind_index_for(&state.hierarchy.blind_index_key, origin);
        let blind_idx = BlindIndexValue {
            value: blind_idx_bytes,
        };
        let id = format!("item-{:08x}", rand::random::<u32>());

        // ChaCha20Poly1305 encryption
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&state.kek));
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher
            .encrypt(nonce, secret.as_bytes())
            .map_err(|e| Error::Internal(format!("encryption failed: {e}")))?;

        let https_state = if origin.starts_with("https://") {
            HttpsCheckState::Pass
        } else {
            HttpsCheckState::Fail
        };

        let item = VaultItemRef {
            id: id.clone(),
            origin: origin.to_string(),
            account_label: account_label.to_string(),
            scope: CredentialScope::ExactOrigin {
                origin: origin.to_string(),
            },
            https_check_state: https_state,
            idn_homograph_check_state: IdnHomographCheckState::Pass,
            last_used_task: None,
            redacted_secret: None,
        };

        self.store
            .save_vault_item(&item, &blind_idx, &encrypted, &nonce_bytes)?;

        Ok(item)
    }

    pub fn list_items(&self) -> Result<Vec<VaultItemRef>, Error> {
        self.store.list_vault_items()
    }

    pub fn delete_item(&self, id: &str) -> Result<bool, Error> {
        let guard = self.unlocked_state.lock();
        if guard.is_none() {
            return Err(Error::PolicyBlocked("vault is locked".into()));
        }
        self.store.delete_vault_item(id)
    }

    /// Authorized secure fill: decrypts the secret natively for the given item ID
    /// and fills it via the execution engine without passing raw secret to React.
    pub fn get_secret_for_secure_fill(&self, id: &str) -> Result<String, Error> {
        let guard = self.unlocked_state.lock();
        let state = guard
            .as_ref()
            .ok_or_else(|| Error::PolicyBlocked("vault is locked".into()))?;

        let (encrypted, nonce_bytes) = self
            .store
            .get_vault_item_secret(id)?
            .ok_or_else(|| Error::InvalidParameter("vault item not found".into()))?;

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&state.kek));
        let nonce = Nonce::from_slice(&nonce_bytes);
        let decrypted = cipher
            .decrypt(nonce, encrypted.as_ref())
            .map_err(|e| Error::Internal(format!("decryption failed: {e}")))?;

        String::from_utf8(decrypted)
            .map_err(|e| Error::Internal(format!("secret not valid utf-8: {e}")))
    }

    fn get_stored_header(&self) -> Result<Option<VaultHeader>, Error> {
        match self.store.get_settings()? {
            Some(settings) => {
                if settings.vault.master_key_storage.starts_with("header:") {
                    let json = &settings.vault.master_key_storage["header:".len()..];
                    let header: VaultHeader = serde_json::from_str(json)
                        .map_err(|e| Error::Internal(format!("invalid stored header: {e}")))?;
                    Ok(Some(header))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    fn save_stored_header(&self, header: &VaultHeader) -> Result<(), Error> {
        let mut settings = self.store.get_settings()?.unwrap_or_default();
        let json = serde_json::to_string(header)
            .map_err(|e| Error::Internal(format!("failed to serialize header: {e}")))?;
        settings.vault.master_key_storage = format!("header:{json}");
        self.store.save_settings(&settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_vault_lifecycle() {
        let store = Arc::new(SqliteStore::open_in_memory().unwrap());
        let service = VaultService::new(store);

        let initial_status = service.status().unwrap();
        assert!(!initial_status.is_initialized);
        assert!(!initial_status.is_unlocked);

        // Create vault
        service.create_vault("master_secret_123").unwrap();
        let status = service.status().unwrap();
        assert!(status.is_initialized);
        assert!(status.is_unlocked);

        // Add item
        let item = service
            .add_item("https://test.example.com", "admin", "super_password_456")
            .unwrap();
        assert_eq!(item.account_label, "admin");

        // Lock
        service.lock();
        assert!(!service.status().unwrap().is_unlocked);
        assert!(service.add_item("https://other.com", "u", "p").is_err());

        // Unlock with wrong password
        assert!(service.unlock("wrong_password").is_err());
        assert!(!service.status().unwrap().is_unlocked);

        // Unlock with correct password
        service.unlock("master_secret_123").unwrap();
        assert!(service.status().unwrap().is_unlocked);

        // List items
        let items = service.list_items().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, item.id);

        // Secure fill access
        let secret = service.get_secret_for_secure_fill(&item.id).unwrap();
        assert_eq!(secret, "super_password_456");

        // Password change
        service
            .change_password("master_secret_123", "new_master_secret_789")
            .unwrap();

        // Lock and unlock with new password
        service.lock();
        assert!(service.unlock("master_secret_123").is_err());
        service.unlock("new_master_secret_789").unwrap();
        assert!(service.status().unwrap().is_unlocked);

        // Delete item
        assert!(service.delete_item(&item.id).unwrap());
        assert!(service.list_items().unwrap().is_empty());
    }
}
