//! Browser-Agent executor (Sections 11.4, 12.3, 13.1).
//!
//! The executor performs only an action accompanied by a valid, unconsumed
//! authorization bound to that exact action, target, frame, origin and
//! observation epoch. It atomically consumes the authorization before dispatch
//! and rejects replay, expiry, parameter mutation and context change.
//!
//! The actual browser operation is performed by a [`BrowserPort`]
//! implementation: the CEF-native adapter in production, a recording port in
//! tests. When no native host is attached, [`AbsentHostPort`] reports honestly
//! that nothing was dispatched instead of simulating a page.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use ba_domain::{
    ActionClass, ObservationId, Sequence, UnixMillis, ValidatedAction, VerificationOutcome,
};
use ba_policy::{Authorization, AuthorizationBinding, AuthorizationError, AuthorizationReceipt};

/// Why a dispatch could not happen.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExecutorError {
    /// The authorization was rejected.
    #[error("dispatch refused: {0}")]
    Authorization(#[from] AuthorizationError),
    /// The trusted class is never executed by the agent.
    #[error("class {0} is never executed by the agent; it requires a user handoff")]
    ClassNotExecutable(ActionClass),
    /// The port reported that the browser refused the operation.
    #[error("the browser refused the operation: {0}")]
    BrowserRefused(String),
}

/// Result the browser port reports for one dispatch attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortOutcome {
    /// The operation reached the page.
    Dispatched {
        /// Observation epoch the page was in when input was sent.
        epoch: ObservationId,
        /// Monotonic time of dispatch.
        monotonic_ms: u64,
    },
    /// The operation never reached the page.
    NotDispatched {
        /// Why not.
        reason: DispatchBlocked,
    },
    /// Whether the side effect occurred cannot be established.
    Ambiguous {
        /// Plain-language cause, for the audit record.
        detail: String,
    },
}

/// Reasons a port reports that nothing was dispatched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchBlocked {
    /// No native browser host is attached to this process.
    HostUnavailable,
    /// The page changed after authorization.
    StaleEpoch,
    /// The target disappeared or lost its required properties.
    TargetMissing,
    /// The frame was replaced or navigated away.
    FrameReplaced,
    /// The user is in control of the browser.
    UserInControl,
    /// Policy or audit state forbids new dispatch.
    PolicyStopped,
}

impl DispatchBlocked {
    /// Returns the stable code for the audit record.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::HostUnavailable => "HOST_UNAVAILABLE",
            Self::StaleEpoch => "STALE_EPOCH",
            Self::TargetMissing => "TARGET_MISSING",
            Self::FrameReplaced => "FRAME_REPLACED",
            Self::UserInControl => "USER_IN_CONTROL",
            Self::PolicyStopped => "POLICY_STOPPED",
        }
    }

    /// Returns the plain-language explanation.
    #[must_use]
    pub const fn plain_language(&self) -> &'static str {
        match self {
            Self::HostUnavailable => {
                "The managed browser host is not running, so no action was sent to any page."
            }
            Self::StaleEpoch => {
                "The page changed after this action was approved, so it was not sent."
            }
            Self::TargetMissing => "The control this action targeted is no longer on the page.",
            Self::FrameReplaced => "The part of the page this action targeted was replaced.",
            Self::UserInControl => "You have control of the browser, so the agent did not act.",
            Self::PolicyStopped => "Recording stopped working, so new actions are paused.",
        }
    }
}

/// The browser operations the executor is allowed to request.
///
/// The trait exposes the typed action schema only: there is no generic command
/// name, script evaluation, filesystem or shell surface.
pub trait BrowserPort {
    /// Performs one already-authorized typed action.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError`] when the browser layer refuses the operation.
    fn perform(
        &mut self,
        action: &ValidatedAction,
        receipt: &AuthorizationReceipt,
    ) -> Result<PortOutcome, ExecutorError>;
}

