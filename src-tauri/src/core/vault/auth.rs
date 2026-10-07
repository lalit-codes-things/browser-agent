// Vault authentication.
//
// C-148: password change re-wraps keys without breaking HMAC chain
//        verification across rotated segments.

pub struct VaultAuth;

impl VaultAuth {
    pub fn authenticate(_master_password: &str) -> Result<crate::core::vault::keys::KeyHierarchy, crate::Error> {
        Err(crate::Error::NotImplemented("VaultAuth::authenticate is scheduled".into()))
    }

    pub fn change_master_password(_old: &str, _new: &str) -> Result<crate::core::vault::keys::KeyHierarchy, crate::Error> {
        Err(crate::Error::NotImplemented("VaultAuth::change_master_password is scheduled".into()))
    }
}
