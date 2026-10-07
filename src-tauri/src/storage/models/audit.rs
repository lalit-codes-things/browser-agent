// Stored audit record model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAuditRecord {
    pub sequence: u64,
    pub event_type: String,
    pub payload: String,
    pub hmac_link: String,
}