/// The production default when no managed CEF host is attached.
///
/// It never pretends to have navigated, clicked or typed: every operation
/// reports [`DispatchBlocked::HostUnavailable`].
#[derive(Debug, Default, Clone, Copy)]
pub struct AbsentHostPort;

impl BrowserPort for AbsentHostPort {
    fn perform(
        &mut self,
        _action: &ValidatedAction,
        _receipt: &AuthorizationReceipt,
    ) -> Result<PortOutcome, ExecutorError> {
        Ok(PortOutcome::NotDispatched {
            reason: DispatchBlocked::HostUnavailable,
        })
    }
}

/// A port that records what it was asked to do, for tests.
#[derive(Debug, Default)]
pub struct RecordingPort {
    calls: Vec<String>,
    outcome: Option<PortOutcome>,
}

impl RecordingPort {
    /// Creates a port that reports [`PortOutcome::Dispatched`] for the recorded
    /// epoch.
    #[must_use]
    pub fn accepting(epoch: ObservationId, monotonic_ms: u64) -> Self {
        Self {
            calls: Vec::new(),
            outcome: Some(PortOutcome::Dispatched {
                epoch,
                monotonic_ms,
            }),
        }
    }

    /// Creates a port that always reports `outcome`.
    #[must_use]
    pub fn reporting(outcome: PortOutcome) -> Self {
        Self {
            calls: Vec::new(),
            outcome: Some(outcome),
        }
    }

    /// Returns the action kinds dispatched, in order.
    #[must_use]
    pub fn calls(&self) -> Vec<&str> {
        self.calls.iter().map(String::as_str).collect()
    }
}

impl BrowserPort for RecordingPort {
    fn perform(
        &mut self,
        action: &ValidatedAction,
        _receipt: &AuthorizationReceipt,
    ) -> Result<PortOutcome, ExecutorError> {
        self.calls.push(action.kind().code().to_owned());
        Ok(self.outcome.clone().unwrap_or(PortOutcome::NotDispatched {
            reason: DispatchBlocked::HostUnavailable,
        }))
    }
}

/// The result of one dispatch attempt, with the audit trail it produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchOutcome {
    /// Outcome reported by the port.
    pub port: PortOutcome,
    /// Evidence that the authorization was consumed exactly once.
    pub receipt: Option<AuthorizationReceipt>,
    /// Sequence of the authorizing decision.
    pub authorization_sequence: Option<Sequence>,
    /// Verification outcome implied by the port result. `None` means
    /// verification is not applicable because nothing reached the page.
    pub verification: Option<VerificationOutcome>,
}

impl DispatchOutcome {
    /// Returns whether input may have reached the page.
    #[must_use]
    pub fn may_have_reached_page(&self) -> bool {
        matches!(
            self.port,
            PortOutcome::Dispatched { .. } | PortOutcome::Ambiguous { .. }
        )
    }

    /// Returns the blocked reason, when nothing was dispatched.
    #[must_use]
    pub fn blocked(&self) -> Option<&DispatchBlocked> {
        match &self.port {
            PortOutcome::NotDispatched { reason } => Some(reason),
            _ => None,
        }
    }
}

/// Performs policy-authorized actions through a browser port.
#[derive(Debug)]
pub struct Executor<P: BrowserPort> {
    port: P,
}

impl<P: BrowserPort> Executor<P> {
    /// Creates an executor over `port`.
    pub const fn new(port: P) -> Self {
        Self { port }
    }

    /// Borrows the port.
    pub const fn port(&self) -> &P {
        &self.port
    }

    /// Mutably borrows the port.
    pub fn port_mut(&mut self) -> &mut P {
        &mut self.port
    }

