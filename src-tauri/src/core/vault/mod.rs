// Vault subsystem.
//
// C-148: vault key hierarchy: master password -> Argon2id KEK -> Keychain-
//        resident master key wrapping distinct vault-item and audit keys;
//        password change re-wraps keys without breaking HMAC chain
//        verification across rotated segments.

pub mod keys;
pub mod items;
pub mod import;
pub mod auth;
pub mod header;
pub mod secret;
pub mod blind_index;
