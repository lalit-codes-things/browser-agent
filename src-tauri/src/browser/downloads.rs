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
    pub fn quarantine(_path: &str, _size: u64, _declared_type: Option<String>) -> Result<QuarantinedDownload, crate::Error> {
        Err(crate::Error::NotImplemented("DownloadManager::quarantine is scheduled".into()))
    }

    pub fn verify_and_reveal(_id: &str) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("DownloadManager::verify_and_reveal is scheduled".into()))
    }
}
