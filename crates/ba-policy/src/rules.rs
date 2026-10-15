//! Stable policy rule identifiers.
//!
//! Rule codes appear in the blocked-state detail view and in audit records, so
//! they are a documented, append-only vocabulary. The Agent panel shows the
//! plain-language text, never the code, and never hidden reasoning.

use core::fmt;

/// Outcome of one rule evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleOutcome {
    /// The rule permitted the action.
    Permitted,
    /// The rule requires a typed user decision.
    RequiresUser,
    /// The rule refused the action.
    Refused,
}

/// A stable policy rule identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RuleId {
    /// The request belongs to a Private Window, where the Agent is disabled.
    AgentDisabledInPrivateWindow,
    /// Trusted classification: inspection or plain navigation.
    ClassifiedRead,
    /// Trusted classification: bounded change with evidenced compensation.
    ClassifiedReversibleWrite,
    /// Trusted classification: consequential or non-recoverable effect.
    ClassifiedIrreversible,
    /// Trusted classification: effect or authority cannot be established.
    ClassifiedUnknown,
    /// The URL carries userinfo or a credential-bearing query parameter.
    CredentialsInUrl,
    /// The destination origin is outside the approved set for this task.
    OriginOutsideAllowlist,
    /// Durable audit persistence is unavailable.
    AuditUnavailable,
    /// Platform secure storage is locked or unavailable.
    VaultUnavailable,
    /// No capability record evidences a compensation path.
    WeakReversibilityEvidence,
    /// The capability record has expired.
    CapabilityRecordExpired,
    /// The capability record does not cover this operation.
    CapabilityRecordMismatch,
    /// Secure fill was not explicitly enabled for this exact origin.
    SecureFillNotAuthorized,
    /// Secure fill requires one explicit user decision.
    SecureFillRequiresApproval,
    /// A bounded change still needs the user's consent.
    ReversibleWriteNeedsApproval,
    /// The action is consequential and must be handed to the user.
    IrreversibleRequiresHandoff,
    /// The action's effect is unknown and must be handed to the user.
    UnknownRequiresHandoff,
    /// The value would leave the device to an origin the user did not ask for.
    EgressNotRequested,
    /// An origin-bound secret would be sent to a different origin.
    CredentialOriginMismatch,
    /// A raw secret value was detected in the outgoing payload.
    RawSecretEgress,
    /// Page-supplied content cannot authorize its own transmission.
    PageSuppliedEgress,
    /// The action passed every check and was authorized once.
    Allowed,
}

