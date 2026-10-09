use serde_json::Value;

pub struct AuditRedact;

impl AuditRedact {
    pub fn redact_secrets(record: &mut Value) {
        Self::redact_value(record);
    }

    fn redact_value(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, child) in map.iter_mut() {
                    let key_lower = key.to_ascii_lowercase();
                    if [
                        "password",
                        "secret",
                        "token",
                        "cvv",
                        "pan",
                        "pin",
                        "cookie",
                        "authorization",
                    ]
                    .iter()
                    .any(|needle| key_lower.contains(needle))
                    {
                        *child = Value::String("[redacted]".into());
                    } else {
                        Self::redact_value(child);
                    }
                }
            }
            Value::Array(items) => items.iter_mut().for_each(Self::redact_value),
            _ => {}
        }
    }
}
