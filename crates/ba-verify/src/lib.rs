//! Browser-Agent independent verification (Section 12.7, BA-VERIFY-001).
//!
//! Verification uses evidence distinct from the model's assertion that it
//! succeeded. A click being issued, a button animation or a tool returning
//! without error are not proof. When evidence is insufficient, stale or
//! contradictory, the outcome is `UNKNOWN`, and only `VERIFIED_SUCCESS` is
//! reported as a verified success.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use std::collections::BTreeMap;

use ba_domain::{Origin, VerificationOutcome};

/// Download completion metadata for verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadEvidence {
    /// CEF reported the download completing.
    pub completion_reported: bool,
    /// The final file exists at the expected destination.
    pub file_exists: bool,
    /// The file name that was written.
    pub file_name: String,
    /// Observed size in bytes.
    pub size_bytes: u64,
    /// Whether the size is plausible for the expected content type.
    pub size_plausible: bool,
}

/// Trusted browser evidence gathered after the action.
///
/// `fresh` is the observation epoch guard: evidence collected before the action
/// or before a navigation cannot verify it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Evidence {
    /// Whether the evidence was collected after the action in the current epoch.
    pub fresh: bool,
    /// Origin currently shown in the active tab.
    pub observed_origin: Option<Origin>,
    /// Observed non-secret field values, keyed by field label.
    pub observed_fields: BTreeMap<String, String>,
    /// Observed text of the relevant element or page region.
    pub observed_text: Option<String>,
    /// Download metadata, when the action initiated a download.
    pub download: Option<DownloadEvidence>,
    /// Value read back after a setting change.
    pub setting_value: Option<String>,
    /// Whether the session is independently observed as authenticated.
    pub authenticated: Option<bool>,
    /// Sources of evidence that disagree with each other.
    pub contradictions: Vec<String>,
}

/// A task-specific condition that must hold for success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Postcondition {
    /// The active tab shows this exact origin.
    UrlOrigin(Origin),
    /// A non-secret field contains the expected value.
    FieldContent {
        /// Field label.
        field: String,
        /// Expected value.
        expected: String,
    },
    /// The relevant region contains the expected text.
    TextContains {
        /// Expected substring.
        expected: String,
    },
    /// The named download completed and exists.
    DownloadCompleted {
        /// Expected file name.
        file_name: String,
    },
    /// A setting reads back as the expected value.
    SettingReadback {
        /// Expected value.
        expected: String,
    },
    /// The session is authenticated, established by evidence other than the
    /// page's own success message.
    Authenticated,
}

/// Result of evaluating one postcondition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionStatus {
    /// The condition holds.
    Satisfied,
    /// Evidence establishes that the condition does not hold.
    Unmet,
    /// Evidence cannot establish either outcome.
    Unavailable,
}

/// A completed verification evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    /// Outcome vocabulary value.
    pub outcome: VerificationOutcome,
    /// Plain-language condition results, in evaluation order.
    pub conditions: Vec<(String, ConditionStatus)>,
    /// Evidence that contradicts the intended result.
    pub contradictions: Vec<String>,
}

impl VerificationReport {
    /// Returns the user-facing label for this report.
    #[must_use]
    pub fn label(&self) -> &'static str {
        ba_domain::verification_label(self.outcome)
    }

    /// Returns the explanatory sentence for this report.
    #[must_use]
    pub fn meaning(&self) -> &'static str {
        ba_domain::verification_meaning(self.outcome)
    }

    /// Returns the conditions that could not be established.
    #[must_use]
    pub fn unavailable_conditions(&self) -> Vec<&str> {
        self.conditions
            .iter()
            .filter(|(_, status)| *status == ConditionStatus::Unavailable)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Returns the evidence hashes recorded for the audit trail.
    #[must_use]
    pub fn evidence_references(&self) -> Vec<String> {
        self.conditions
            .iter()
            .map(|(name, status)| format!("{name}:{}", describe_status(*status)))
            .collect()
    }
}

/// Evaluates a set of postconditions against evidence.
///
/// Returns `UNKNOWN` whenever evidence is stale, missing or contradictory, and
/// never reports `VERIFIED_SUCCESS` without every mandatory condition satisfied
/// by fresh evidence.
#[must_use]
pub fn evaluate(postconditions: &[Postcondition], evidence: &Evidence) -> VerificationReport {
    if !evidence.fresh {
        return unknown_report(
            postconditions,
            "the browser state was not re-observed after the action",
        );
    }
    if !evidence.contradictions.is_empty() {
        return VerificationReport {
            outcome: VerificationOutcome::Unknown,
            conditions: postconditions
                .iter()
                .map(|condition| (describe(condition), ConditionStatus::Unavailable))
                .collect(),
            contradictions: evidence.contradictions.clone(),
        };
    }

    let evaluated: Vec<(String, ConditionStatus)> = postconditions
        .iter()
        .map(|condition| (describe(condition), assess(condition, evidence)))
        .collect();

    let satisfied = evaluated
        .iter()
        .filter(|(_, status)| *status == ConditionStatus::Satisfied)
        .count();
    let unmet = evaluated
        .iter()
        .filter(|(_, status)| *status == ConditionStatus::Unmet)
        .count();
    let unavailable = evaluated
        .iter()
        .filter(|(_, status)| *status == ConditionStatus::Unavailable)
        .count();

    let outcome = if postconditions.is_empty() {
        VerificationOutcome::Unknown
    } else if satisfied == postconditions.len() {
        VerificationOutcome::VerifiedSuccess
    } else if unmet > 0 && unavailable == 0 {
        VerificationOutcome::VerifiedFailure
    } else if unmet > 0 {
        VerificationOutcome::LikelyFailure
    } else if satisfied > 0 {
        VerificationOutcome::LikelySuccess
    } else {
        VerificationOutcome::Unknown
    };

    VerificationReport {
        outcome,
        conditions: evaluated,
        contradictions: Vec::new(),
    }
}