impl RuleId {
    /// Returns the stable machine-readable code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::AgentDisabledInPrivateWindow => "BA-PRIV-001.agent-disabled-in-private-window",
            Self::ClassifiedRead => "BA-ACTION-CLASS.read",
            Self::ClassifiedReversibleWrite => "BA-ACTION-CLASS.reversible-write",
            Self::ClassifiedIrreversible => "BA-ACTION-CLASS.irreversible",
            Self::ClassifiedUnknown => "BA-ACTION-CLASS.unknown",
            Self::CredentialsInUrl => "BA-EGRESS-001.credentials-in-url",
            Self::OriginOutsideAllowlist => "BA-ALLOWLIST-001.origin-outside-allowlist",
            Self::AuditUnavailable => "BA-AUDIT-002.audit-unavailable",
            Self::VaultUnavailable => "BA-VAULT-001.vault-unavailable",
            Self::WeakReversibilityEvidence => "BA-REV-001.weak-reversibility-evidence",
            Self::CapabilityRecordExpired => "BA-REV-001.capability-record-expired",
            Self::CapabilityRecordMismatch => "BA-REV-001.capability-record-mismatch",
            Self::SecureFillNotAuthorized => "BA-FILL-001.secure-fill-not-authorized",
            Self::SecureFillRequiresApproval => "BA-FILL-001.secure-fill-requires-approval",
            Self::ReversibleWriteNeedsApproval => "BA-POLICY-001.reversible-write-needs-approval",
            Self::IrreversibleRequiresHandoff => "BA-POLICY-001.irreversible-requires-handoff",
            Self::UnknownRequiresHandoff => "BA-POLICY-001.unknown-requires-handoff",
            Self::EgressNotRequested => "BA-EGRESS-001.egress-not-requested",
            Self::CredentialOriginMismatch => "BA-EGRESS-001.credential-origin-mismatch",
            Self::RawSecretEgress => "BA-EGRESS-001.raw-secret-egress",
            Self::PageSuppliedEgress => "BA-EGRESS-001.page-supplied-egress",
            Self::Allowed => "BA-POLICY-001.allowed",
        }
    }

    /// Returns the plain-language explanation shown to the user.
    #[must_use]
    pub const fn plain_language(self) -> &'static str {
        match self {
            Self::AgentDisabledInPrivateWindow => {
                "The Agent is unavailable in Private Windows, so this action was not sent anywhere."
            }
            Self::ClassifiedRead => {
                "This action only inspects the page or opens the page you asked for."
            }
            Self::ClassifiedReversibleWrite => {
                "This change can be undone with a tested operation, so it is reviewed once before it runs."
            }
            Self::ClassifiedIrreversible => {
                "This action cannot be undone from the app, so you perform it yourself."
            }
            Self::ClassifiedUnknown => {
                "The app cannot establish what this action would do, so it is treated as unsafe."
            }
            Self::CredentialsInUrl => {
                "The destination address carries credentials or a token in the URL. The Agent will not follow it."
            }
            Self::OriginOutsideAllowlist => {
                "You asked the Agent to use only sites you have already opened, and this site is not one of them."
            }
            Self::AuditUnavailable => {
                "Required recording failed, so new actions are stopped. Reading the page still works."
            }
            Self::VaultUnavailable => {
                "Your saved logins are locked or unavailable, so no credential was used."
            }
            Self::WeakReversibilityEvidence => {
                "There is no tested way to undo this action, so you perform it yourself."
            }
            Self::CapabilityRecordExpired => {
                "The recorded undo evidence for this action has expired, so you perform it yourself."
            }
            Self::CapabilityRecordMismatch => {
                "The recorded undo evidence does not cover this kind of action, so you perform it yourself."
            }
            Self::SecureFillNotAuthorized => {
                "You have not enabled a saved login for this exact site, so nothing was filled in."
            }
            Self::SecureFillRequiresApproval => {
                "Filling a saved login sends a secret to this site, so it needs your explicit approval."
            }
            Self::ReversibleWriteNeedsApproval => {
                "This change would alter the page, so it needs your explicit approval first."
            }
            Self::IrreversibleRequiresHandoff => {
                "This action has a lasting effect that the Agent must never perform for you."
            }
            Self::UnknownRequiresHandoff => {
                "The effect of this action cannot be established, so the Agent will not perform it."
            }
            Self::EgressNotRequested => {
                "This would send information you did not ask to send to this site."
            }
            Self::CredentialOriginMismatch => {
                "A saved login may only be used on the exact site it was saved for."
            }
            Self::RawSecretEgress => {
                "Secret-looking text was found in the outgoing value, so it was not sent."
            }
            Self::PageSuppliedEgress => {
                "This value came from the page itself, and a page cannot approve sending data."
            }
            Self::Allowed => "This action passed every required check.",
        }
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_codes_are_unique_and_namespaced() {
        let rules = [
            RuleId::AgentDisabledInPrivateWindow,
            RuleId::ClassifiedRead,
            RuleId::ClassifiedReversibleWrite,
            RuleId::ClassifiedIrreversible,
            RuleId::ClassifiedUnknown,
            RuleId::CredentialsInUrl,
            RuleId::OriginOutsideAllowlist,
            RuleId::AuditUnavailable,
            RuleId::VaultUnavailable,
            RuleId::WeakReversibilityEvidence,
            RuleId::CapabilityRecordExpired,
            RuleId::CapabilityRecordMismatch,
            RuleId::SecureFillNotAuthorized,
            RuleId::SecureFillRequiresApproval,
            RuleId::ReversibleWriteNeedsApproval,
            RuleId::IrreversibleRequiresHandoff,
            RuleId::UnknownRequiresHandoff,
            RuleId::EgressNotRequested,
            RuleId::CredentialOriginMismatch,
            RuleId::RawSecretEgress,
            RuleId::PageSuppliedEgress,
            RuleId::Allowed,
        ];
        let mut seen = std::collections::BTreeSet::new();
        for rule in rules {
            assert!(
                seen.insert(rule.code()),
                "duplicate rule code {}",
                rule.code()
            );
            assert!(rule.code().contains('.'), "code must be namespaced");
            assert!(!rule.plain_language().is_empty());
        }
    }
}
