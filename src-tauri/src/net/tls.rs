// TLS.
//
// C-76: HTTPS body/action semantics come from browser instrumentation, not
//        trusted HTTPS interception.

pub struct TlsVerifier;

impl TlsVerifier {
    pub fn verify_peer(_chain: &[String]) -> Result<bool, crate::Error> {
        Err(crate::Error::NotImplemented(
            "TlsVerifier::verify_peer is scheduled".into(),
        ))
    }
}
