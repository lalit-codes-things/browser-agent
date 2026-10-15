//! Single-use, fully bound authorizations (BA-POLICY-001, Section 13.3).
//!
//! An authorization is bound to the exact action digest, action type, tab,
//! frame, origin, observation epoch and an expiry. The executor consumes it
//! atomically before dispatch, so replay, expiry, parameter mutation, target
//! replacement or context change are all rejected.

use core::fmt;

use ba_domain::{
    ActionClass, ActionDigest, FrameId, IdError, ObservationId, Origin, Sequence, TabId, TaskId,
    UnixMillis,
};

/// Stable identity of one issued authorization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorizationId(String);

impl AuthorizationId {
    /// Derives the identifier from the issuing sequence.
    ///
    /// The derived text is always a valid identifier because it is a fixed
    /// prefix followed by decimal digits, so this constructor is infallible.
    #[must_use]
    pub fn from_sequence(sequence: Sequence) -> Self {
        Self(format!("authz-{}", sequence.get()))
    }

    /// Parses an identifier from a boundary payload.
    ///
    /// # Errors
    ///
    /// Returns [`IdError`] for malformed input.
    pub fn parse(value: &str) -> Result<Self, IdError> {
        if ba_domain::is_valid_id(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(IdError::InvalidCharacter)
        }
    }

    /// Borrows the identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AuthorizationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Everything an authorization is bound to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationRequest {
    /// Issuing sequence.
    pub sequence: Sequence,
    /// Owning task.
    pub task_id: TaskId,
    /// Action identity when the runtime has already allocated one.
    pub action_id: Option<ba_domain::ActionId>,
    /// Digest over the exact action parameters.
    pub digest: ActionDigest,
    /// Digest over verb and grounded target.
    pub normalized_digest: ActionDigest,
    /// Tab identity.
    pub tab: TabId,
    /// Frame identity.
    pub frame: FrameId,
    /// Exact origin.
    pub origin: Origin,
    /// Observation epoch.
    pub epoch: ObservationId,
    /// Trusted consequence class.
    pub class: ActionClass,
    /// Expiry boundary.
    pub expires_at_unix_ms: UnixMillis,
}

/// The context the executor must present when consuming an authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationBinding {
    /// Task the action belongs to.
    pub task_id: TaskId,
    /// Tab the action targets.
    pub tab: TabId,
    /// Frame the action targets.
    pub frame: FrameId,
    /// Exact origin at dispatch time.
    pub origin: Origin,
    /// Observation epoch at dispatch time.
    pub epoch: ObservationId,
    /// Digest of the action about to be dispatched.
    pub digest: ActionDigest,
}

/// Why an authorization was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AuthorizationError {
    /// The token was already used. Authorizations are single-use.
    #[error("authorization `{0}` was already consumed")]
    AlreadyConsumed(String),
    /// The token's expiry boundary passed.
    #[error("authorization `{0}` expired")]
    Expired(String),
    /// The action belongs to another task.
    #[error("authorization belongs to a different task")]
    TaskMismatch,
    /// The active tab changed.
    #[error("authorization was bound to a different tab")]
    TabMismatch,
    /// The frame changed or was replaced.
    #[error("authorization was bound to a different frame")]
    FrameMismatch,
    /// The origin changed, including scheme, subdomain or port.
    #[error("authorization was bound to a different origin")]
    OriginMismatch,
    /// The page was re-observed, invalidating target identity.
    #[error("authorization was bound to a stale observation epoch")]
    EpochMismatch,
    /// The action parameters differ from the ones that were authorized.
    #[error("authorization does not cover these action parameters")]
    DigestMismatch,
}

/// The evidence that one dispatch used an authorization exactly once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationReceipt {
    /// Identifier of the consumed authorization.
    pub authorization: AuthorizationId,
    /// Sequence of the issuing decision.
    pub sequence: Sequence,
    /// Task the dispatch belongs to.
    pub task_id: TaskId,
    /// Trusted class that was authorized.
    pub class: ActionClass,
    /// Digest of the dispatched action.
    pub digest: ActionDigest,
    /// Consumption time.
    pub consumed_at_unix_ms: UnixMillis,
}

