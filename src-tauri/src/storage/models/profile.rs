// Stored profile model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProfile {
    pub id: String,
    pub kind: crate::browser::profiles::ProfileKind,
    pub storage_used_bytes: u64,
    pub expires_at: Option<String>,
}
