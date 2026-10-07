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
    pub fn is_success_family(&self) -> bool {
        matches!(self, Self::VerifiedSuccess | Self::LikelySuccess)
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
    fn likely_success_is_not_verified_success() {
        assert!(!VerificationOutcome::LikelySuccess.is_success_family() && !matches!(VerificationOutcome::LikelySuccess, VerificationOutcome::VerifiedSuccess));
    }

    #[test]
    fn unknown_is_not_success_or_failure() {
        assert!(VerificationOutcome::Unknown.is_unknown());
    }
}
