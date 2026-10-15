//! The application-wide approval book (BA-APPROVAL-001).
//!
//! Exactly one approval request may be pending across the whole application. It
//! is bound to one task/action identity and expires after five minutes, earlier
//! if the target, origin, frame, parameters or observation epoch change. An
//! expired request is cleared and must be reviewed again from fresh
//! observation; it is never renewed silently.

use ba_domain::{ActionDigest, FrameId, ObservationId, Origin, TaskId, UnixMillis};

use crate::presentation::ActionPresentation;
use crate::rules::RuleId;

/// Maximum pending approvals application-wide.
pub const MAX_PENDING_APPROVALS: usize = 1;

/// Approval lifetime in milliseconds.
pub const APPROVAL_TTL_MS: u64 = 5 * 60 * 1_000;

/// Why an approval could not be created or granted.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApprovalError {
    /// Another approval is already pending application-wide.
    #[error("an approval request is already pending for action `{pending}`")]
    AlreadyPending {
        /// Action identifier of the pending request.
        pending: String,
    },
    /// No approval is pending.
    #[error("no approval request is pending")]
    NoPendingApproval,
    /// The decision does not match the pending request.
    #[error("decision does not match the pending approval")]
    ActionMismatch,
    /// The request's lifetime ended.
    #[error("the approval request expired and must be reviewed again")]
    Expired,
    /// The page or target changed while the request was pending.
    #[error("the request is no longer valid: {0}")]
    BindingDrift(&'static str),
}

/// A request to place one approval in the book.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApprovalRequest {
    /// Action awaiting approval.
    pub action_id: ba_domain::ActionId,
    /// Owning task.
    pub task_id: TaskId,
    /// Digest of the exact action parameters.
    pub digest: ActionDigest,
    /// Exact origin.
    pub origin: Origin,
    /// Frame identity.
    pub frame: FrameId,
    /// Observation epoch.
    pub epoch: ObservationId,
    /// Rule that required the approval.
    pub rule: RuleId,
    /// Structured presentation shown to the user.
    pub presentation: ActionPresentation,
}

/// One pending approval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingApproval {
    /// Action awaiting approval.
    pub action_id: ba_domain::ActionId,
    /// Owning task.
    pub task_id: TaskId,
    /// Digest of the exact action parameters.
    pub digest: ActionDigest,
    /// Exact origin.
    pub origin: Origin,
    /// Frame identity.
    pub frame: FrameId,
    /// Observation epoch.
    pub epoch: ObservationId,
    /// Rule that required the approval.
    pub rule: RuleId,
    /// Structured presentation shown to the user.
    pub presentation: ActionPresentation,
    /// When the request was created.
    pub requested_at_unix_ms: UnixMillis,
    /// When the request stops being actionable.
    pub expires_at_unix_ms: UnixMillis,
}

impl PendingApproval {
    /// Returns the remaining lifetime in milliseconds, saturating at zero.
    #[must_use]
    pub fn remaining_ms(&self, now: UnixMillis) -> u64 {
        let remaining = self.expires_at_unix_ms.get().saturating_sub(now.get());
        u64::try_from(remaining).unwrap_or(0)
    }

    /// Returns whether the request is still actionable.
    #[must_use]
    pub fn is_actionable(&self, now: UnixMillis) -> bool {
        now.get() < self.expires_at_unix_ms.get()
    }
}

/// Evidence that an approval was granted exactly once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalGrant {
    /// Approved action.
    pub action_id: ba_domain::ActionId,
    /// Digest the approval binds to.
    pub digest: ActionDigest,
    /// Exact origin.
    pub origin: Origin,
    /// Observation epoch at approval time.
    pub epoch: ObservationId,
    /// Time of the decision.
    pub granted_at_unix_ms: UnixMillis,
}

