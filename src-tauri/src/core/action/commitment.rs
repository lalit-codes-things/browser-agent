// Generic action commitment and authorization capability.
//
// The model proposes; Policy compiles the trusted inputs; the commitment is
// the immutable facts of the operation. Authorization is bound to that
// commitment and to a state epoch, not to any reusable boolean.
//
// Invalidating conditions (must match exactly at execution time):
//   - same commitment_hash
//   - same state_epoch
//   - same authorization_nonce
//   - same operation_type
//   - not expired
//
// If the action parameters or the epoch change or the expiry passes, the
// commitment changes and the capability does not authorize the new one.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionCommitment {
    pub task_id: String,
    pub task_authority_id: Option<String>,
    pub origin_authority: String,
    pub destination: String,
    pub operation_type: OperationType,
    pub commitment_version: u32,
    pub state_epoch: u64,
    pub created_at_monotonic: u64,
    pub expires_at_monotonic: u64,
}

impl ActionCommitment {
    pub fn new(
        task_id: String,
        task_authority_id: Option<String>,
        origin_authority: String,
        destination: String,
        state_epoch: u64,
        expiry_monotonic_from_now: u64,
        now_monotonic: u64,
    ) -> Self {
        Self {
            task_id,
            task_authority_id,
            origin_authority,
            destination,
            operation_type: OperationType::ConsequentialAction,
            commitment_version: 1,
            state_epoch,
            created_at_monotonic: now_monotonic,
            expires_at_monotonic: now_monotonic.saturating_add(expiry_monotonic_from_now),
        }
    }

    pub fn canonical_for_hashing(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(self.origin_authority.as_bytes());
        v.push(b'\0');
        v.extend_from_slice(self.destination.as_bytes());
        v.push(b'\0');
        v.extend_from_slice(self.operation_type.as_bytes());
        v.push(b'\0');
        v.extend_from_slice(self.task_id.as_bytes());
        v.push(b'\0');
        if let Some(ref aid) = self.task_authority_id {
            v.extend_from_slice(aid.as_bytes());
            v.push(b'\0');
        }
        v.extend_from_slice(&self.commitment_version.to_be_bytes());
        v.push(b'\0');
        v.extend_from_slice(&self.state_epoch.to_be_bytes());
        v.push(b'\0');
        v.extend_from_slice(&self.created_at_monotonic.to_be_bytes());
        v
    }

    pub fn expires_at(&self) -> u64 {
        self.expires_at_monotonic
    }

    pub fn created_at(&self) -> u64 {
        self.created_at_monotonic
    }
}

pub fn action_commitment_hash(canonical: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(canonical).into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    ConsequentialAction,
    SensitiveDisclosure,
    AccountSecurityChange,
}

impl OperationType {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Self::ConsequentialAction => b"CONSEQUENTIAL",
            Self::SensitiveDisclosure => b"SENSITIVE_DISCLOSURE",
            Self::AccountSecurityChange => b"ACCOUNT_SECURITY_CHANGE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionAuthorizationCapability {
    pub commitment_hash: [u8; 32],
    pub state_epoch: u64,
    pub authorization_nonce: [u8; 32],
    pub operation_type: OperationType,
    pub issued_at_monotonic: u64,
    pub expires_at_monotonic: u64,
}

impl ActionAuthorizationCapability {
    pub fn new(
        commitment: &ActionCommitment,
        operation_type: OperationType,
        nonce: [u8; 32],
        now_monotonic: u64,
        expiry_monotonic_from_now: u64,
    ) -> Self {
        Self {
            commitment_hash: action_commitment_hash(&commitment.canonical_for_hashing()),
            state_epoch: commitment.state_epoch,
            authorization_nonce: nonce,
            operation_type,
            issued_at_monotonic: now_monotonic,
            expires_at_monotonic: now_monotonic.saturating_add(expiry_monotonic_from_now),
        }
    }

    pub fn authorizes_commitment(&self, commitment: &ActionCommitment) -> bool {
        self.commitment_hash == action_commitment_hash(&commitment.canonical_for_hashing())
            && self.state_epoch == commitment.state_epoch
            && !self.is_expired(commitment.created_at())
    }

    pub fn matches_nonce(&self, nonce: &[u8; 32]) -> bool {
        self.authorization_nonce == *nonce
    }

    pub fn is_expired(&self, now_monotonic: u64) -> bool {
        now_monotonic >= self.expires_at_monotonic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> u64 {
        1_000_000
    }

    #[test]
    fn commitment_hash_is_deterministic() {
        let c = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let a = action_commitment_hash(&c.canonical_for_hashing());
        let b = action_commitment_hash(&c.canonical_for_hashing());
        assert_eq!(a, b);
    }

    #[test]
    fn different_destination_produces_different_commitment() {
        let base = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let changed = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "other@merchant".into(),
            7,
            300_000,
            now(),
        );
        assert_ne!(
            action_commitment_hash(&base.canonical_for_hashing()),
            action_commitment_hash(&changed.canonical_for_hashing())
        );
    }

    #[test]
    fn authorization_is_bound_to_commitment() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let cap = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            300_000,
        );
        assert!(cap.authorizes_commitment(&commitment));
    }

    #[test]
    fn authorization_does_not_authorize_different_commitment() {
        let base = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let cap = ActionAuthorizationCapability::new(
            &base,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            300_000,
        );
        let changed = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "other@merchant".into(),
            7,
            300_000,
            now(),
        );
        assert!(!cap.authorizes_commitment(&changed));
    }

    #[test]
    fn authorization_invalidated_by_epoch_change() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let cap = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            300_000,
        );
        let changed_epoch = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            8,
            300_000,
            now(),
        );
        assert!(!cap.authorizes_commitment(&changed_epoch));
    }

    #[test]
    fn authorization_expired_after_monotonic_time() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            100,
            now(),
        );
        let cap = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            100,
        );
        let later = now() + 200;
        assert!(cap.is_expired(later));
    }

    #[test]
    fn nonce_is_unique_per_authorization() {
        let commitment = ActionCommitment::new(
            "task-1".into(),
            None,
            "native:auth".into(),
            "merchant@payee".into(),
            7,
            300_000,
            now(),
        );
        let cap_a = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            300_000,
        );
        let cap_b = ActionAuthorizationCapability::new(
            &commitment,
            OperationType::ConsequentialAction,
            rand::random(),
            now(),
            300_000,
        );
        assert_ne!(cap_a.authorization_nonce, cap_b.authorization_nonce);
    }
}
