// Vault import.
//
// C-143: import paths never expose raw secrets into model context or
//        frontend state. Imported items are represented to the UI layer
//        only as VaultItemRef (metadata) plus a redacted secret descriptor
//        for payment fields.
//
// Import authorization is task-authority-driven and audit-logged.

use crate::core::vault::items::{BlindIndexValue, VaultItemRef};
use crate::core::vault::secret::VaultPassword;
use crate::core::vault::keys::KeyHierarchy;

pub struct VaultImport;

impl VaultImport {
    /// Import a vault item under the given key hierarchy. The raw secret
    /// never leaks to model context or frontend state.
    ///
    /// This is the structural import entry point; the actual persistence
    /// (SQLite) is scheduled.
    /// Derive a blind-index lookup value for an origin under the given
    /// hierarchy. The origin plaintext is not stored; only the blind-index
    /// value is.
    pub fn blind_index_for(authority: &KeyHierarchy, origin: &str) -> BlindIndexValue {
        let idx = crate::core::vault::blind_index::blind_index_for(
            &authority.blind_index_key,
            origin,
        );
        BlindIndexValue { value: idx }
    }

    /// Import a vault item under the given key hierarchy. The raw secret
    /// never leaks to model context or frontend state.
    ///
    /// This is the structural import entry point; the actual persistence
    /// (SQLite) is scheduled.
    pub fn import_item(
        _hierarchy: &KeyHierarchy,
        _master_password: &VaultPassword,
        _redacted_descriptor: Option<crate::core::verification::redaction::RedactedSecretDescriptor>,
    ) -> Result<VaultItemRef, crate::Error> {
        Err(crate::Error::NotImplemented(
            "VaultImport::import_item persistence is scheduled".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vault::header::Argon2Parameters;
    use crate::core::vault::header::VaultHeader;

    #[test]
    fn blind_index_for_does_not_leak_origin_or_key() {
        let header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw = VaultPassword::new(b"hunter2");
        let kh = KeyHierarchy::from_master_password(&pw, header).unwrap();
        let bi = VaultImport::blind_index_for(&kh, "https://payee.example");
        let import = VaultImport;
        assert!(!format!("{:?}", bi).contains("payee.example"));
        assert!(!format!("{:?}", kh).contains("hunter2"));
    }
}
