// Sessions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpSession {
    pub session_id: String,
    pub target_id: String,
}

pub struct SessionManager;

impl SessionManager {
    pub fn attach(_target_id: &str) -> Result<CdpSession, crate::Error> {
        Err(crate::Error::NotImplemented(
            "SessionManager::attach is scheduled".into(),
        ))
    }
}
