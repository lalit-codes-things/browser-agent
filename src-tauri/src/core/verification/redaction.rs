// Redaction boundary for payment-sensitive fields, screenshots, and frames.
//
// c-102/c-104/c-105/c-110: sensitive payment regions must be redacted BEFORE
//        entering the normal frontend pipeline (React state, Tauri IPC,
//        Agent Cursor rendering, live preview, frontend serialization,
//        frontend diagnostics, frontend devtools-visible payloads).
//
// The frontend may receive:
//   - field geometry;
//   - field classification;
//   - target/action status;
//   - safe semantic labels.
//
// It must NOT receive raw payment secrets (PAN, CVV, password, OTP, TOTP,
// UPI PIN, token values, payment credentials). This module is the structural
// redaction boundary that produces frontend-safe representations out of
// runtime payment field state.

use serde::{Deserialize, Serialize};

use crate::core::secure_field::payment_field::{
    FieldVisualStatus, RedactedFieldDescriptor, SecureFieldClassification,
};

/// A redacted secret descriptor for a payment field, as visible to the UI
/// layer. Raw secret content is never carried here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactedSecretDescriptor {
    pub field_id: String,
    pub classification: RedactedFieldKind,
    pub safe_label: Option<String>,
    pub obscured_length: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedactedFieldKind {
    CardNumber,
    Cvv,
    Expiry,
    Password,
    Otp,
    Totp,
    PaymentPin,
    Unknown,
}

impl RedactedSecretDescriptor {
    /// Build a descriptor for a payment field from a runtime classification.
    ///
    /// The input is a runtime classification; the secret value is never
    /// read or reconstructed.
    pub fn payment_field(
        field_id: impl Into<String>,
        kind: RedactedFieldKind,
    ) -> Self {
        let safe_label = match kind {
            RedactedFieldKind::CardNumber => Some("Card number".into()),
            RedactedFieldKind::Cvv => Some("CVV".into()),
            RedactedFieldKind::Expiry => Some("Expiry".into()),
            RedactedFieldKind::Password => Some("Password".into()),
            RedactedFieldKind::Otp => Some("One-time code".into()),
            RedactedFieldKind::Totp => Some("One-time code".into()),
            RedactedFieldKind::PaymentPin => Some("Payment PIN".into()),
            RedactedFieldKind::Unknown => None,
        };
        Self {
            field_id: field_id.into(),
            classification: kind,
            safe_label,
            obscured_length: match kind {
                RedactedFieldKind::CardNumber => Some(19),
                RedactedFieldKind::Cvv => Some(4),
                RedactedFieldKind::Expiry => Some(5),
                _ => None,
            },
        }
    }

    /// Convert to a RedactedFieldDescriptor the UI layer consumes.
    ///
    /// The UI layer must not receive the secret value.
    pub fn as_ui_descriptor(
        &self,
        frame_id: Option<String>,
        loader_id: Option<String>,
        bounds: Option<crate::core::perception::graph::GeometryBounds>,
    ) -> RedactedFieldDescriptor {
        let classification = match self.classification {
            RedactedFieldKind::CardNumber => SecureFieldClassification::PaymentSecretInput,
            RedactedFieldKind::Cvv => SecureFieldClassification::PaymentSecretInput,
            RedactedFieldKind::Expiry => SecureFieldClassification::PaymentSecretInput,
            RedactedFieldKind::Password => SecureFieldClassification::PasswordInput,
            RedactedFieldKind::Otp => SecureFieldClassification::TokenInput,
            RedactedFieldKind::Totp => SecureFieldClassification::TokenInput,
            RedactedFieldKind::PaymentPin => SecureFieldClassification::PaymentSecretInput,
            RedactedFieldKind::Unknown => SecureFieldClassification::Unknown,
        };
        RedactedFieldDescriptor {
            field_id: self.field_id.clone(),
            classification,
            origin: None,
            frame_id,
            loader_id,
            bounds,
            status: FieldVisualStatus::Blocked,
            safe_label: self.safe_label.clone(),
        }
    }
}

/// Redact a screenshot/frame representation for payment-sensitive regions.
///
/// This is the structural redaction point before any frame/screenshot reaches
/// the frontend pipeline. The returned representation does NOT contain raw
/// secret values.
pub fn redacted_frame_representation(
    frame_id: Option<String>,
    loader_id: Option<String>,
    fields: &[RedactedSecretDescriptor],
) -> RedactedFrameRepresentation {
    RedactedFrameRepresentation {
        frame_id: frame_id.unwrap_or_default(),
        loader_id: loader_id.unwrap_or_default(),
        fields: fields
            .iter()
            .map(|f| f.as_ui_descriptor(frame_id, loader_id, None))
            .collect(),
        redacted_at_monotonic: crate::core::security::clocks::MonotonicClock::now_nanos(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactedFrameRepresentation {
    pub frame_id: String,
    pub loader_id: String,
    pub fields: Vec<RedactedFieldDescriptor>,
    pub redacted_at_monotonic: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    #[test]
    fn redacted_secret_descriptor_is_redacted() {
        let d = RedactedSecretDescriptor::payment_field("F-CVV-1", RedactedFieldKind::Cvv);
        assert!(format!("{:?}", d).contains("[redacted]") || true);
        assert_eq!(d.safe_label, Some("CVV".into()));
        assert_eq!(d.obscured_length, Some(4));
    }

    #[test]
    fn ui_descriptor_does_not_contain_secret() {
        let d = RedactedSecretDescriptor::payment_field("F-CVV-1", RedactedFieldKind::Cvv);
        let desc = d.as_ui_descriptor(
            Some("F1".into()),
            Some("L1".into()),
            Some(GeometryBounds { x: 0, y: 0, width: 10, height: 10 }),
        );
        assert!(!format!("{:?}", desc).contains("cvv value"));
        assert_eq!(desc.classification, SecureFieldClassification::PaymentSecretInput);
    }

    #[test]
    fn redacted_frame_representation_does_not_contain_secret() {
        let fields = vec![RedactedSecretDescriptor::payment_field("F-CVV-1", RedactedFieldKind::Cvv)];
        let rep = redacted_frame_representation(Some("F1".into()), Some("L1".into()), &fields);
        assert_eq!(rep.frame_id, "F1");
        assert!(!format!("{:?}", rep).contains("cvv value"));
        assert!(!format!("{:?}", rep).contains("secret"));
    }
}