/// A single-use authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authorization {
    id: AuthorizationId,
    sequence: Sequence,
    task_id: TaskId,
    action_id: Option<ba_domain::ActionId>,
    digest: ActionDigest,
    normalized_digest: ActionDigest,
    tab: TabId,
    frame: FrameId,
    origin: Origin,
    epoch: ObservationId,
    class: ActionClass,
    expires_at_unix_ms: UnixMillis,
    consumed: bool,
}

impl Authorization {
    /// Issues a new authorization.
    #[must_use]
    pub fn issue(request: AuthorizationRequest) -> Self {
        let id = AuthorizationId::from_sequence(request.sequence);
        Self {
            id,
            sequence: request.sequence,
            task_id: request.task_id,
            action_id: request.action_id,
            digest: request.digest,
            normalized_digest: request.normalized_digest,
            tab: request.tab,
            frame: request.frame,
            origin: request.origin,
            epoch: request.epoch,
            class: request.class,
            expires_at_unix_ms: request.expires_at_unix_ms,
            consumed: false,
        }
    }

    /// Returns the authorization identifier.
    #[must_use]
    pub fn id(&self) -> &AuthorizationId {
        &self.id
    }

    /// Returns the issuing sequence.
    #[must_use]
    pub const fn sequence(&self) -> Sequence {
        self.sequence
    }

    /// Returns the bound action digest.
    #[must_use]
    pub const fn digest(&self) -> ActionDigest {
        self.digest
    }

    /// Returns the normalized digest used by no-progress detection.
    #[must_use]
    pub const fn normalized_digest(&self) -> ActionDigest {
        self.normalized_digest
    }

    /// Returns the bound action identity, when one was allocated.
    #[must_use]
    pub const fn action_id(&self) -> Option<&ba_domain::ActionId> {
        self.action_id.as_ref()
    }

    /// Returns the trusted class that was authorized.
    #[must_use]
    pub const fn class(&self) -> ActionClass {
        self.class
    }

    /// Returns the bound origin.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Returns the bound observation epoch.
    #[must_use]
    pub const fn epoch(&self) -> &ObservationId {
        &self.epoch
    }

    /// Returns the expiry boundary.
    #[must_use]
    pub const fn expires_at_unix_ms(&self) -> UnixMillis {
        self.expires_at_unix_ms
    }

    /// Returns whether the token has been consumed.
    #[must_use]
    pub const fn is_consumed(&self) -> bool {
        self.consumed
    }

    /// Consumes the authorization for exactly one dispatch.
    ///
    /// # Errors
    ///
    /// Returns [`AuthorizationError`] on replay, expiry or any binding
    /// mismatch. Every failure leaves the token unconsumed but unusable, so the
    /// caller must fail closed rather than retry.
    pub fn consume(
        &mut self,
        now: UnixMillis,
        binding: &AuthorizationBinding,
    ) -> Result<AuthorizationReceipt, AuthorizationError> {
        if self.consumed {
            return Err(AuthorizationError::AlreadyConsumed(
                self.id.as_str().to_owned(),
            ));
        }
        if now.get() >= self.expires_at_unix_ms.get() {
            return Err(AuthorizationError::Expired(self.id.as_str().to_owned()));
        }
        if self.task_id != binding.task_id {
            return Err(AuthorizationError::TaskMismatch);
        }
        if self.tab != binding.tab {
            return Err(AuthorizationError::TabMismatch);
        }
        if self.frame != binding.frame {
            return Err(AuthorizationError::FrameMismatch);
        }
        if !self.origin.same_origin_exact(&binding.origin) {
            return Err(AuthorizationError::OriginMismatch);
        }
        if self.epoch != binding.epoch {
            return Err(AuthorizationError::EpochMismatch);
        }
        if self.digest != binding.digest {
            return Err(AuthorizationError::DigestMismatch);
        }
        self.consumed = true;
        Ok(AuthorizationReceipt {
            authorization: self.id.clone(),
            sequence: self.sequence,
            task_id: self.task_id.clone(),
            class: self.class,
            digest: self.digest,
            consumed_at_unix_ms: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::{Action, BoundedText, NavigateParams, ValidatedAction};

    fn digest(url: &str) -> ActionDigest {
        ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse(url, "navigate.url", 2_048).unwrap(),
        }))
        .unwrap()
        .digest()
    }

