// Generic action authorization API.
//
// This is the boundary a platform confirmation path (Touch ID / Windows Hello
// / native confirmation) would call before issuing a prompt and before
// execution consumes the resulting capability.
//
// The capability is created by the platform path under real confirmation
// wiring. Here we model the shape and expose the verification used by the
// execution path.

use crate::core::action::commitment::{
    ActionAuthorizationCapability, ActionCommitment, OperationType,
};
use crate::security::clocks::MonotonicClock;

/// Authorize an action commitment.
///
/// Returns an authorization capability bound to the exact canonical commitment
/// and operation type. This is the abstraction a platform confirmation path
/// would call before issuing a prompt.
pub fn authorize_action_commitment(
    commitment: &ActionCommitment,
    operation_type: OperationType,
) -> ActionAuthorizationCapability {
    let nonce: [u8; 32] = rand::random(); // black_box(nonce);
    ActionAuthorizationCapability::new(
        commitment,
        operation_type,
        nonce,
        MonotonicClock::now_nanos(),
        300_000,
    )
}

/// Verify that an authorization capability is still valid for a given
/// commitment, nonce, and monotonic time.
///
/// If amount/recipient/destination/operation_type/epoch changed since the
/// commitment was created, the commitment fingerprint changes and this
/// returns false.
///
/// This is the authoritative verification used by both the policy layer and
/// the execution layer. Do not duplicate this semantics elsewhere.
pub fn verify_action_authorization_capability(
    capability: &ActionAuthorizationCapability,
    commitment: &ActionCommitment,
    nonce: &[u8; 32],
    now_monotonic: u64,
) -> bool {
    capability.authorizes_commitment(commitment)
        && capability.matches_nonce(nonce)
        && !capability.is_expired(now_monotonic)
}
