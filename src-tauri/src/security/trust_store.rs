// Trust store.
//
// Certificate-chain validation boundary. Not used for HTTPS body interception
// (C-76).

pub struct TrustStore;

impl TrustStore {
    pub fn pinned_cert_pin(_pin: &str) -> Result<bool, crate::Error> {
        Err(crate::Error::NotImplemented(
            "TrustStore::pinned_cert_pin is scheduled".into(),
        ))
    }
}
