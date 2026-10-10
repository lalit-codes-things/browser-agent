// Egress status.
//
// C-71/C-73: egress enforcement is a platform runtime property (local
// proxy + pf). The frontend must never infer enforcement from a Policy
// decision or from the configured mode alone. This module reports both
// the configured mode and whether the enforcement runtime is actually
// active.

use crate::config::{EgressMode, RuntimeConfig};
use crate::ipc::events::EgressStateEvent;

/// Compute the current egress state from real runtime evidence.
///
/// The configured mode is what the runtime is set to; enforcement_active
/// reflects whether the platform enforcement path (proxy + pf helper) is
/// actually engaged. The two are never conflated.
pub fn current_status(config: &RuntimeConfig) -> EgressStateEvent {
    let mode = match config.egress_mode {
        EgressMode::Enforced => "ENFORCED",
        EgressMode::Disabled => "DISABLED",
    };

    // Enforcement is only real when the pf helper is installed AND the
    // local proxy control is running. Neither runtime is attached yet, so
    // enforcement is honestly reported as inactive; the configured mode is
    // still shown so the operator sees the intended policy.
    let helper_present = crate::net::pf::PfHelper::install_degradation_present();
    let enforcement_active = false;

    EgressStateEvent {
        mode: mode.into(),
        enforcement_active,
        reason: Some(if enforcement_active {
            "none".into()
        } else if helper_present {
            "EGRESS_ENFORCEMENT_RUNTIME_NOT_STARTED".into()
        } else {
            "EGRESS_PF_HELPER_NOT_INSTALLED".into()
        })
        .filter(|r| r != "none"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(mode: EgressMode) -> RuntimeConfig {
        RuntimeConfig {
            model_manifest_path: String::new(),
            installed_models_dir: String::new(),
            audit_segment_dir: String::new(),
            quarantine_dir: String::new(),
            profiles_dir: String::new(),
            egress_mode: mode,
            chromium_executable: None,
            max_steps: None,
            max_llm_calls: None,
            max_recovery_attempts: None,
            max_task_duration_ms: None,
            max_confirmations: None,
        }
    }

    #[test]
    fn enforced_mode_without_runtime_is_not_reported_as_active() {
        let status = current_status(&config(EgressMode::Enforced));
        assert_eq!(status.mode, "ENFORCED");
        // The enforcement runtime is not attached; never claim it is.
        assert!(!status.enforcement_active);
        assert!(status.reason.is_some());
    }

    #[test]
    fn disabled_mode_has_no_enforcement_claim() {
        let status = current_status(&config(EgressMode::Disabled));
        assert_eq!(status.mode, "DISABLED");
        assert!(!status.enforcement_active);
    }
}