    /// Consumes the authorization and performs the action.
    ///
    /// # Errors
    ///
    /// Returns [`ExecutorError`] on a rejected authorization, a class the agent
    /// may never execute, or a browser refusal. In every error case no page
    /// operation is recorded as dispatched.
    pub fn dispatch(
        &mut self,
        authorization: &mut Authorization,
        binding: &AuthorizationBinding,
        action: &ValidatedAction,
        now: UnixMillis,
    ) -> Result<DispatchOutcome, ExecutorError> {
        if !authorization.class().is_agent_executable() {
            return Err(ExecutorError::ClassNotExecutable(authorization.class()));
        }
        let receipt = authorization.consume(now, binding)?;
        let port = self.port.perform(action, &receipt)?;
        let verification = match &port {
            PortOutcome::Dispatched { .. } => None,
            PortOutcome::NotDispatched { .. } => None,
            PortOutcome::Ambiguous { .. } => Some(VerificationOutcome::Unknown),
        };
        Ok(DispatchOutcome {
            port,
            receipt: Some(receipt),
            authorization_sequence: Some(authorization.sequence()),
            verification,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ba_domain::{
        Action, ActionDigest, ActionId, BoundedText, FrameId, NavigateParams, Origin, TabId, TaskId,
    };
    use ba_policy::{AuthorizationRequest, PolicyEngine, PolicyRequest};

    fn navigate(url: &str) -> ValidatedAction {
        ValidatedAction::new(Action::Navigate(NavigateParams {
            url: BoundedText::parse(url, "navigate.url", 2_048).unwrap(),
        }))
        .unwrap()
    }

    fn binding(action: &ValidatedAction) -> AuthorizationBinding {
        AuthorizationBinding {
            task_id: TaskId::parse("task-1").unwrap(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            digest: action.digest(),
        }
    }

    fn authorization(action: &ValidatedAction, sequence: u64) -> Authorization {
        Authorization::issue(AuthorizationRequest {
            sequence: Sequence::new(sequence),
            task_id: TaskId::parse("task-1").unwrap(),
            action_id: Some(ActionId::parse("action-1").unwrap()),
            digest: action.digest(),
            normalized_digest: action.normalized_digest(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            class: ActionClass::Read,
            expires_at_unix_ms: UnixMillis::from_millis(10_000),
        })
    }

    #[test]
    fn an_absent_host_never_pretends_to_act() {
        let action = navigate("https://example.com");
        let mut executor = Executor::new(AbsentHostPort);
        let mut auth = authorization(&action, 1);
        let outcome = executor
            .dispatch(
                &mut auth,
                &binding(&action),
                &action,
                UnixMillis::from_millis(1),
            )
            .unwrap();
        assert!(!outcome.may_have_reached_page());
        assert_eq!(outcome.blocked(), Some(&DispatchBlocked::HostUnavailable));
        assert!(outcome.verification.is_none());
        assert!(auth.is_consumed());
    }

    #[test]
    fn replay_is_rejected_before_the_port_is_called() {
        let action = navigate("https://example.com");
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        let mut auth = authorization(&action, 1);
        executor
            .dispatch(
                &mut auth,
                &binding(&action),
                &action,
                UnixMillis::from_millis(1),
            )
            .unwrap();
        let replay = executor.dispatch(
            &mut auth,
            &binding(&action),
            &action,
            UnixMillis::from_millis(2),
        );
        assert!(matches!(
            replay,
            Err(ExecutorError::Authorization(
                AuthorizationError::AlreadyConsumed(_)
            ))
        ));
        assert_eq!(executor.port().calls(), vec!["NAVIGATE"]);
    }

    #[test]
    fn an_irreversible_authorization_is_never_executed() {
        let action = navigate("https://example.com");
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        let mut auth = Authorization::issue(AuthorizationRequest {
            class: ActionClass::Irreversible,
            ..auth_request(&action)
        });
        assert!(matches!(
            executor.dispatch(
                &mut auth,
                &binding(&action),
                &action,
                UnixMillis::from_millis(1)
            ),
            Err(ExecutorError::ClassNotExecutable(ActionClass::Irreversible))
        ));
        assert!(executor.port().calls().is_empty());
        assert!(!auth.is_consumed());
    }

    #[test]
    fn parameter_mutation_is_rejected() {
        let authorized = navigate("https://example.com");
        let mutated = navigate("https://evil.test");
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            5,
        ));
        let mut auth = authorization(&authorized, 1);
        assert!(matches!(
            executor.dispatch(
                &mut auth,
                &binding(&mutated),
                &mutated,
                UnixMillis::from_millis(1)
            ),
            Err(ExecutorError::Authorization(
                AuthorizationError::DigestMismatch
            ))
        ));
        assert!(executor.port().calls().is_empty());
    }

    #[test]
    fn an_ambiguous_dispatch_reports_unknown_rather_than_success() {
        let action = navigate("https://example.com");
        let mut executor = Executor::new(RecordingPort::reporting(PortOutcome::Ambiguous {
            detail: "the renderer exited after input was sent".to_owned(),
        }));
        let mut auth = authorization(&action, 3);
        let outcome = executor
            .dispatch(
                &mut auth,
                &binding(&action),
                &action,
                UnixMillis::from_millis(1),
            )
            .unwrap();
        assert!(outcome.may_have_reached_page());
        assert_eq!(outcome.verification, Some(VerificationOutcome::Unknown));
    }

    #[test]
    fn the_engine_and_executor_agree_on_the_authorization_binding() {
        let action = navigate("https://example.com");
        let mut engine = PolicyEngine::new();
        let request = PolicyRequest {
            context: ba_policy::PolicyContext {
                task_id: TaskId::parse("task-1").unwrap(),
                tab: TabId::parse("tab-1").unwrap(),
                frame: FrameId::parse("frame-1").unwrap(),
                origin: Origin::parse("https://example.com").unwrap(),
                epoch: ObservationId::parse("epoch-1").unwrap(),
                profile: ba_policy::ProfileIdentity {
                    context: ba_domain::BrowserContextId::parse("ctx-normal").unwrap(),
                    kind: ba_domain::ProfileKind::Normal,
                },
                approved_origins: Vec::new(),
                origin_allowlist_enabled: false,
                audit_healthy: true,
                vault_available: true,
                secure_fill_authorized: false,
            },
            action: &action,
            egress: Vec::new(),
            capability: None,
            destination_requested_by_user: true,
        };
        let now = UnixMillis::from_millis(1);
        let outcome = engine.evaluate(&request, now).unwrap();
        let mut authorization = match outcome.decision {
            ba_policy::PolicyDecision::Allow(authorization) => *authorization,
            other => panic!("expected an allow decision, got {other:?}"),
        };
        let mut executor = Executor::new(RecordingPort::accepting(
            ObservationId::parse("epoch-1").unwrap(),
            9,
        ));
        let dispatched = executor
            .dispatch(&mut authorization, &binding(&action), &action, now)
            .unwrap();
        assert!(dispatched.may_have_reached_page());
        assert_eq!(executor.port().calls(), vec!["NAVIGATE"]);
    }

    fn auth_request(action: &ValidatedAction) -> AuthorizationRequest {
        AuthorizationRequest {
            sequence: Sequence::new(9),
            task_id: TaskId::parse("task-1").unwrap(),
            action_id: Some(ActionId::parse("action-2").unwrap()),
            digest: action.digest(),
            normalized_digest: action.normalized_digest(),
            tab: TabId::parse("tab-1").unwrap(),
            frame: FrameId::parse("frame-1").unwrap(),
            origin: Origin::parse("https://example.com").unwrap(),
            epoch: ObservationId::parse("epoch-1").unwrap(),
            class: ActionClass::Read,
            expires_at_unix_ms: UnixMillis::from_millis(10_000),
        }
    }

    #[test]
    fn digest_helper_is_used_consistently() {
        let action = navigate("https://example.com");
        let digest: ActionDigest = action.digest();
        assert_eq!(digest.to_hex().len(), 64);
    }
}