/// The context presented when a decision arrives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalBinding {
    /// Action the decision names.
    pub action_id: ba_domain::ActionId,
    /// Digest of the action as currently observed.
    pub digest: ActionDigest,
    /// Exact origin as currently observed.
    pub origin: Origin,
    /// Observation epoch as currently observed.
    pub epoch: ObservationId,
    /// Frame as currently observed.
    pub frame: FrameId,
}

/// Why a pending approval was cleared without a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationReason {
    /// The page or target changed.
    PageChanged,
    /// The user paused, took over or cancelled.
    UserStoppedAgent,
    /// The app is suspending or the system slept.
    SystemSuspend,
}

impl InvalidationReason {
    /// Returns the plain-language explanation for the audit record.
    #[must_use]
    pub const fn plain_language(self) -> &'static str {
        match self {
            Self::PageChanged => "the page or target changed while the request was pending",
            Self::UserStoppedAgent => "the requested agent control changed",
            Self::SystemSuspend => "the system suspended",
        }
    }
}

/// A cleared approval, reported so the UI can explain what happened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearedApproval {
    /// The cleared request.
    pub approval: PendingApproval,
    /// Whether it expired or was invalidated.
    pub cause: ClearCause,
}

/// Why an approval left the book.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearCause {
    /// The five-minute lifetime elapsed.
    Expired,
    /// The binding changed or the user stopped the agent.
    Invalidated(InvalidationReason),
    /// The user approved it.
    Approved,
    /// The user cancelled it.
    Cancelled,
}

/// The application-wide approval book.
#[derive(Debug, Default, Clone)]
pub struct ApprovalStore {
    pending: Option<PendingApproval>,
}

impl ApprovalStore {
    /// Creates an empty book.
    #[must_use]
    pub fn new() -> Self {
        Self { pending: None }
    }

    /// Returns the pending approval, if any.
    #[must_use]
    pub const fn pending(&self) -> Option<&PendingApproval> {
        self.pending.as_ref()
    }

