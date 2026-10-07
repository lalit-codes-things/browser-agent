// Verification outcomes.
//
// C-91: verification has exactly five outcome categories.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum VerificationOutcome {
    VerifiedSuccess,
    LikelySuccess,
    Unknown,
    LikelyFailure,
    VerifiedFailure,
}

impl VerificationOutcome {
    /// Success family for classification: VERIFIED_SUCCESS and
    /// LIKELY_SUCCESS. UI treatment differs (DESIGN.md: LIKELY_SUCCESS
    /// never receives verified-green styling); see is_verified.
    pub fn is_success_family(&self) -> bool {
        matches!(self, Self::VerifiedSuccess | Self::LikelySuccess)
    }

    /// True only for VERIFIED_SUCCESS: the sole outcome permitted
    /// verified-green treatment and success claims.
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::VerifiedSuccess)
    }

    pub fn is_failure_family(&self) -> bool {
        matches!(self, Self::VerifiedFailure | Self::LikelyFailure)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verified_success_is_success_family() {
        assert!(VerificationOutcome::VerifiedSuccess.is_success_family());
    }

    #[test]
    fn likely_success_is_success_family_but_never_verified() {
        // LIKELY_SUCCESS classifies as success-family for triage, but is
        // distinct from VERIFIED_SUCCESS and must never receive
        // verified-green treatment (DESIGN.md #6).
        let outcome = VerificationOutcome::LikelySuccess;
        assert!(outcome.is_success_family());
        assert!(!outcome.is_verified());
        assert_ne!(outcome, VerificationOutcome::VerifiedSuccess);
    }

    #[test]
    fn unknown_is_not_success_or_failure() {
        assert!(VerificationOutcome::Unknown.is_unknown());
    }
}
