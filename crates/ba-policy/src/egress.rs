//! Egress classification (BA-EGRESS-001).
//!
//! Every Agent-caused typed value, selected option, form submit, URL parameter,
//! downloaded request and external-protocol launch is an egress decision. The
//! policy layer classifies the data's provenance, its exact destination and
//! whether the user asked to disclose it. The model never decides egress alone.

use ba_domain::Origin;

use crate::rules::RuleId;

/// Where an outgoing value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressProvenance {
    /// Text typed by the user into the composer or a field they control.
    UserText,
    /// Text derived from the task instruction.
    TaskText,
    /// Content read from the page itself.
    PageSupplied,
    /// A value owned by the vault or credential store.
    VaultDerived,
    /// A value derived by the runtime from page state, for example a selection.
    AgentDerived,
}

/// One outgoing value and its destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataEgress {
    /// Exact destination origin.
    pub destination: Origin,
    /// Provenance of the value.
    pub provenance: EgressProvenance,
    /// For origin-bound values, the only origin they may go to.
    pub bound_origin: Option<Origin>,
    /// Whether the user's task explicitly asked for this destination.
    pub user_authorized: bool,
    /// Whether the trusted scanner found a raw known-secret fingerprint.
    pub contains_secret_fingerprint: bool,
}

/// The policy verdict for one egress decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EgressVerdict {
    /// The value may leave for this destination.
    Permitted,
    /// One typed user decision is required.
    RequiresApproval(RuleId),
    /// The value may not leave for this destination.
    Refused(RuleId),
}

/// Evaluates one egress decision.
#[must_use]
pub fn evaluate(egress: &DataEgress) -> EgressVerdict {
    if egress.contains_secret_fingerprint {
        return EgressVerdict::Refused(RuleId::RawSecretEgress);
    }
    match egress.provenance {
        EgressProvenance::VaultDerived => match &egress.bound_origin {
            Some(bound) if bound.same_origin_exact(&egress.destination) => EgressVerdict::Permitted,
            _ => EgressVerdict::Refused(RuleId::CredentialOriginMismatch),
        },
        EgressProvenance::PageSupplied => {
            EgressVerdict::RequiresApproval(RuleId::PageSuppliedEgress)
        }
        EgressProvenance::UserText
        | EgressProvenance::TaskText
        | EgressProvenance::AgentDerived => {
            if egress.user_authorized {
                EgressVerdict::Permitted
            } else {
                EgressVerdict::RequiresApproval(RuleId::EgressNotRequested)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::parse(value).unwrap()
    }

    fn egress(provenance: EgressProvenance) -> DataEgress {
        DataEgress {
            destination: origin("https://example.com"),
            provenance,
            bound_origin: None,
            user_authorized: false,
            contains_secret_fingerprint: false,
        }
    }

    #[test]
    fn raw_secret_fingerprints_are_refused_outright() {
        let mut decision = egress(EgressProvenance::UserText);
        decision.user_authorized = true;
        decision.contains_secret_fingerprint = true;
        assert_eq!(
            evaluate(&decision),
            EgressVerdict::Refused(RuleId::RawSecretEgress)
        );
    }

    #[test]
    fn vault_values_only_go_to_the_bound_origin() {
        let mut decision = egress(EgressProvenance::VaultDerived);
        decision.bound_origin = Some(origin("https://example.com"));
        assert_eq!(evaluate(&decision), EgressVerdict::Permitted);

        decision.destination = origin("https://example.com.evil.test");
        assert_eq!(
            evaluate(&decision),
            EgressVerdict::Refused(RuleId::CredentialOriginMismatch)
        );

        decision.destination = origin("https://sub.example.com");
        assert_eq!(
            evaluate(&decision),
            EgressVerdict::Refused(RuleId::CredentialOriginMismatch)
        );

        decision.destination = origin("https://example.com:8443");
        assert_eq!(
            evaluate(&decision),
            EgressVerdict::Refused(RuleId::CredentialOriginMismatch)
        );
    }

    #[test]
    fn page_supplied_values_always_need_a_user_decision() {
        let mut decision = egress(EgressProvenance::PageSupplied);
        decision.user_authorized = true;
        assert_eq!(
            evaluate(&decision),
            EgressVerdict::RequiresApproval(RuleId::PageSuppliedEgress)
        );
    }

    #[test]
    fn user_and_task_text_need_explicit_intent() {
        assert_eq!(
            evaluate(&egress(EgressProvenance::TaskText)),
            EgressVerdict::RequiresApproval(RuleId::EgressNotRequested)
        );
        let mut authorized = egress(EgressProvenance::UserText);
        authorized.user_authorized = true;
        assert_eq!(evaluate(&authorized), EgressVerdict::Permitted);
    }
}
