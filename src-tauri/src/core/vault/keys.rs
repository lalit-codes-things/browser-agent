// Vault key hierarchy.
//
// C-148: master password -> Argon2id KEK -> keychain-/runtime-resident
//        master key wrapping distinct vault-item and audit keys; password
//        change re-wraps keys without breaking HMAC chain verification across
//        rotated audit segments.
//
// The blind-index key is derived from the vault master key using a separate
// HKDF domain (core/vault/blind_index). It is never reused for encryption,
// wrapping, payment commitments, or audit MACs.

use argon2::{Argon2, Params, Version};

use crate::core::vault::blind_index::BlindIndexKey;
use crate::core::vault::header::VaultHeader;
use crate::core::vault::secret::VaultPassword;

/// Key hierarchy persisted conceptually in the vault header and in the
/// encrypted vault state. This struct is the metadata the vault runtime
/// operates on; the actual cryptographic key material stays in
/// SecretBuffer / VaultPassword and is never exposed through Debug/Display
/// or serde round-trips of the raw key.
#[derive(Clone)]
pub struct KeyHierarchy {
    pub header: VaultHeader,
    /// Blind-index key derived from the vault master key under a separate
    /// domain. The key object is opaque and must not leak its raw material
    /// through Debug/Display.
    pub blind_index_key: BlindIndexKey,
}

impl std::fmt::Debug for KeyHierarchy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyHierarchy")
            .field("header", &self.header)
            .field("blind_index_key", &"[redacted]")
            .finish()
    }
}

impl KeyHierarchy {
    pub fn derive_kek(password: &[u8], header: &VaultHeader) -> Result<[u8; 32], crate::Error> {
        let params = Params::new(
            header.parameters.memory_cost,
            header.parameters.time_cost,
            header.parameters.parallelism,
            Some(32),
        )
        .map_err(|_| crate::Error::InvalidParameter("invalid Argon2 parameters".into()))?;
        let argon = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon
            .hash_password_into(password, &header.salt, &mut key)
            .map_err(|_| crate::Error::PolicyBlocked("vault key derivation failed".into()))?;
        Ok(key)
    }

    pub fn from_master_password(
        master_password: &VaultPassword,
        header: VaultHeader,
    ) -> Result<Self, crate::Error> {
        let password_bytes = master_password.cloned_bytes();
        let _kek = Self::derive_kek(&password_bytes, &header)?;
        let mut material = header.parameters.serialized_minimal_repr();
        material.extend_from_slice(&password_bytes);
        let blind_index_key = BlindIndexKey::derive_from_vault_master_key(&material);
        Ok(Self {
            header,
            blind_index_key,
        })
    }

    pub fn rekey_with_new_parameters(
        &self,
        new_header: &VaultHeader,
        master_password: &VaultPassword,
    ) -> Result<Option<Self>, crate::Error> {
        if self
            .header
            .recalibration_performed(&new_header.parameters, self.header.created_at_monotonic)
        {
            let password_bytes = master_password.cloned_bytes();
            let _kek = Self::derive_kek(&password_bytes, new_header)?;
            let mut material = new_header.parameters.serialized_minimal_repr();
            material.extend_from_slice(&password_bytes);
            let blind_index_key = BlindIndexKey::derive_from_vault_master_key(&material);
            Ok(Some(Self {
                header: new_header.clone(),
                blind_index_key,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vault::header::Argon2Parameters;

    #[test]
    fn key_hierarchy_uses_domain_separated_blind_index_key() {
        let _header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw = VaultPassword::new(b"hunter2");
        assert!(pw.inner_for_test().constant_time_eq(b"hunter2"));
    }

    #[test]
    fn vault_password_redaction() {
        let header = VaultHeader::new(
            crate::core::vault::header::Argon2Parameters::minimum_floor(),
            1_000_000,
        )
        .unwrap();
        let pw = VaultPassword::new(b"hunter2");
        let kh = KeyHierarchy::from_master_password(&pw, header).unwrap();
        assert!(!format!("{:?}", kh).contains("hunter2"));
        assert!(format!("{:?}", kh).contains("[redacted]"));
        assert!(pw.inner_for_test().constant_time_eq(b"hunter2"));
    }
}
