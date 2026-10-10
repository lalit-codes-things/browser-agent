// Generic idempotency.
//
// The idempotency key is a double-charge / duplicate-submission protection
// mechanism for the external ecosystem, not a license for the agent runtime
// to retry. The runtime never uses a durable idempotency key to justify
// autonomous retry of an unresolved action after a crash.

use crate::core::action::commitment::ActionCommitment;
use crate::core::action::durable::IdempotencyKey;
use sha2::Digest;

pub fn idempotency_key_for(
    commitment: &ActionCommitment,
    task_id: &str,
    nonce: [u8; 16],
    expires_at_monotonic: u64,
) -> IdempotencyKey {
    let mut buf = Vec::new();
    buf.extend_from_slice(commitment.task_id.as_bytes());
    buf.push(b'\0');
    buf.extend_from_slice(task_id.as_bytes());
    buf.push(b'\0');
    buf.extend_from_slice(&commitment.canonical_for_hashing());
    buf.extend_from_slice(&nonce);
    let digest = sha2::Sha256::digest(&buf);
    let key_bytes: [u8; 32] = digest.into();
    let safe = base16_lower_hex(&key_bytes);
    IdempotencyKey::new(format!("ak-{}-{}", task_id, safe), expires_at_monotonic)
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

    #[test]
    fn idempotency_key_is_tied_to_commitment() {
        let c = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            1_000_000,
        );
        let k1 = idempotency_key_for(&c, "t-1", rand::random(), 1_100_000);
        assert!(k1.alive_at(1_050_000));
        assert!(!k1.alive_at(1_200_000));

        let c2 = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "other@merchant".into(),
            7,
            300_000,
            1_000_000,
        );
        let k2 = idempotency_key_for(&c2, "t-1", rand::random(), 1_100_000);
        assert_ne!(k1.key, k2.key, "different commitments -> different keys");
    }
}