fn assess(condition: &Postcondition, evidence: &Evidence) -> ConditionStatus {
    match condition {
        Postcondition::UrlOrigin(expected) => match &evidence.observed_origin {
            Some(observed) if observed.same_origin_exact(expected) => ConditionStatus::Satisfied,
            Some(_) => ConditionStatus::Unmet,
            None => ConditionStatus::Unavailable,
        },
        Postcondition::FieldContent { field, expected } => {
            match evidence.observed_fields.get(field) {
                Some(observed) if observed == expected => ConditionStatus::Satisfied,
                Some(_) => ConditionStatus::Unmet,
                None => ConditionStatus::Unavailable,
            }
        }
        Postcondition::TextContains { expected } => match &evidence.observed_text {
            Some(observed) if observed.contains(expected) => ConditionStatus::Satisfied,
            Some(_) => ConditionStatus::Unmet,
            None => ConditionStatus::Unavailable,
        },
        Postcondition::DownloadCompleted { file_name } => match &evidence.download {
            Some(download)
                if download.completion_reported
                    && download.file_exists
                    && download.size_plausible
                    && download.file_name == *file_name =>
            {
                ConditionStatus::Satisfied
            }
            Some(download)
                if download.completion_reported
                    && (!download.file_exists || !download.size_plausible) =>
            {
                ConditionStatus::Unmet
            }
            _ => ConditionStatus::Unavailable,
        },
        Postcondition::SettingReadback { expected } => match &evidence.setting_value {
            Some(observed) if observed == expected => ConditionStatus::Satisfied,
            Some(_) => ConditionStatus::Unmet,
            None => ConditionStatus::Unavailable,
        },
        Postcondition::Authenticated => match evidence.authenticated {
            Some(true) => ConditionStatus::Satisfied,
            Some(false) => ConditionStatus::Unmet,
            None => ConditionStatus::Unavailable,
        },
    }
}

fn unknown_report(postconditions: &[Postcondition], cause: &str) -> VerificationReport {
    VerificationReport {
        outcome: VerificationOutcome::Unknown,
        conditions: postconditions
            .iter()
            .map(|condition| (describe(condition), ConditionStatus::Unavailable))
            .collect(),
        contradictions: vec![cause.to_owned()],
    }
}

/// Returns a plain-language description of a postcondition.
#[must_use]
pub fn describe(condition: &Postcondition) -> String {
    match condition {
        Postcondition::UrlOrigin(origin) => format!("the page is on {}", origin.canonical()),
        Postcondition::FieldContent { field, .. } => {
            format!("the field \u{201c}{field}\u{201d} contains the intended value")
        }
        Postcondition::TextContains { expected } => {
            format!("the page shows \u{201c}{expected}\u{201d}")
        }
        Postcondition::DownloadCompleted { file_name } => {
            format!("the file \u{201c}{file_name}\u{201d} was saved")
        }
        Postcondition::SettingReadback { expected } => {
            format!("the setting reads back as \u{201c}{expected}\u{201d}")
        }
        Postcondition::Authenticated => "the session is signed in".to_owned(),
    }
}

