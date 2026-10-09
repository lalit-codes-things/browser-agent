// Download verification.
//
// C-95: download verification checks size and type bounds before VERIFIED_SUCCESS.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadVerification {
    pub file_size_bytes: u64,
    pub declared_content_type: Option<String>,
    pub verified_size: bool,
    pub verified_type: bool,
}

pub struct DownloadChecker;

impl DownloadChecker {
    pub fn check(
        size: u64,
        max_bytes: Option<u64>,
        content_type: Option<&str>,
        allowed_types: &[&str],
    ) -> DownloadVerification {
        let verified_size = match max_bytes {
            Some(max) => size <= max,
            None => false,
        };
        let verified_type = allowed_types.iter().any(|t| Some(*t) == content_type);
        DownloadVerification {
            file_size_bytes: size,
            declared_content_type: content_type.map(|s| s.to_string()),
            verified_size,
            verified_type,
        }
    }
}