    /// Returns whether an approval is pending.
    #[must_use]
    pub const fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Places a request in the book.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError::AlreadyPending`] when another request is
    /// already pending. There is no hidden queue and no automatic replacement.
    pub fn request(
        &mut self,
        request: PendingApprovalRequest,
        now: UnixMillis,
    ) -> Result<&PendingApproval, ApprovalError> {
        if let Some(existing) = &self.pending {
            return Err(ApprovalError::AlreadyPending {
                pending: existing.action_id.as_str().to_owned(),
            });
        }
        let expires_at_unix_ms = UnixMillis::from_millis(
            now.get()
                .saturating_add(i64::try_from(APPROVAL_TTL_MS).unwrap_or(APPROVAL_TTL_MS as i64)),
        );
        self.pending = Some(PendingApproval {
            action_id: request.action_id,
            task_id: request.task_id,
            digest: request.digest,
            origin: request.origin,
            frame: request.frame,
            epoch: request.epoch,
            rule: request.rule,
            presentation: request.presentation,
            requested_at_unix_ms: now,
            expires_at_unix_ms,
        });
        self.pending
            .as_ref()
            .ok_or(ApprovalError::NoPendingApproval)
    }

    /// Expires the request if its lifetime elapsed.
    pub fn expire(&mut self, now: UnixMillis) -> Option<ClearedApproval> {
        let expired = self
            .pending
            .as_ref()
            .filter(|approval| !approval.is_actionable(now))?;
        let cleared = ClearedApproval {
            approval: expired.clone(),
            cause: ClearCause::Expired,
        };
        self.pending = None;
        Some(cleared)
    }

    /// Clears the request because the world changed.
    pub fn invalidate(&mut self, reason: InvalidationReason) -> Option<ClearedApproval> {
        let approval = self.pending.take()?;
        Some(ClearedApproval {
            approval,
            cause: ClearCause::Invalidated(reason),
        })
    }

    /// Cancels the request without approving it.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError::NoPendingApproval`] when nothing is pending.
    pub fn cancel(&mut self) -> Result<ClearedApproval, ApprovalError> {
        let approval = self
            .pending
            .take()
            .ok_or(ApprovalError::NoPendingApproval)?;
        Ok(ClearedApproval {
            approval,
            cause: ClearCause::Cancelled,
        })
    }

    /// Grants the pending request for the exact action and context.
    ///
    /// # Errors
    ///
    /// Returns [`ApprovalError`] when nothing is pending, the decision names a
    /// different action, the lifetime ended, or the observation binding drifted.
    pub fn approve(
        &mut self,
        binding: &ApprovalBinding,
        now: UnixMillis,
    ) -> Result<ApprovalGrant, ApprovalError> {
        let approval = self
            .pending
            .take()
            .ok_or(ApprovalError::NoPendingApproval)?;
        if !approval.is_actionable(now) {
            return Err(ApprovalError::Expired);
        }
        if approval.action_id != binding.action_id {
            self.pending = Some(approval);
            return Err(ApprovalError::ActionMismatch);
        }
        if approval.digest != binding.digest {
            return Err(ApprovalError::BindingDrift("the action parameters changed"));
        }
        if !approval.origin.same_origin_exact(&binding.origin) {
            return Err(ApprovalError::BindingDrift("the origin changed"));
        }
        if approval.epoch != binding.epoch {
            return Err(ApprovalError::BindingDrift("the page was re-observed"));
        }
        if approval.frame != binding.frame {
            return Err(ApprovalError::BindingDrift("the frame changed"));
        }
        Ok(ApprovalGrant {
            action_id: approval.action_id,
            digest: approval.digest,
            origin: approval.origin,
            epoch: approval.epoch,
            granted_at_unix_ms: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PolicyContext;
    use crate::presentation::{ActionPresentation, for_action};
    use ba_domain::{Action, ActionId, BoundedText, NavigateParams, ValidatedAction};

    fn digest(url: &str) -> ActionDigest {
        ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse(url, "navigate.url", 2_048).unwrap(),
        }))
        .unwrap()
        .digest()
    }

    fn presentation() -> ActionPresentation {
        let action = ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse("https://example.com", "navigate.url", 2_048).unwrap(),
        }))
        .unwrap();
        let request = crate::PolicyRequest {
            context: context(),
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: false,
        };
        for_action(
            &request,
            RuleId::ReversibleWriteNeedsApproval,
            ba_domain::ActionClass::ReversibleWrite,
        )
    }

    fn context() -> PolicyContext {
        PolicyContext {
            task_id: TaskId::parse("task-1").unwrap(),
            tab: ba_domain::TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            profile: crate::ProfileIdentity {
                context: ba_domain::BrowserContextId::parse("ctx-normal").unwrap(),
                kind: ba_domain::ProfileKind::Normal,
            },
            approved_origins: Vec::new(),
            origin_allowlist_enabled: false,
            audit_healthy: true,
            vault_available: true,
            secure_fill_authorized: false,
        }
    }

    fn request(action_id: &str, url: &str) -> PendingApprovalRequest {
        PendingApprovalRequest {
            action_id: ActionId::parse(action_id).unwrap(),
            task_id: TaskId::parse("task-1").unwrap(),
            digest: digest(url),
            origin: Origin::parse("https://example.com").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            rule: RuleId::ReversibleWriteNeedsApproval,
            presentation: presentation(),
        }
    }

    fn binding(action_id: &str, url: &str) -> ApprovalBinding {
        ApprovalBinding {
            action_id: ActionId::parse(action_id).unwrap(),
            digest: digest(url),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
        }
    }

    #[test]
    fn only_one_approval_may_be_pending() {
        let mut store = ApprovalStore::new();
        let now = UnixMillis::from_millis(0);
        store
            .request(request("action-1", "https://example.com"), now)
            .unwrap();
        assert!(store.has_pending());
        assert!(matches!(
            store.request(request("action-2", "https://example.com"), now),
            Err(ApprovalError::AlreadyPending { .. })
        ));
        assert_eq!(
            store.pending().map(|p| p.action_id.as_str()),
            Some("action-1")
        );
    }

    #[test]
    fn approvals_expire_after_five_minutes() {
        let mut store = ApprovalStore::new();
        let start = UnixMillis::from_millis(1_000_000);
        store
            .request(request("action-1", "https://example.com"), start)
            .unwrap();
        let pending = store.pending().unwrap();
        assert_eq!(pending.expires_at_unix_ms.get(), start.get() + 300_000);
        assert!(
            store
                .expire(UnixMillis::from_millis(start.get() + 299_999))
                .is_none()
        );
        let cleared = store
            .expire(UnixMillis::from_millis(start.get() + 300_000))
            .unwrap();
        assert_eq!(cleared.cause, ClearCause::Expired);
        assert!(!store.has_pending());
    }

    #[test]
    fn approving_the_wrong_action_leaves_the_request_pending() {
        let mut store = ApprovalStore::new();
        let now = UnixMillis::from_millis(0);
        store
            .request(request("action-1", "https://example.com"), now)
            .unwrap();
        assert_eq!(
            store.approve(&binding("action-9", "https://example.com"), now),
            Err(ApprovalError::ActionMismatch)
        );
        assert!(store.has_pending());
    }

    #[test]
    fn binding_drift_clears_the_request_and_requires_fresh_review() {
        for drifted in [
            binding("action-1", "https://example.com/other"),
            ApprovalBinding {
                origin: Origin::parse("https://evil.test").unwrap(),
                ..binding("action-1", "https://example.com")
            },
            ApprovalBinding {
                epoch: ObservationId::parse("epoch-2").unwrap(),
                ..binding("action-1", "https://example.com")
            },
            ApprovalBinding {
                frame: FrameId::parse("frame-2").unwrap(),
                ..binding("action-1", "https://example.com")
            },
        ] {
            let mut store = ApprovalStore::new();
            let now = UnixMillis::from_millis(0);
            store
                .request(request("action-1", "https://example.com"), now)
                .unwrap();
            assert!(matches!(
                store.approve(&drifted, now),
                Err(ApprovalError::BindingDrift(_))
            ));
            assert!(!store.has_pending(), "drift must clear the request");
        }
    }

    #[test]
    fn expired_requests_are_never_approvable() {
        let mut store = ApprovalStore::new();
        let start = UnixMillis::from_millis(0);
        store
            .request(request("action-1", "https://example.com"), start)
            .unwrap();
        assert_eq!(
            store.approve(
                &binding("action-1", "https://example.com"),
                UnixMillis::from_millis(APPROVAL_TTL_MS as i64)
            ),
            Err(ApprovalError::Expired)
        );
        assert!(!store.has_pending());
    }

    #[test]
    fn a_valid_approval_grants_once() {
        let mut store = ApprovalStore::new();
        let start = UnixMillis::from_millis(500);
        store
            .request(request("action-1", "https://example.com"), start)
            .unwrap();
        let grant = store
            .approve(&binding("action-1", "https://example.com"), start)
            .unwrap();
        assert_eq!(grant.action_id.as_str(), "action-1");
        assert!(!store.has_pending());
        assert_eq!(
            store.approve(&binding("action-1", "https://example.com"), start),
            Err(ApprovalError::NoPendingApproval)
        );
    }

    #[test]
    fn invalidation_reports_the_cause() {
        let mut store = ApprovalStore::new();
        let now = UnixMillis::from_millis(0);
        store
            .request(request("action-1", "https://example.com"), now)
            .unwrap();
        let cleared = store.invalidate(InvalidationReason::PageChanged).unwrap();
        assert_eq!(
            cleared.cause,
            ClearCause::Invalidated(InvalidationReason::PageChanged)
        );
        assert!(store.pending().is_none());
    }
}
