// Vault subsystem.
//
// C-148: vault key hierarchy: master password -> Argon2id KEK -> Keychain-
//        resident master key wrapping distinct vault-item and audit keys;
//        password change re-wraps keys without breaking HMAC chain
//        verification across rotated segments.

pub mod auth;
pub mod blind_index;
pub mod header;
pub mod import;
pub mod items;
pub mod keys;
pub mod secret;
pub mod service;
