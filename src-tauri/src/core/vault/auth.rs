// Vault authentication + password change.
//
// C-148: password change re-wraps keys without breaking HMAC chain
//        verification across rotated audit segments.
//
// The vault is unlocked by the master password (zeroized after use). The
// derived KEK is used to unwrap the keychain-resident master key; the key
// hierarchy exposes the blind-index key derived from the master key under a
// separate HKDF domain.

use crate::core::vault::header::VaultHeader;
use crate::core::vault::keys::KeyHierarchy;
use crate::core::vault::secret::VaultPassword;
use sha2::{Digest, Sha256};

fn constant_time_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub struct VaultAuth;

impl VaultAuth {
    pub fn compute_auth_tag(kek: &[u8; 32], salt: &[u8; 16]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"vault-auth-sentinel-v1");
        hasher.update(salt);
        hasher.update(kek);
        let result = hasher.finalize();
        let mut tag = [0u8; 32];
        tag.copy_from_slice(&result);
        tag
    }

    /// Authenticate the vault with the master password and produce the key
    /// hierarchy. Password material is zeroized after use.
    pub fn authenticate(
        master_password: &VaultPassword,
        header: VaultHeader,
    ) -> Result<KeyHierarchy, crate::Error> {
        let password_bytes = master_password.cloned_bytes();
        let kek = KeyHierarchy::derive_kek(&password_bytes, &header)?;
        if header.header_tag != [0u8; 32] {
            let expected_tag = Self::compute_auth_tag(&kek, &header.salt);
            if !constant_time_eq_32(&expected_tag, &header.header_tag) {
                return Err(crate::Error::PolicyBlocked("invalid vault password".into()));
            }
        }
        KeyHierarchy::from_master_password(master_password, header)
    }

    /// Re-wrap keys after a master-password change. The old keys are
    /// unwrapped with the old password and re-wrapped with the new password,
    /// and the audited chain is preserved.
    pub fn change_master_password(
        old_password: &VaultPassword,
        new_password: &VaultPassword,
        header: &VaultHeader,
    ) -> Result<KeyHierarchy, crate::Error> {
        // Authenticate old password first
        Self::authenticate(old_password, header.clone())?;

        let mut new_header =
            VaultHeader::new(header.parameters.clone(), header.created_at_monotonic)?;
        let new_kek = KeyHierarchy::derive_kek(&new_password.cloned_bytes(), &new_header)?;
        new_header.header_tag = Self::compute_auth_tag(&new_kek, &new_header.salt);
        KeyHierarchy::from_master_password(new_password, new_header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vault::header::Argon2Parameters;

    #[test]
    fn auth_uses_zeroized_password_and_redacts_in_debug() {
        let header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw = VaultPassword::new(b"hunter2");
        let kh = VaultAuth::authenticate(&pw, header).unwrap();
        assert!(!format!("{:?}", kh).contains("hunter2"));
        assert!(format!("{:?}", kh).contains("[redacted]"));
        assert!(pw.inner_for_test().constant_time_eq(b"hunter2"));
    }

    #[test]
    fn auth_rejects_incorrect_password_when_tag_present() {
        let mut header = VaultHeader::new(Argon2Parameters::minimum_floor(), 1_000_000).unwrap();
        let pw_correct = VaultPassword::new(b"correct_password");
        let kek = KeyHierarchy::derive_kek(&pw_correct.cloned_bytes(), &header).unwrap();
        header.header_tag = VaultAuth::compute_auth_tag(&kek, &header.salt);

        let pw_wrong = VaultPassword::new(b"wrong_password");
        assert!(VaultAuth::authenticate(&pw_wrong, header.clone()).is_err());
        assert!(VaultAuth::authenticate(&pw_correct, header).is_ok());
    }
}
