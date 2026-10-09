// Execution authorization.
//
// c-36/c-39/c-92/c-100: an execution path must verify the authorization
// capability against the current commitment and state epoch immediately
// before acting. Authorization is bound to the canonical action commitment
// and to a state epoch; if the commitment changes (destination/operation type)
// or the epoch changes, the capability does not authorize the new action.
//
// The model knows only the typed request surface and never gains direct
// access to secrets, QR payloads, or biometric primitives.

use crate::core::action::durable::{DurableActionRecord, DurableActionState};
use crate::core::action::commitment::ActionCommitment;
use crate::core::action::authz::verify_action_authorization_capability;
use crate::core::action::ActionAuthorizationCapability;
use crate::security::clocks::MonotonicClock;

/// The result of verifying an authorization capability for a durable action
/// record and a current commitment before execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationVerificationResult {
    Authorized {
        durable_id: String,
        durable_state: DurableActionState,
    },
    /// The capability does not match the current commitment.
    CommitmentMismatch,
    /// The state epoch changed since the commitment was made.
    EpochMismatch,
    /// The authorization has expired.
    Expired,
    /// The capability cannot be verified because the durable record is
    /// unresolved (submitted-and-unresolved) after a crash. Human checkpoint
    /// + fresh verification required; no automatic retry.
    UnresolvedRequiresHumanCheckpoint,
    /// Capability is not bound to an action commitment at all (non-action or
    /// invalid state).
    NotApplicable,
}

/// Verify before execution: does the capability authorize the current
/// durable action record and current commitment at the current monotonic
/// time?
///
/// If the durable record is in a submitted-and-unresolved state (after a
/// crash / restart), the authorization verification treats it as unresolved
/// and does not authorize automated retry. The runtime must then require a
/// human checkpoint and fresh verification.
///
/// `verify_action_authorization_capability` is defined in `core/action/authz.rs` and
/// called directly so the execution layer uses the action-owned binding
/// semantics without going through a policy proxy.
pub fn verify_before_execution(
    capability: &ActionAuthorizationCapability,
    durable: &DurableActionRecord,
    commitment: &ActionCommitment,
    nonce: &[u8; 32],
) -> AuthorizationVerificationResult {
    let now_monotonic = MonotonicClock::now_nanos();

    if durable.submitted_and_unresolved() {
        return AuthorizationVerificationResult::UnresolvedRequiresHumanCheckpoint;
    }

    if !verify_action_authorization_capability(
        capability,
        commitment,
        nonce,
        now_monotonic,
    ) {
        if capability.is_expired(now_monotonic) {
            return AuthorizationVerificationResult::Expired;
        }
        if commitment.state_epoch != capability.state_epoch {
            return AuthorizationVerificationResult::EpochMismatch;
        }
        return AuthorizationVerificationResult::CommitmentMismatch;
    }

    match durable.durable_state {
        DurableActionState::Submitted
        | DurableActionState::WaitingForExternalAuth
        | DurableActionState::Processing
        | DurableActionState::Unknown => {
            AuthorizationVerificationResult::UnresolvedRequiresHumanCheckpoint
        }
        _ => AuthorizationVerificationResult::Authorized {
            durable_id: durable.durable_id.clone(),
            durable_state: durable.durable_state.clone(),
        },
    }
}

/// Attempt to execute a consequential action only after the capability has
/// been verified against the current commitment and durable record.
///
/// This is the structural execute-before-commitment check; the actual
/// external dispatch is scheduled and not faked here.
pub fn execute_action(
    capability: &ActionAuthorizationCapability,
    durable: &DurableActionRecord,
    commitment: &ActionCommitment,
    nonce: &[u8; 32],
) -> Result<(), crate::Error> {
    let result = verify_before_execution(capability, durable, commitment, nonce);
    match result {
        AuthorizationVerificationResult::Authorized { .. } => Ok(()),
        AuthorizationVerificationResult::UnresolvedRequiresHumanCheckpoint => {
            Err(crate::Error::VerificationUnknown(
                "Action unresolved after crash/restart; human checkpoint + fresh verification required"
                    .into(),
            ))
        }
        AuthorizationVerificationResult::CommitmentMismatch
        | AuthorizationVerificationResult::EpochMismatch
        | AuthorizationVerificationResult::Expired => Err(crate::Error::StateMismatch(
            "Authorization capability does not match current commitment/epoch".into(),
        )),
        AuthorizationVerificationResult::NotApplicable => Err(crate::Error::PolicyBlocked(
            "Authorization capability not applicable to this action".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_monotonic() -> u64 {
        MonotonicClock::now_nanos()
    }

    fn capability_for(
        commitment: &ActionCommitment,
        nonce: [u8; 32],
    ) -> ActionAuthorizationCapability {
        let mut cap = crate::core::action::authz::authorize_action_commitment(commitment, crate::core::action::OperationType::ConsequentialAction);
        cap.authorization_nonce = nonce;
        cap
    }

    fn durable(state: DurableActionState) -> DurableActionRecord {
        DurableActionRecord {
            durable_id: "dr-1".into(),
            task_id: "t-1".into(),
            action_commitment_hash: [0u8; 32],
            idempotency_key: crate::core::action::durable::IdempotencyKey::new(
                "ik-1".into(),
                u64::MAX,
            ),
            durable_state: state,
            origin_authority: "native:auth".into(),
            destination: "merchant@payee".into(),
            submission_epoch: 7,
            created_at_monotonic: now_monotonic(),
        }
    }

    #[test]
    fn verification_passes_for_exact_commitment_before_dispatch() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = capability_for(&commitment, nonce);
        let dur = durable(DurableActionState::Ready);
        assert_eq!(
            verify_before_execution(&cap, &dur, &commitment, &nonce),
            AuthorizationVerificationResult::Authorized {
                durable_id: "dr-1".into(),
                durable_state: DurableActionState::Ready,
            }
        );
    }

    #[test]
    fn unresolved_durable_never_authorizes() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = capability_for(&commitment, nonce);
        let dur = durable(DurableActionState::Submitted);
        assert_eq!(
            verify_before_execution(&cap, &dur, &commitment, &nonce),
            AuthorizationVerificationResult::UnresolvedRequiresHumanCheckpoint
        );
    }

    #[test]
    fn execute_fails_on_unresolved() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = capability_for(&commitment, nonce);
        let dur = durable(DurableActionState::Submitted);
        assert!(execute_action(&cap, &dur, &commitment, &nonce).is_err());
    }

    #[test]
    fn execute_succeeds_on_authorized_ready_state() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = capability_for(&commitment, nonce);
        let dur = durable(DurableActionState::Ready);
        assert!(execute_action(&cap, &dur, &commitment, &nonce).is_ok());
    }

    #[test]
    fn wrong_nonce_fails_verification() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let cap = capability_for(&commitment, rand::random());
        let dur = durable(DurableActionState::Ready);
        let wrong_nonce: [u8; 32] = rand::random();
        assert_eq!(
            verify_before_execution(&cap, &dur, &commitment, &wrong_nonce),
            AuthorizationVerificationResult::CommitmentMismatch,
        );
    }

    #[test]
    fn verify_action_authorization_capability_is_reachable_from_execution_layer() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now_monotonic(),
        );
        let nonce: [u8; 32] = rand::random();
        let cap = capability_for(&commitment, nonce);
        assert!(crate::core::action::authz::verify_action_authorization_capability(
            &cap,
            &commitment,
            &nonce,
            now_monotonic()
        ));
    }
}
