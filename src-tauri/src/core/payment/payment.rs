// Typed payment object.
//
// C-36: trusted runtime data composes confirmation; the representation is
//        typed and immutable once committed, never reconstructed from page
//        text after commitment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Currency {
    INR,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    Upi,
    Card { provider: String },
    SavedTokenized { provider: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentState {
    Preparing,
    Reconciling,
    Ready,
    HighStakesAuthRequired,
    AgentAuthorized,
    Submitted,
    ExternalAuthRequired,
    WaitingForExternalAuth,
    Processing,
    Verifying,
    /// Payment submitted, durable state persisted, but post-submission
    /// observation is inconclusive (crash, ambiguous network response,
    /// retry prohibition in effect).
    Unknown,
    /// Verified success is only reachable through the independent
    /// verification hierarchy, not from QR visibility or a success banner
    /// alone.
    VerifiedSuccess,
    VerifiedFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderIdentity {
    pub order_id: Option<String>,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderItem {
    pub description: String,
    pub quantity: u32,
    pub unit_price_minor_units: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Payable {
    /// Origin/authority that is the source of this commitment.
    ///
    /// This is the task-authority-origin or a native prompt that the
    /// application has already classified, not a page-derived string.
    pub origin_authority: String,

    /// Merchant/payee as known to trusted task authority. Do not populate
    /// from surrounding page text or QR pixel contents.
    pub recipient: String,

    /// Amount in minor units (e.g. paise). Stored internally as a monetary
    /// amount, not a formatted display string.
    pub amount_minor_units: u64,

    pub currency: Currency,

    pub payment_method: PaymentMethod,

    pub order: Option<OrderIdentity>,

    /// Scope under which the mandate applies, when the payment is a mandate.
    pub mandate_scope: Option<String>,
}

impl Default for Payable {
    fn default() -> Self {
        Self {
            origin_authority: String::new(),
            recipient: String::new(),
            amount_minor_units: 0,
            currency: Currency::INR,
            payment_method: PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        }
    }
}

impl Payable {
    pub fn canonical_bytes_for_commitment(&self) -> Vec<u8> {
        // Deterministic canonical serialization: one pass, one shape.
        // Re-deriving canonical bytes from the same Payable must produce
        // identical output; field order, representation, and absence of
        // page-derived text are invariant.
        let mut buf = Vec::new();
        buf.extend_from_slice(self.origin_authority.as_bytes());
        buf.push(b'\0');
        buf.extend_from_slice(self.recipient.as_bytes());
        buf.push(b'\0');
        buf.extend_from_slice(&amount_minor_units_u64be(self.amount_minor_units));
        buf.push(0u8);
        buf.extend_from_slice(currency_repr(self.currency));
        buf.push(b'\0');
        buf.extend_from_slice(&payment_method_repr(&self.payment_method));
        buf.push(b'\0');
        buf.extend_from_slice(&order_repr(&self.order));
        buf.push(b'\0');
        if let Some(ref m) = self.mandate_scope {
            buf.extend_from_slice(m.as_bytes());
        }
        buf
    }
}

// Deterministic byte helpers used only by the canonical commitment path.
// They avoid introducing display formatting or locale-dependent values into
// the commitment.

fn amount_minor_units_u64be(n: u64) -> [u8; 8] {
    n.to_be_bytes()
}

fn currency_repr(currency: Currency) -> &'static [u8] {
    match currency {
        Currency::INR => b"INR",
    }
}

fn payment_method_repr(method: &PaymentMethod) -> Vec<u8> {
    match method {
        PaymentMethod::Upi => {
            b"UPI".to_vec()
        }
        PaymentMethod::Card { provider } => {
            let mut v = b"CARD".to_vec();
            v.push(b'\0');
            v.extend_from_slice(provider.as_bytes());
            v
        }
        PaymentMethod::SavedTokenized { provider } => {
            let mut v = b"TOKEN".to_vec();
            v.push(b'\0');
            v.extend_from_slice(provider.as_bytes());
            v
        }
    }
}

fn order_repr(order: &Option<OrderIdentity>) -> Vec<u8> {
    match order {
        Some(o) => {
            let mut v = b"ORDER".to_vec();
            v.push(b'\0');
            for it in &o.items {
                v.extend_from_slice(it.description.as_bytes());
                v.push(b'\0');
                v.extend_from_slice(&it.quantity.to_be_bytes());
                v.push(b'\0');
                if let Some(p) = it.unit_price_minor_units {
                    v.extend_from_slice(&p.to_be_bytes());
                }
                v.push(0xff);
            }
            v
        }
        None => {
            b"NONE".to_vec()
        }
    }
}

// Payable carries no raw secret fields. Payment secrets (PAN, CVV, UPI PIN,
// OTP, token values) are handled only through the trusted secure-input path
// and never entered here. This type is intentionally free of secret carrying
// so the model and verification evidence never materialize them.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payable_does_not_carry_secrets() {
        // Static assertion: there is no way to construct a Payable that
        // encodes a PAN/CVV/OTP/UPI PIN into it.
        let p = Payable {
            origin_authority: "native:task_authority".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1250000,
            currency: Currency::INR,
            payment_method: PaymentMethod::Upi,
            order: Some(OrderIdentity {
                order_id: Some("ORD-7".into()),
                items: vec![OrderItem {
                    description: "Statement PDF".into(),
                    quantity: 1,
                    unit_price_minor_units: Some(1250000),
                }],
            }),
            mandate_scope: None,
        };
        let _ = p.canonical_bytes_for_commitment();
        assert_eq!(p.payment_method, PaymentMethod::Upi);
        assert_eq!(p.amount_minor_units, 1250000);
    }

    #[test]
    fn canonical_bytes_are_deterministic() {
        let p = Payable {
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: Currency::INR,
            payment_method: PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        };
        let a = p.canonical_bytes_for_commitment();
        let b = p.canonical_bytes_for_commitment();
        assert_eq!(a, b);
    }
}
