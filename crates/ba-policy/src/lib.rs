//! Browser-Agent trusted policy layer (Sections 13.1-13.6).
//!
//! The model proposes; it never authorizes. Every action is classified from
//! trusted runtime evidence, evaluated against the required checks, and either
//! refused, escalated to a typed user decision, or bound to a single-use
//! authorization that the executor consumes before dispatch.
//!
//! This crate performs no inference, no I/O and no clock reads of its own: the
//! caller supplies the current time, so policy behaviour is deterministic and
//! testable.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

mod approval;
mod authz;
mod capability;
mod classify;
mod egress;
mod presentation;
mod rules;

pub use approval::{
    APPROVAL_TTL_MS, ApprovalBinding, ApprovalError, ApprovalGrant, ApprovalStore, ClearCause,
    ClearedApproval, InvalidationReason, MAX_PENDING_APPROVALS, PendingApproval,
    PendingApprovalRequest,
};
pub use authz::{
    Authorization, AuthorizationBinding, AuthorizationError, AuthorizationId, AuthorizationReceipt,
    AuthorizationRequest,
};
pub use capability::{CapabilityError, CapabilityRecord, CapabilityRegistry};
pub use classify::{ClassifyInput, classify};
pub use egress::{DataEgress, EgressProvenance, EgressVerdict};
pub use presentation::{ActionPresentation, HandoffPresentation, NonSecretParameter};
pub use rules::{RuleId, RuleOutcome};

use ba_domain::{
    ActionClass, ActionKind, FrameId, ObservationId, Origin, ProfileKind, Sequence, TabId, TaskId,
    UnixMillis, ValidatedAction,
};

/// Failure modes reported by the policy engine.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    /// The action was structurally valid but not bound to a resolvable target.
    #[error("action is not bound to a grounded target")]
    UngroundedTarget,
    /// The requested origin could not be parsed.
    #[error("origin is unusable: {0}")]
    Origin(#[from] ba_domain::OriginError),
}

/// Identity of the browser context the request is evaluated in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileIdentity {
    /// Pseudonymous context identity.
    pub context: ba_domain::BrowserContextId,
    /// Whether this is the persistent normal profile or an ephemeral window.
    pub kind: ProfileKind,
}

/// Trusted runtime facts about the request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyContext {
    /// Owning task.
    pub task_id: TaskId,
    /// Active tab.
    pub tab: TabId,
    /// Frame the action targets.
    pub frame: FrameId,
    /// Exact origin of that frame, as validated by the browser layer.
    pub origin: Origin,
    /// Observation epoch the target identity belongs to.
    pub epoch: ObservationId,
    /// Browser context identity.
    pub profile: ProfileIdentity,
    /// Origins already opened or explicitly approved for this task.
    pub approved_origins: Vec<Origin>,
    /// Whether the user enabled `Only use sites I've already opened`.
    pub origin_allowlist_enabled: bool,
    /// Whether the durable audit log is healthy (BA-AUDIT-002).
    pub audit_healthy: bool,
    /// Whether the platform credential store is unlocked and available.
    pub vault_available: bool,
    /// Whether the secure-fill origin was explicitly authorized by the user.
    pub secure_fill_authorized: bool,
}

/// A policy evaluation request.
#[derive(Debug)]
pub struct PolicyRequest<'a> {
    /// Trusted runtime facts.
    pub context: PolicyContext,
    /// Action under evaluation.
    pub action: &'a ValidatedAction,
    /// Data the action would send or write.
    pub egress: Vec<DataEgress>,
    /// Capability evidence relevant to the action, when one exists.
    pub capability: Option<&'a CapabilityRecord>,
    /// Whether the user explicitly asked for this destination in the task text.
    pub destination_requested_by_user: bool,
}