    fn id(value: &str) -> TabId {
        TabId::parse(value).unwrap()
    }

    fn binding(url: &str) -> AuthorizationBinding {
        AuthorizationBinding {
            task_id: TaskId::parse("task-1").unwrap(),
            tab: id("tab-1"),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            digest: digest(url),
        }
    }

    fn issued() -> Authorization {
        Authorization::issue(AuthorizationRequest {
            sequence: Sequence::new(7),
            task_id: TaskId::parse("task-1").unwrap(),
            action_id: Some(ba_domain::ActionId::parse("action-1").unwrap()),
            digest: digest("https://example.com"),
            normalized_digest: digest("https://example.com"),
            tab: id("tab-1"),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            class: ActionClass::Read,
            expires_at_unix_ms: UnixMillis::from_millis(1_000),
        })
    }

    #[test]
    fn identifier_is_derived_from_the_sequence() {
        assert_eq!(
            AuthorizationId::from_sequence(Sequence::new(12)).as_str(),
            "authz-12"
        );
        assert!(AuthorizationId::parse("authz-12").is_ok());
        assert!(AuthorizationId::parse("../authz").is_err());
    }

    #[test]
    fn consumes_exactly_once() {
        let mut authorization = issued();
        let receipt = authorization
            .consume(
                UnixMillis::from_millis(500),
                &binding("https://example.com"),
            )
            .unwrap();
        assert_eq!(receipt.sequence, Sequence::new(7));
        assert!(authorization.is_consumed());
        assert!(matches!(
            authorization.consume(
                UnixMillis::from_millis(500),
                &binding("https://example.com")
            ),
            Err(AuthorizationError::AlreadyConsumed(_))
        ));
    }

    #[test]
    fn rejects_expired_tokens() {
        let mut authorization = issued();
        assert!(matches!(
            authorization.consume(
                UnixMillis::from_millis(1_000),
                &binding("https://example.com")
            ),
            Err(AuthorizationError::Expired(_))
        ));
    }

    #[test]
    fn rejects_every_binding_mismatch() {
        let cases: Vec<(AuthorizationBinding, AuthorizationError)> = vec![
            (
                AuthorizationBinding {
                    origin: Origin::parse("https://www.example.com").unwrap(),
                    ..binding("https://example.com")
                },
                AuthorizationError::OriginMismatch,
            ),
            (
                AuthorizationBinding {
                    origin: Origin::parse("https://example.com:8443").unwrap(),
                    ..binding("https://example.com")
                },
                AuthorizationError::OriginMismatch,
            ),
            (
                AuthorizationBinding {
                    tab: id("tab-2"),
                    ..binding("https://example.com")
                },
                AuthorizationError::TabMismatch,
            ),
            (
                AuthorizationBinding {
                    frame: FrameId::parse("frame-2").unwrap(),
                    ..binding("https://example.com")
                },
                AuthorizationError::FrameMismatch,
            ),
            (
                AuthorizationBinding {
                    epoch: ObservationId::parse("epoch-2").unwrap(),
                    ..binding("https://example.com")
                },
                AuthorizationError::EpochMismatch,
            ),
            (
                AuthorizationBinding {
                    task_id: TaskId::parse("task-2").unwrap(),
                    ..binding("https://example.com")
                },
                AuthorizationError::TaskMismatch,
            ),
            (
                binding("https://example.com/other"),
                AuthorizationError::DigestMismatch,
            ),
        ];
        for (binding, expected) in cases {
            let mut authorization = issued();
            assert_eq!(
                authorization.consume(UnixMillis::from_millis(100), &binding),
                Err(expected),
                "binding {binding:?} must be rejected"
            );
            assert!(!authorization.is_consumed());
        }
    }
}
