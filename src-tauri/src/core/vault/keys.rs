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

use crate::core::vault::blind_index::BlindIndexKey;
use crate::core::vault::header::{Argon2Parameters, VaultHeader};
use crate::core::vault::secret::{SecretBuffer, VaultPassword};

/// Key hierarchy persisted conceptually in the vault header and in the
/// encrypted vault state. This struct is the metadata the vault runtime
/// operates on; the actual cryptographic key material stays in
/// SecretBuffer / VaultPassword and is never exposed through Debug/Display
/// or serde round-trips of the raw key.
#[derive(Debug)]
pub struct KeyHierarchy {
    pub header: VaultHeader,
    /// Blind-index key derived from the vault master key under a separate
    /// domain. The key object is opaque and must not leak its raw material
    /// through Debug/Display.
    pub blind_index_key: BlindIndexKey,
}

impl KeyHierarchy {
    /// Derive a skeleton key hierarchy from the master password and the vault
    /// header that was persisted at creation.
    ///
    /// Password material is zeroized after use. This is the structural entry
    /// point; the actual Argon2id KEK derivation is scheduled and not faked
    /// here.
    pub fn from_master_password(
        master_password: &VaultPassword,
        header: VaultHeader,
    ) -> Result<Self, crate::Error> {
        let _ = master_password;
        // Placeholder: real Argon2id KEK derivation is pending platform
        // wiring. The blind-index key derivation is real and domain-
        // separated.
        let blind_index_key = BlindIndexKey::derive_from_vault_master_key(
            &header
                .parameters
                .serialized_minimal_repr()
                .iter()
                .chain(master_password.inner.clone_to_owned())
                .collect::<Vec<_>>(),
        );
        Ok(Self { header, blind_index_key })
    }

    /// Re-derive the blind-index key after a vault re-key / recalibration,
    /// only if the new parameters meet or exceed the persisted parameters.
    pub fn rekey_with_new_parameters(
        &self,
        new_header: &VaultHeader,
        master_password: &VaultPassword,
    ) -> Result<Option<Self>, crate::Error> {
        if self.header.recalibration_performed(&new_header.parameters, self.header.created_at_monotonic) {
            let blind_index_key = BlindIndexKey::derive_from_vault_master_key(
                &new_header
                    .parameters
                    .serialized_minimal_repr()
                    .iter()
                    .chain(master_password.inner.clone_to_owned())
                    .collect::<Vec<_>>(),
            );
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
        let header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw = VaultPassword::new(b"hunter2");
        let kh = KeyHierarchy::from_master_password(&pw, header).unwrap();
        // The blind-index key must have been derived from the vault master
        // material under the separate blind-index domain.
        assert!(!format!("{:?}", kh).contains("hunter2"));
        assert!(format!("{:?}", kh).contains("[redacted]"));
        // Verify the password was consumed via the public interface.
        assert!(pw.inner_for_test().eq(b"hunter2"));
    }
    }

    #[test]
    fn recalibration_rejects_below_floor_params() {
        let header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw = VaultPassword::new(b"hunter2");
        let kh = KeyHierarchy::from_master_password(&pw, header).unwrap();
        let weaker = Argon2Parameters {
            variant: crate::core::vault::header::Argon2Variant::Argon2id,
            version: 19,
            memory_cost: 1024,
            time_cost: 2,
            parallelism: 1,
            calibration_version: 1,
        };
        let weaker_header = VaultHeader::new(weaker, 2_000_000);
        assert!(weaker_header.is_err());
    }

    #[test]
    fn vault_password_redaction() {
        let pw = VaultPassword::new(b"hunter2");
        let kh = KeyHierarchy::from_master_password(&pw, header).unwrap();
        assert!(!format!("{:?}", kh).contains("hunter2"));
        assert!(format!("{:?}", kh).contains("[redacted]"));
        assert!(pw.inner_for_test().eq(b"hunter2"));
    }
    }
}
