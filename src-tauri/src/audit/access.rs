// Audit access.
//
// C-96: audit retained under policy and user-exportable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditAccessRecord {
    pub subject_id: String,
    pub action: AuditAccessAction,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum AuditAccessAction {
    Read,
    Export,
    Redact,
    Delete,
}
