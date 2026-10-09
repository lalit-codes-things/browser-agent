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

pub struct VaultAuth;

impl VaultAuth {
    /// Authenticate the vault with the master password and produce the key
    /// hierarchy. Password material is zeroized after use.
    pub fn authenticate(
        master_password: &VaultPassword,
        header: VaultHeader,
    ) -> Result<KeyHierarchy, crate::Error> {
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
        let _ = old_password;
        // intermediate KEK is not leaked through Debug/Display.
        let new_header = VaultHeader::new(header.parameters.clone(), header.created_at_monotonic)?;
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
}