/// The decision policy returns. Only [`PolicyDecision::Allow`] carries an
/// authorization, and that authorization is single-use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// The action may be dispatched once, using this authorization.
    Allow(Box<Authorization>),
    /// One typed user decision is required before dispatch.
    RequireApproval {
        /// Why approval is required, as a stable rule code.
        rule: RuleId,
        /// Structured presentation rendered by app-owned UI.
        presentation: Box<ActionPresentation>,
        /// Whether the user may approve with the agent still executing, or the
        /// action must be handed over instead.
        approvable: bool,
    },
    /// Policy refuses the action. The task enters `BLOCKED`.
    Refuse {
        /// Which rule refused it.
        rule: RuleId,
        /// Plain-language reason for the blocked-state detail view.
        reason: String,
    },
    /// The user must perform this action themselves (Section 13.4).
    HandoffRequired {
        /// Structured presentation for the handoff prompt.
        presentation: Box<HandoffPresentation>,
    },
}

/// One evaluated decision plus the trusted classification and latency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyOutcome {
    /// The decision.
    pub decision: PolicyDecision,
    /// Trusted action classification (never chosen by the model).
    pub class: ActionClass,
    /// Rules that shaped the decision, in evaluation order.
    pub rules_applied: Vec<RuleId>,
    /// Measured evaluation latency in microseconds (BA-POLICY-PERF-001).
    pub latency_us: u64,
    /// Sequence assigned to the authorization, when one was issued.
    pub authorization_sequence: Option<Sequence>,
}

