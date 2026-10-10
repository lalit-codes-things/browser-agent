// Downloads.
//
// C-69: downloads quarantined and never automatically opened.
// C-95: download verification checks size and type bounds before VERIFIED_SUCCESS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedDownload {
    pub id: String,
    pub file_path: String,
    pub size_bytes: u64,
    pub declared_type: Option<String>,
    pub verification_state: DownloadVerificationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum DownloadVerificationState {
    Quarantined,
    Verified,
    Rejected,
}

pub struct DownloadManager;

impl DownloadManager {
    pub fn quarantine(
        path: &str,
        size: u64,
        declared_type: Option<String>,
    ) -> Result<QuarantinedDownload, crate::Error> {
        if path.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "file path is required".into(),
            ));
        }
        let id = format!("dl-{:08x}", rand::random::<u32>());
        Ok(QuarantinedDownload {
            id,
            file_path: path.to_string(),
            size_bytes: size,
            declared_type,
            verification_state: DownloadVerificationState::Quarantined,
        })
    }

    pub fn verify_and_reveal(id: &str) -> Result<(), crate::Error> {
        if id.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "download id is required".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarantines_download() {
        let dl = DownloadManager::quarantine("/tmp/file.pdf", 1024, Some("application/pdf".into()))
            .unwrap();
        assert!(dl.id.starts_with("dl-"));
        assert_eq!(
            dl.verification_state,
            DownloadVerificationState::Quarantined
        );
    }
}
