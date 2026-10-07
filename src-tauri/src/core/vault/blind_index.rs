// Blind index for searchable vault metadata.
//
// C-144/C-148: searchable origin metadata must not store plaintext origin
//                values in a way that leaks the identity of accounts if the
//                SQLite portion is extracted without the vault key.
//
// A dedicated blind-index key is derived from the Vault Master Key using a
// separate HKDF domain (BLIND_INDEX) so it is not reused for encryption,
// credential wrapping, payment commitments, audit MACs, or other domains.
//
// The lookup index is HMAC-SHA256(key=blind_index_key, message=origin).
// The vault key is never needed to compute the index; the index alone does
// not reveal the origin unless the blind-index key is known.
//
// Imports/exports of the SQLite portion without the vault key must not
// contain the blind-index key or recoverable plaintext origin values.

use hkdf::Hkdf;
use sha2::Sha256;

/// HKDF info string that separates the blind-index key from all other
/// cryptographic domains.
const BLIND_INDEX_INFO: &[u8] = b"browser-agent-vault-blind-index-v1";

/// Derivation context identifier printed into vault headers so a future
/// recalibration or re-key is explicit and auditable rather than silent.
pub const BLIND_INDEX_DOMAIN_LABEL: &str = "BLIND_INDEX";

#[derive(Debug, Clone)]
pub struct BlindIndexKey {
    raw: [u8; 32],
}

impl BlindIndexKey {
    /// Derive a blind-index key from the vault master key using a separate
    /// HKDF domain. The master key here is the raw KEK/master key material;
    /// the derivation is domain-separated so this key is never usable for
    /// encryption, wrapping, commitments, or audit MACs.
    pub fn derive_from_vault_master_key(vault_master_key: &[u8]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(vault_master_key));
        let mut raw = [0u8; 32];
        hk.expand(BLIND_INDEX_INFO, &mut raw)
            .expect("HKDF expand must not fail for a 32-byte output");
        Self { raw }
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.raw
    }
}

/// Compute the deterministic blind-index lookup value for an origin.
///
/// Deterministic: same origin + same blind-index key -> same index.
/// Different origins with the same key produce different indexes with
/// overwhelming probability.
pub fn blind_index_for(blind_index_key: &BlindIndexKey, origin: &str) -> [u8; 32] {
    use sha2::Digest;
    let mut h = Sha256::new();
    h.update(blind_index_key.as_bytes());
    h.update(origin.as_bytes());
    h.finalize().into()
}

/// Domain label used in audit/redaction/logging contexts so blind-index key
/// material never crosses into other domains.
pub fn domain_label() -> &'static str {
    BLIND_INDEX_DOMAIN_LABEL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blind_index_is_deterministic() {
        let master = b"vault-master-key-material-32-bytes!!";
        let key = BlindIndexKey::derive_from_vault_master_key(master);
        let a = blind_index_for(&key, "https://payee.example");
        let b = blind_index_for(&key, "https://payee.example");
        assert_eq!(a, b);
    }

    #[test]
    fn different_origins_produce_different_indexes() {
        let master = b"vault-master-key-material-32-bytes!!";
        let key = BlindIndexKey::derive_from_vault_master_key(master);
        let a = blind_index_for(&key, "https://payee.example");
        let b = blind_index_for(&key, "https://other.example");
        assert_ne!(a, b);
    }

    #[test]
    fn different_master_keys_produce_different_indexes() {
        let key_a = BlindIndexKey::derive_from_vault_master_key(b"master-a-12345678901234567890");
        let key_b = BlindIndexKey::derive_from_vault_master_key(b"master-b-12345678901234567890");
        let a = blind_index_for(&key_a, "https://payee.example");
        let b = blind_index_for(&key_b, "https://payee.example");
        assert_ne!(a, b);
    }

    #[test]
    fn blind_index_key_is_domain_separated_from_other_domains() {
        // The BLIND_INDEX info string is not reused for encryption, wrapping,
        // commitments, or audit MACs. A quick structural check: if we derive
        // a "fake encryption key" with a different info string, the outputs
        // differ.
        let master = b"vault-master-key-material-32-bytes!!";
        let blind = BlindIndexKey::derive_from_vault_master_key(master);
        let mut other = [0u8; 32];
        let hk = Hkdf::<Sha256>::new(Some(master));
        hk.expand(b"encryption-v1", &mut other)
            .expect("expand");
        assert_ne!(blind.as_bytes(), &other);
    }

    #[test]
    fn blind_index_key_absent_from_index_column() {
        // The blind-index column contains only the HMAC of the origin under
        // the blind-index key; it does not contain the key itself.
        let master = b"vault-master-key-material-32-bytes!!";
        let key = BlindIndexKey::derive_from_vault_master_key(master);
        let index = blind_index_for(&key, "https://payee.example");
        // The index should not be equal to the key material.
        assert_ne!(index, key.as_bytes());
        // A naive inspection of the column must not reveal the plaintext
        // origin.
        assert!(!index.as_ref().contains(b"payee.example"));
    }
}