/// The probability-free policy engine.
///
/// Holds no ambient state beyond the authorization/approval books, so
/// evaluation is deterministic for a given request.
#[derive(Debug)]
pub struct PolicyEngine {
    approvals: ApprovalStore,
    sequence: Sequence,
    max_authorization_ttl_ms: u64,
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEngine {
    /// Creates an engine with the default authorization TTL.
    #[must_use]
    pub fn new() -> Self {
        Self {
            approvals: ApprovalStore::new(),
            sequence: Sequence::new(1),
            max_authorization_ttl_ms: 30_000,
        }
    }

    /// Borrows the approval book.
    #[must_use]
    pub const fn approvals(&self) -> &ApprovalStore {
        &self.approvals
    }

    /// Mutably borrows the approval book.
    pub const fn approvals_mut(&mut self) -> &mut ApprovalStore {
        &mut self.approvals
    }

    fn next_sequence(&mut self) -> Sequence {
        let current = self.sequence;
        self.sequence = current.next().unwrap_or(current);
        current
    }

    /// Evaluates one action against every required policy check.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when the request itself is unusable. An unusable
    /// request is never treated as allowed.
    pub fn evaluate(
        &mut self,
        request: &PolicyRequest<'_>,
        now: UnixMillis,
    ) -> Result<PolicyOutcome, PolicyError> {
        let started = std::time::Instant::now();
        let mut rules = Vec::new();

        // 1. Private windows never run the Agent (BA-PRIV-001).
        if request.context.profile.kind == ProfileKind::Private {
            rules.push(RuleId::AgentDisabledInPrivateWindow);
            return Ok(self.finish(
                PolicyDecision::Refuse {
                    rule: RuleId::AgentDisabledInPrivateWindow,
                    reason: "The Agent is disabled in Private Windows.".to_owned(),
                },
                ActionClass::Unknown,
                rules,
                started,
            ));
        }

        // 2. Trusted classification. Hints from the model may only tighten the
        //    result, never loosen it.
        let classify_input = ClassifyInput {
            action: request.action.action(),
            capability: request.capability,
            now,
            secure_fill_authorized: request.context.secure_fill_authorized,
        };
        let class = classify(&classify_input);
        rules.push(match class {
            ActionClass::Read => RuleId::ClassifiedRead,
            ActionClass::ReversibleWrite => RuleId::ClassifiedReversibleWrite,
            ActionClass::Irreversible => RuleId::ClassifiedIrreversible,
            ActionClass::Unknown => RuleId::ClassifiedUnknown,
        });

        // 3. Egress and credential rules, which can refuse independently of the
        //    consequence class.
        for egress in &request.egress {
            match egress::evaluate(egress) {
                EgressVerdict::Permitted => {}
                EgressVerdict::RequiresApproval(rule) => {
                    rules.push(rule);
                    return Ok(self.finish(
                        PolicyDecision::RequireApproval {
                            rule,
                            presentation: Box::new(presentation::for_action(request, rule, class)),
                            approvable: true,
                        },
                        class,
                        rules,
                        started,
                    ));
                }
                EgressVerdict::Refused(rule) => {
                    rules.push(rule);
                    return Ok(self.finish(
                        PolicyDecision::Refuse {
                            rule,
                            reason: rule.plain_language().to_owned(),
                        },
                        class,
                        rules,
                        started,
                    ));
                }
            }
        }

        // 4. Navigation destination rules: credentials in URL, and the
        //    `Only use sites I've already opened` constraint.
        if request.action.kind() == ActionKind::Navigate {
            let url = match request.action.action() {
                ba_domain::Action::Navigate(params) => params.url.as_str(),
                _ => "",
            };
            if ba_domain::url_discloses_credentials(url) {
                rules.push(RuleId::CredentialsInUrl);
                return Ok(self.finish(
                    PolicyDecision::Refuse {
                        rule: RuleId::CredentialsInUrl,
                        reason: RuleId::CredentialsInUrl.plain_language().to_owned(),
                    },
                    ActionClass::Irreversible,
                    rules,
                    started,
                ));
            }
            if request.context.origin_allowlist_enabled {
                let destination = Origin::parse(url)?;
                if !request
                    .context
                    .approved_origins
                    .iter()
                    .any(|origin| origin.same_origin_exact(&destination))
                {
                    rules.push(RuleId::OriginOutsideAllowlist);
                    return Ok(self.finish(
                        PolicyDecision::RequireApproval {
                            rule: RuleId::OriginOutsideAllowlist,
                            presentation: Box::new(presentation::for_action(
                                request,
                                RuleId::OriginOutsideAllowlist,
                                class,
                            )),
                            approvable: true,
                        },
                        class,
                        rules,
                        started,
                    ));
                }
            }
        }

        // 5. Audit persistence is a prerequisite for anything that can produce
        //    an external side effect (BA-AUDIT-002).
        if !request.context.audit_healthy && class != ActionClass::Read {
            rules.push(RuleId::AuditUnavailable);
            return Ok(self.finish(
                PolicyDecision::Refuse {
                    rule: RuleId::AuditUnavailable,
                    reason: RuleId::AuditUnavailable.plain_language().to_owned(),
                },
                class,
                rules,
                started,
            ));
        }

        // 6. Credential isolation: the vault must be available and the fill must
        //    have been explicitly enabled for this exact origin.
        if request.action.kind() == ActionKind::SecureFill && !request.context.vault_available {
            rules.push(RuleId::VaultUnavailable);
            return Ok(self.finish(
                PolicyDecision::Refuse {
                    rule: RuleId::VaultUnavailable,
                    reason: RuleId::VaultUnavailable.plain_language().to_owned(),
                },
                ActionClass::Irreversible,
                rules,
                started,
            ));
        }

        if class == ActionClass::ReversibleWrite && request.capability.is_none() {
            // No capability record means no evidenced compensation path, which
            // is exactly the UNKNOWN case in Section 13.2.
            rules.push(RuleId::WeakReversibilityEvidence);
            return Ok(self.finish(
                PolicyDecision::HandoffRequired {
                    presentation: Box::new(presentation::for_handoff(
                        request,
                        RuleId::WeakReversibilityEvidence,
                        class,
                    )),
                },
                ActionClass::Unknown,
                rules,
                started,
            ));
        }

        if let Some(capability) = request.capability {
            if capability.check_valid_at(now).is_err() {
                rules.push(RuleId::CapabilityRecordExpired);
                return Ok(self.finish(
                    PolicyDecision::HandoffRequired {
                        presentation: Box::new(presentation::for_handoff(
                            request,
                            RuleId::CapabilityRecordExpired,
                            ActionClass::Unknown,
                        )),
                    },
                    ActionClass::Unknown,
                    rules,
                    started,
                ));
            }
            if !capability.matches_action(request.action.kind()) {
                rules.push(RuleId::CapabilityRecordMismatch);
                return Ok(self.finish(
                    PolicyDecision::HandoffRequired {
                        presentation: Box::new(presentation::for_handoff(
                            request,
                            RuleId::CapabilityRecordMismatch,
                            ActionClass::Unknown,
                        )),
                    },
                    ActionClass::Unknown,
                    rules,
                    started,
                ));
            }
        }

        // 7. Consequence gate: IRREVERSIBLE and UNKNOWN never execute.
        match class {
            ActionClass::Irreversible if request.action.kind() == ActionKind::SecureFill => {
                // Secure fill is executed by the trusted path only after the user
                // explicitly authorizes the exact origin and approves this fill.
                if !request.context.secure_fill_authorized {
                    rules.push(RuleId::SecureFillNotAuthorized);
                    return Ok(self.finish(
                        PolicyDecision::Refuse {
                            rule: RuleId::SecureFillNotAuthorized,
                            reason: RuleId::SecureFillNotAuthorized.plain_language().to_owned(),
                        },
                        class,
                        rules,
                        started,
                    ));
                }
                rules.push(RuleId::SecureFillRequiresApproval);
                return Ok(self.finish(
                    PolicyDecision::RequireApproval {
                        rule: RuleId::SecureFillRequiresApproval,
                        presentation: Box::new(presentation::for_action(
                            request,
                            RuleId::SecureFillRequiresApproval,
                            class,
                        )),
                        approvable: true,
                    },
                    class,
                    rules,
                    started,
                ));
            }
            ActionClass::Irreversible => {
                rules.push(RuleId::IrreversibleRequiresHandoff);
                return Ok(self.finish(
                    PolicyDecision::HandoffRequired {
                        presentation: Box::new(presentation::for_handoff(
                            request,
                            RuleId::IrreversibleRequiresHandoff,
                            class,
                        )),
                    },
                    class,
                    rules,
                    started,
                ));
            }
            ActionClass::Unknown => {
                rules.push(RuleId::UnknownRequiresHandoff);
                return Ok(self.finish(
                    PolicyDecision::HandoffRequired {
                        presentation: Box::new(presentation::for_handoff(
                            request,
                            RuleId::UnknownRequiresHandoff,
                            class,
                        )),
                    },
                    class,
                    rules,
                    started,
                ));
            }
            ActionClass::Read | ActionClass::ReversibleWrite => {}
        }

        // 8. Bounded REVERSIBLE_WRITE without an explicit user request still
        //    needs approval; the capability record proves compensation exists,
        //    not that the user wanted the change.
        if class == ActionClass::ReversibleWrite && !request.destination_requested_by_user {
            rules.push(RuleId::ReversibleWriteNeedsApproval);
            return Ok(self.finish(
                PolicyDecision::RequireApproval {
                    rule: RuleId::ReversibleWriteNeedsApproval,
                    presentation: Box::new(presentation::for_action(
                        request,
                        RuleId::ReversibleWriteNeedsApproval,
                        class,
                    )),
                    approvable: true,
                },
                class,
                rules,
                started,
            ));
        }

        // 9. Allowed: issue a single-use authorization bound to the exact
        //    action, target, frame, origin, epoch and parameters.
        let sequence = self.next_sequence();
        let expires_at_unix_ms = UnixMillis::from_millis(
            now.get()
                .saturating_add(self.max_authorization_ttl_ms as i64),
        );
        let authorization = Authorization::issue(AuthorizationRequest {
            sequence,
            task_id: request.context.task_id.clone(),
            action_id: None,
            digest: request.action.digest(),
            normalized_digest: request.action.normalized_digest(),
            tab: request.context.tab.clone(),
            frame: request.context.frame.clone(),
            origin: request.context.origin.clone(),
            epoch: request.context.epoch.clone(),
            class,
            expires_at_unix_ms,
        });
        rules.push(RuleId::Allowed);
        Ok(self.finish(
            PolicyDecision::Allow(Box::new(authorization)),
            class,
            rules,
            started,
        ))
    }

    fn finish(
        &self,
        decision: PolicyDecision,
        class: ActionClass,
        rules_applied: Vec<RuleId>,
        started: std::time::Instant,
    ) -> PolicyOutcome {
        let authorization_sequence = match &decision {
            PolicyDecision::Allow(authorization) => Some(authorization.sequence()),
            _ => None,
        };
        PolicyOutcome {
            decision,
            class,
            rules_applied,
            latency_us: u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            authorization_sequence,
        }
    }
}
