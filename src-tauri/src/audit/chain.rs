// Audit chain.
//
// C-62: audit is HMAC-chained from the first commit.
// C-96: HMAC-chained with rotated-segment linkage.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSegment {
    pub segment_id: String,
    pub sequence: u64,
    pub hmac_link: String,
    pub segment_hmac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerificationResult {
    pub verified: bool,
    pub verified_at: Option<String>,
    pub tamper_event: Option<String>,
}

pub struct AuditChain;

impl AuditChain {
    pub fn append(_segment: AuditSegment) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("AuditChain::append is scheduled".into()))
    }

    pub fn verify(_segments: &[AuditSegment]) -> ChainVerificationResult {
        ChainVerificationResult {
            verified: false,
            verified_at: None,
            tamper_event: Some("Audit chain verification is scheduled".into()),
        }
    }
}
