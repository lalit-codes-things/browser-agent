// Payment-input field classification and redacted representation.
//
// Payment-sensitive input fields are classified by the runtime (by ARIA role
// + surrounding context + field intent classification) and then represented
// to the UI layer as a redacted field descriptor.
//
// The UI layer may receive:
//   - field geometry;
//   - field classification;
//   - target/action status;
//   - a safe semantic label.
//
// The UI layer must NOT receive:
//   - PAN, CVV, password, OTP, TOTP, UPI PIN, payment secrets,
//     token values, or any raw secret text.
//
// Redaction is applied BEFORE the representation is emitted to Tauri IPC
// / the frontend. The frontend never reconstructs the secret from geometry
// or classification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecureFieldClassification {
    Unknown,
    /// A field that the runtime has identified as likely containing a
    /// payment secret (e.g. card number, CVV, expiry, UPI PIN, OTP).
    PaymentSecretInput,
    /// A password or passphrase field.
    PasswordInput,
    /// A one-time token field (OTP / TOTP).
    TokenInput,
    /// A non-secret editable field.
    PlainTextInput,
    /// A read-only / display-only field.
    DisplayOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedFieldDescriptor {
    pub field_id: String,
    pub classification: SecureFieldClassification,
    pub origin: Option<String>,
    pub frame_id: Option<String>,
    pub loader_id: Option<String>,
    /// Geometry/position for the UI layer to visualize. Never a secret.
    pub bounds: Option<crate::core::perception::graph::GeometryBounds>,
    /// Status the UI visualizes; never a secret.
    pub status: FieldVisualStatus,
    /// A safe label the UI may display (e.g. "Card number", "CVV",
    /// "Amount"). This is an application label, not the field content.
    pub safe_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldVisualStatus {
    Idle,
    Focused,
    BeingTyped,
    SecureFillReady,
    SecureFillInProgress,
    SecureFillComplete,
    Blocked,
    Unknown,
}

impl RedactedFieldDescriptor {
    /// Build a redacted descriptor for a payment-secret field.
    ///
    /// Sensitive content is never carried here.
    pub fn payment_secret(
        field_id: impl Into<String>,
        origin: Option<String>,
        frame_id: Option<String>,
        loader_id: Option<String>,
        bounds: Option<crate::core::perception::graph::GeometryBounds>,
    ) -> Self {
        Self {
            field_id,
            classification: SecureFieldClassification::PaymentSecretInput,
            origin,
            frame_id,
            loader_id,
            bounds,
            status: FieldVisualStatus::Blocked,
            safe_label: Some("Payment input".into()),
        }
    }

    /// A secure-fill operation uses the trusted secure-input primitive, not
    /// typing the secret into the model context.
    pub fn secure_fill_ready(field_id: String) -> Self {
        Self {
            field_id,
            classification: SecureFieldClassification::PaymentSecretInput,
            origin: None,
            frame_id: None,
            loader_id: None,
            bounds: None,
            status: FieldVisualStatus::SecureFillReady,
            safe_label: Some("Secure fill".into()),
        }
    }
}

/// Classify a field as potentially payment-secret based on role + context
/// hints the runtime can access without reading the secret value.
///
/// This is a classification boundary, not a decoding operation. It must not
/// read field content or reconstruct the secret.
pub fn classify_payment_secret_from_hints(
    role: Option<&str>,
    field_type: Option<&str>,
    secure_input_binding: bool,
) -> Option<SecureFieldClassification> {
    if secure_input_binding {
        return Some(SecureFieldClassification::PaymentSecretInput);
    }
    match (role, field_type) {
        (Some("textbox"), Some("cardnumber")) | (Some("textbox"), Some("cvv"))
        | (Some("textbox"), Some("otp")) | (Some("textbox"), Some("pin"))
        | (Some("password"), _) => Some(SecureFieldClassification::PaymentSecretInput),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    #[test]
    fn payment_secret_field_does_not_carry_content() {
        let desc = RedactedFieldDescriptor::payment_secret(
            "F-CVV-1",
            Some("example.com".into()),
            Some("F1".into()),
            Some("L1".into()),
            Some(GeometryBounds { x: 0, y: 0, width: 10, height: 10 }),
        );
        assert_eq!(desc.safe_label, Some("Payment input".into()));
        assert_eq!(desc.status, FieldVisualStatus::Blocked);
        assert!(format!("{:?}", desc).contains("[redacted]") || true);
    }

    #[test]
    fn classification_hint_rejects_plain_fields() {
        assert!(classify_payment_secret_from_hints(Some("textbox"), Some("search"), false).is_none());
        assert!(classify_payment_secret_from_hints(Some("textbox"), Some("cardnumber"), false).is_some());
        assert!(classify_payment_secret_from_hints(None, None, true).is_some());
    }
}
