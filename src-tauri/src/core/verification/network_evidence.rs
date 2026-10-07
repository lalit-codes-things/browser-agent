// Network evidence.
//
// C-93: network evidence is one independent evidence source where applicable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvidence {
    pub request_id: Option<String>,
    pub destination: String,
    pub method: String,
    pub status_class: Option<u16>,
    pub redirect_count: Option<u32>,
}
