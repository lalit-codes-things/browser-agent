//! User-first error copy (Section 19, BA-ERROR-COPY-001).
//!
//! Every visible error follows the four-part contract: what was attempted, what
//! was observed, what is known or unknown now, and the safe next action. Raw
//! stack traces, CEF diagnostics and error identifiers stay in the redacted
//! diagnostic view.

use serde::Serialize;

/// One four-part error message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FourPartError {
    /// Stable identifier shown only in diagnostic detail.
    pub error_id: &'static str,
    /// What the app was trying to do.
    pub attempted: &'static str,
    /// What was observed.
    pub observed: &'static str,
    /// What is known and what remains unknown.
    pub known: &'static str,
    /// The safe next action available to the user.
    pub next_action: &'static str,
}

/// The required worked error examples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCase {
    /// A page failed to load.
    NavigationFailed,
    /// The local model is not ready to accept tasks.
    ModelNotReady,
    /// The inference runner exited during a task.
    RunnerCrashed,
    /// Trusted policy refused the proposed action.
    PolicyBlocked,
    /// An approval request expired before a decision.
    ApprovalExpired,
    /// Verification could not establish the result.
    VerificationUnknown,
    /// An Agent-initiated download was refused.
    DownloadRejected,
    /// Required audit recording failed.
    AuditWriteFailed,
    /// The runtime stopped to protect a safe state.
    ResourceStop,
    /// The managed browser host is not attached to this process.
    BrowserHostUnavailable,
}

impl ErrorCase {
    /// Every case.
    pub const ALL: [Self; 10] = [
        Self::NavigationFailed,
        Self::ModelNotReady,
        Self::RunnerCrashed,
        Self::PolicyBlocked,
        Self::ApprovalExpired,
        Self::VerificationUnknown,
        Self::DownloadRejected,
        Self::AuditWriteFailed,
        Self::ResourceStop,
        Self::BrowserHostUnavailable,
    ];

    /// Returns the canonical copy for this case.
    #[must_use]
    pub const fn copy(self) -> FourPartError {
        match self {
            Self::NavigationFailed => FourPartError {
                error_id: "BA-ERR-NAV-001",
                attempted: "Browser-Agent tried to open a page in the managed browser.",
                observed: "The browser reported that the page did not load.",
                known: "The address is shown exactly as the browser resolved it. The page content was never displayed, and no agent action ran on it.",
                next_action: "Check the address, then try again. The browser error details are available in Diagnostics.",
            },
            Self::ModelNotReady => FourPartError {
                error_id: "BA-ERR-MODEL-001",
                attempted: "Browser-Agent tried to start this task with the local model.",
                observed: "The local model is not ready to accept tasks.",
                known: "No page action was proposed or taken. Browsing and your saved data are unaffected.",
                next_action: "Open Model status to install, verify or resume the model, or keep browsing without the agent.",
            },
            Self::RunnerCrashed => FourPartError {
                error_id: "BA-ERR-MODEL-002",
                attempted: "The agent was working on your task with the local model.",
                observed: "The model runner exited unexpectedly.",
                known: "Further steps are stopped and the last safe state is kept. If a step may already have reached the page, its result is reported as unknown.",
                next_action: "Review the current page, then restart the model for the next task. The previous step is not replayed automatically.",
            },
            Self::PolicyBlocked => FourPartError {
                error_id: "BA-ERR-POLICY-001",
                attempted: "The agent proposed one action on the current page.",
                observed: "Trusted policy refused the action before anything was dispatched.",
                known: "Nothing was sent to the site. The rule that refused the action and what it protects are listed below.",
                next_action: "Read the reason, then choose a different step, or perform this step yourself and resume.",
            },
            Self::ApprovalExpired => FourPartError {
                error_id: "BA-ERR-APPROVAL-001",
                attempted: "The agent asked you to approve one bounded action.",
                observed: "The approval request expired before you decided.",
                known: "Nothing was executed. The request is no longer actionable and will not be renewed on its own.",
                next_action: "Continue the task to let the agent inspect the page again and prepare a fresh request.",
            },
            Self::VerificationUnknown => FourPartError {
                error_id: "BA-ERR-VERIFY-001",
                attempted: "The agent completed a step and checked its result.",
                observed: "The available evidence does not establish whether the step succeeded.",
                known: "The step may have reached the site. The app does not report success or failure for it.",
                next_action: "Look at the page yourself, then resume the task or cancel it. The step is not repeated automatically.",
            },
            Self::DownloadRejected => FourPartError {
                error_id: "BA-ERR-DOWNLOAD-001",
                attempted: "The agent tried to save a file to the folder you chose.",
                observed: "The download was refused before anything was written.",
                known: "No file was written and nothing was quarantined. The reason is listed below.",
                next_action: "Choose another destination or approve the file again from the Downloads view.",
            },
            Self::AuditWriteFailed => FourPartError {
                error_id: "BA-ERR-AUDIT-001",
                attempted: "The agent tried an action that changes something.",
                observed: "Required recording of the action failed.",
                known: "New actions are stopped so the record stays trustworthy. Reading the page still works. The log is marked degraded and its integrity is not claimed.",
                next_action: "Free disk space or check the app data folder, then restart Browser-Agent to re-open the log.",
            },
            Self::BrowserHostUnavailable => FourPartError {
                error_id: "BA-ERR-BROWSER-001",
                attempted: "Browser-Agent tried to open or change a page.",
                observed: "The managed browser host is not attached to this process.",
                known: "No page was created, changed or loaded, and no agent action ran. Nothing was simulated in the place of a real page.",
                next_action: "Start Browser-Agent with its managed browser host, then try the action again.",
            },
            Self::ResourceStop => FourPartError {
                error_id: "BA-ERR-RESOURCE-001",
                attempted: "The agent was running a task with the local model.",
                observed: "Measured memory pressure reached the safe limit for this computer.",
                known: "Inference was stopped at a safe task boundary and the last checkpoint is kept. Browsing continues normally.",
                next_action: "Close some tabs or other apps, then resume the model from Model status when memory is available.",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_error_has_all_four_parts_and_no_placeholder() {
        for case in ErrorCase::ALL {
            let copy = case.copy();
            assert!(copy.error_id.starts_with("BA-ERR-"));
            for part in [copy.attempted, copy.observed, copy.known, copy.next_action] {
                assert!(!part.is_empty(), "{:?} has an empty part", case);
                assert!(!part.to_ascii_lowercase().contains("something went wrong"));
                assert!(!part.trim_end().ends_with("Done"));
            }
        }
    }

    #[test]
    fn error_ids_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for case in ErrorCase::ALL {
            assert!(
                seen.insert(case.copy().error_id),
                "duplicate id for {case:?}"
            );
        }
    }

    #[test]
    fn audit_failure_copy_never_claims_a_verified_chain() {
        let copy = ErrorCase::AuditWriteFailed.copy();
        assert!(copy.known.contains("not claimed"));
    }
}
