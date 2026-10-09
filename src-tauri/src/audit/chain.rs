use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSegment {
    pub segment_id: String,
    pub sequence: u64,
    pub hmac_link: String,
    pub segment_hmac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainVerificationStatus {
    Verified,
    Tampered { at: u64 },
    NotVerifiable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVerificationResult {
    pub status: ChainVerificationStatus,
    pub verified_at: Option<String>,
}

pub struct AuditChain;

impl AuditChain {
    pub fn append(segment: &AuditSegment, path: &Path) -> Result<(), crate::Error> {
        if segment.segment_id.trim().is_empty() || segment.segment_hmac.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "audit segment is incomplete".into(),
            ));
        }
        let line =
            serde_json::to_vec(segment).map_err(|e| crate::Error::Internal(e.to_string()))?;
        let mut bytes = line;
        bytes.push(b'\n');
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| crate::Error::Internal(e.to_string()))?;
        }
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        file.write_all(&bytes)
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        file.sync_data()
            .map_err(|e| crate::Error::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Vec<AuditSegment>, crate::Error> {
        let data =
            std::fs::read_to_string(path).map_err(|e| crate::Error::Internal(e.to_string()))?;
        data.lines()
            .map(|line| {
                serde_json::from_str(line).map_err(|e| crate::Error::Internal(e.to_string()))
            })
            .collect()
    }

    pub fn verify(segments: &[AuditSegment]) -> ChainVerificationResult {
        if segments.is_empty() {
            return ChainVerificationResult {
                status: ChainVerificationStatus::NotVerifiable,
                verified_at: None,
            };
        }
        for (index, segment) in segments.iter().enumerate() {
            if segment.sequence != index as u64 || segment.segment_hmac.is_empty() {
                return ChainVerificationResult {
                    status: ChainVerificationStatus::Tampered { at: index as u64 },
                    verified_at: None,
                };
            }
            if index > 0 && segment.hmac_link != segments[index - 1].segment_hmac {
                return ChainVerificationResult {
                    status: ChainVerificationStatus::Tampered { at: index as u64 },
                    verified_at: None,
                };
            }
        }
        ChainVerificationResult {
            status: ChainVerificationStatus::Verified,
            verified_at: Some(format!(
                "{}",
                crate::security::clocks::MonotonicClock::now_nanos()
            )),
        }
    }

    pub fn digest(payload: &[u8]) -> String {
        format!("{:x}", Sha256::digest(payload))
    }
}
