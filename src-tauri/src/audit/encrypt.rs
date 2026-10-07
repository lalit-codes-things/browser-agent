// Audit encryption.
//
// C-96: audit is encrypted, append-only.

pub struct AuditEncrypt;

impl AuditEncrypt {
    pub fn encrypt_segment(_payload: &[u8], _key_ref: &str) -> Result<Vec<u8>, crate::Error> {
        Err(crate::Error::NotImplemented("AuditEncrypt::encrypt_segment is scheduled".into()))
    }

    pub fn decrypt_segment(_blob: &[u8], _key_ref: &str) -> Result<Vec<u8>, crate::Error> {
        Err(crate::Error::NotImplemented("AuditEncrypt::decrypt_segment is scheduled".into()))
    }
}