const fn describe_status(status: ConditionStatus) -> &'static str {
    match status {
        ConditionStatus::Satisfied => "satisfied",
        ConditionStatus::Unmet => "unmet",
        ConditionStatus::Unavailable => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin(value: &str) -> Origin {
        Origin::parse(value).unwrap()
    }

    fn fresh() -> Evidence {
        Evidence {
            fresh: true,
            ..Evidence::default()
        }
    }

    #[test]
    fn all_conditions_satisfied_is_verified_success() {
        let mut evidence = fresh();
        evidence.observed_origin = Some(origin("https://example.com"));
        evidence.observed_text = Some("Order confirmed".to_owned());
        let report = evaluate(
            &[
                Postcondition::UrlOrigin(origin("https://example.com")),
                Postcondition::TextContains {
                    expected: "Order confirmed".to_owned(),
                },
            ],
            &evidence,
        );
        assert_eq!(report.outcome, VerificationOutcome::VerifiedSuccess);
        assert!(report.outcome.is_verified_success());
        assert_eq!(report.label(), "Verified success");
    }

    #[test]
    fn stale_evidence_is_never_success() {
        let evidence = Evidence {
            observed_text: Some("Order confirmed".to_owned()),
            ..Evidence::default()
        };
        let report = evaluate(
            &[Postcondition::TextContains {
                expected: "Order confirmed".to_owned(),
            }],
            &evidence,
        );
        assert_eq!(report.outcome, VerificationOutcome::Unknown);
        assert!(report.meaning().contains("insufficient"));
    }

    #[test]
    fn contradictory_evidence_is_unknown() {
        let mut evidence = fresh();
        evidence.observed_origin = Some(origin("https://example.com"));
        evidence.contradictions =
            vec!["the confirmation banner and the record disagree".to_owned()];
        let report = evaluate(
            &[Postcondition::UrlOrigin(origin("https://example.com"))],
            &evidence,
        );
        assert_eq!(report.outcome, VerificationOutcome::Unknown);
        assert_eq!(report.contradictions.len(), 1);
    }

    #[test]
    fn definitive_mismatch_is_verified_failure() {
        let mut evidence = fresh();
        evidence.observed_origin = Some(origin("https://elsewhere.test"));
        let report = evaluate(
            &[Postcondition::UrlOrigin(origin("https://example.com"))],
            &evidence,
        );
        assert_eq!(report.outcome, VerificationOutcome::VerifiedFailure);
    }

    #[test]
    fn partial_evidence_is_likely_success_and_never_green() {
        let mut evidence = fresh();
        evidence.observed_origin = Some(origin("https://example.com"));
        let report = evaluate(
            &[
                Postcondition::UrlOrigin(origin("https://example.com")),
                Postcondition::TextContains {
                    expected: "Order confirmed".to_owned(),
                },
            ],
            &evidence,
        );
        assert_eq!(report.outcome, VerificationOutcome::LikelySuccess);
        assert!(!report.outcome.is_verified_success());
        assert_eq!(
            report.label(),
            "Likely succeeded \u{2014} not fully verified"
        );
        assert_eq!(report.unavailable_conditions().len(), 1);
    }

    #[test]
    fn no_evidence_at_all_is_unknown() {
        let report = evaluate(
            &[Postcondition::SettingReadback {
                expected: "Dark".to_owned(),
            }],
            &fresh(),
        );
        assert_eq!(report.outcome, VerificationOutcome::Unknown);
    }

    #[test]
    fn empty_postconditions_never_report_success() {
        let report = evaluate(&[], &fresh());
        assert_eq!(report.outcome, VerificationOutcome::Unknown);
    }

    #[test]
    fn downloads_need_completion_plus_a_plausible_file() {
        let mut evidence = fresh();
        evidence.download = Some(DownloadEvidence {
            completion_reported: true,
            file_exists: false,
            file_name: "invoice.pdf".to_owned(),
            size_bytes: 0,
            size_plausible: false,
        });
        let condition = Postcondition::DownloadCompleted {
            file_name: "invoice.pdf".to_owned(),
        };
        assert_eq!(
            evaluate(std::slice::from_ref(&condition), &evidence).outcome,
            VerificationOutcome::VerifiedFailure
        );

        evidence.download = Some(DownloadEvidence {
            completion_reported: true,
            file_exists: true,
            file_name: "invoice.pdf".to_owned(),
            size_bytes: 51_200,
            size_plausible: true,
        });
        assert_eq!(
            evaluate(&[condition], &evidence).outcome,
            VerificationOutcome::VerifiedSuccess
        );

        evidence.download = None;
        assert_eq!(
            evaluate(
                &[Postcondition::DownloadCompleted {
                    file_name: "invoice.pdf".to_owned()
                }],
                &evidence
            )
            .outcome,
            VerificationOutcome::Unknown
        );
    }

    #[test]
    fn authentication_success_is_independent_of_a_page_message() {
        let mut evidence = fresh();
        evidence.observed_text = Some("You are signed in".to_owned());
        let report = evaluate(&[Postcondition::Authenticated], &evidence);
        assert_eq!(
            report.outcome,
            VerificationOutcome::Unknown,
            "page text alone cannot prove authentication"
        );
        evidence.authenticated = Some(true);
        assert_eq!(
            evaluate(&[Postcondition::Authenticated], &evidence).outcome,
            VerificationOutcome::VerifiedSuccess
        );
    }

    #[test]
    fn evidence_references_are_recorded_for_audit() {
        let mut evidence = fresh();
        evidence.observed_origin = Some(origin("https://example.com"));
        let report = evaluate(
            &[Postcondition::UrlOrigin(origin("https://example.com"))],
            &evidence,
        );
        assert_eq!(report.evidence_references().len(), 1);
        assert!(report.evidence_references()[0].ends_with(":satisfied"));
    }
}
