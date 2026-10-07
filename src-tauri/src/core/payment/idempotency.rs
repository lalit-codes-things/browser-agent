// Payment idempotency.
//
// C-11, C-39, C-100: the idempotency key is a double-charge protection
//        mechanism for the external ecosystem, not a license for the agent
//        runtime to retry. The runtime never uses a durable idempotency key
//        to justify autonomous retry of an unresolved payment.

use crate::core::payment::commitment::PaymentCommitment;
use crate::core::payment::durability::IdempotencyKey;
use rand::Rng;

/// Derive a durable idempotency key tied to the payment commitment and the
/// task context, with an expiry at which the key is not reused.
///
/// The key is stored in the durable payment record before the external
/// submission boundary is crossed. It is the external-ecosystem's guarantee
/// against duplicate submission, not the runtime's permission to retry.
pub fn idempotency_key_for(
    commitment: &PaymentCommitment,
    task_id: &str,
    nonce: [u8; 16],
    expires_at_monotonic: u64,
) -> IdempotencyKey {
    let mut buf = Vec::new();
    buf.extend_from_slice(&commitment.task_id.as_bytes());
    buf.push(b'\0');
    buf.extend_from_slice(task_id.as_bytes());
    buf.push(b'\0');
    buf.extend_from_slice(&commitment.canonical_for_hashing());
    buf.extend_from_slice(&nonce);
    let digest = sha2::Digest::digest(&buf);
    let key_bytes: [u8; 32] = digest.into();
    let safe = base16_lower_hex(&key_bytes);
    IdempotencyKey::new(format!("pk-{}-{}", task_id, safe), expires_at_monotonic)
    let safe = base16_lower_hex(&digest);
    IdempotencyKey::new(
        format!("pk-{}-{}", task_id, safe),
        expires_at_monotonic,
    )
}

fn base16_lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::payment::payment::{Payable, Currency};

    #[test]
    fn idempotency_key_is_tied_to_commitment() {
        let payable = Payable {
            origin_authority: "native:auth".into(),
            recipient: "merchant@payee".into(),
            amount_minor_units: 1000,
            currency: Currency::INR,
            payment_method: crate::core::payment::payment::PaymentMethod::Upi,
            order: None,
            mandate_scope: None,
        };
        let c = PaymentCommitment::new(
            "task-1".into(),
            None,
            payable,
            7,
            300_000,
            1_000_000,
        );
        let k1 = idempotency_key_for(&c, "t-1", rand::random(), 1_100_000);
        assert!(k1.alive_at(1_050_000));
        assert!(!k1.alive_at(1_200_000));
        let c2 = PaymentCommitment::new(
            "task-1".into(),
            None,
            Payable {
                recipient: "other@merchant".into(),
                ..payable
            },
            7,
            300_000,
            1_000_000,
        );
        let k2 = idempotency_key_for(&c2, "t-1", rand::random(), 1_100_000);
        assert_ne!(k1.key, k2.key, "different commitments -> different keys");
    }
}
