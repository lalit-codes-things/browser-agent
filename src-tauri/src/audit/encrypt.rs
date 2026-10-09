use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};
use sha2::{Digest, Sha256};

pub struct AuditEncrypt;

impl AuditEncrypt {
    fn key(key_ref: &str) -> Result<[u8; 32], crate::Error> {
        if key_ref.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "audit key reference is required".into(),
            ));
        }
        Ok(Sha256::digest(key_ref.as_bytes()).into())
    }

    pub fn encrypt_segment(payload: &[u8], key_ref: &str) -> Result<Vec<u8>, crate::Error> {
        let key = Self::key(key_ref)?;
        let cipher = XChaCha20Poly1305::new((&key).into());
        let nonce_bytes: [u8; 24] = rand::random();
        let ciphertext = cipher
            .encrypt(XNonce::from_slice(&nonce_bytes), payload)
            .map_err(|_| crate::Error::Internal("audit encryption failed".into()))?;
        let mut blob = nonce_bytes.to_vec();
        blob.extend_from_slice(&ciphertext);
        Ok(blob)
    }

    pub fn decrypt_segment(blob: &[u8], key_ref: &str) -> Result<Vec<u8>, crate::Error> {
        if blob.len() < 24 {
            return Err(crate::Error::InvalidParameter("invalid audit blob".into()));
        }
        let key = Self::key(key_ref)?;
        let cipher = XChaCha20Poly1305::new((&key).into());
        cipher
            .decrypt(XNonce::from_slice(&blob[..24]), &blob[24..])
            .map_err(|_| crate::Error::PolicyBlocked("audit authentication failed".into()))
    }
}
