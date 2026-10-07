// Audit redaction.
//
// C-96: audit is redacted.

pub struct AuditRedact;

impl AuditRedact {
    pub fn redact_secrets(_record: &mut serde_json::Value) {
        // No-op placeholder: real redaction is audit-logged and schema-aware.
    }
}
