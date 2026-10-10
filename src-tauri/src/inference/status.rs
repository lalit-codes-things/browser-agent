// Model status.
//
// Derives the real model artifact state from the pinned manifest and the
// installed-artifact check (C-41 to C-45). The frontend projects this
// state; it never invents verification or residency.
//
// Availability vocabulary:
//  - VERIFIED_LOADED: artifact loaded and SHA-256 verified at load.
//  - PRESENT_UNVERIFIED: artifact bytes exist; integrity not verified yet
//    (verification runs at load, not on every status query).
//  - UNAVAILABLE: manifest or artifact is absent; reasoning cannot run.

use std::path::Path;

use crate::config::RuntimeConfig;
use crate::inference::pin::ModelPin;
use crate::ipc::events::ModelStateEvent;

/// Compute the current model state from real filesystem evidence.
///
/// This never hashes the full artifact (that runs at load); it reports
/// presence truthfully and leaves verification claims absent until a real
/// load has verified the bytes.
pub fn current_status(config: &RuntimeConfig) -> ModelStateEvent {
    let manifest_path = Path::new(&config.model_manifest_path);
    let pin: ModelPin = match std::fs::read_to_string(manifest_path)
        .ok()
        .and_then(|raw| serde_json::from_str::<ModelPin>(&raw).ok())
    {
        Some(pin) => pin,
        None => {
            return ModelStateEvent {
                model_id: "UNKNOWN".into(),
                quantization: "UNKNOWN".into(),
                residency: "UNLOADED".into(),
                availability: "UNAVAILABLE".into(),
                sha256: "UNKNOWN".into(),
                reason: Some("MODEL_MANIFEST_UNREADABLE".into()),
            };
        }
    };

    let models_dir = Path::new(&config.installed_models_dir);
    // The artifact may sit at the repo-local provisional path or directly
    // under the installed models directory; both are inside the pinned
    // install root and checked for presence only.
    let candidates = [
        models_dir.join("provisional").join(&pin.source.file),
        models_dir.join(&pin.source.file),
    ];
    let installed = candidates.iter().any(|p| p.is_file());

    ModelStateEvent {
        model_id: pin.artifact_id,
        quantization: pin.model.quantization,
        residency: "UNLOADED".into(),
        availability: if installed {
            "PRESENT_UNVERIFIED".into()
        } else {
            "UNAVAILABLE".into()
        },
        sha256: pin.sha256,
        reason: if installed {
            None
        } else {
            Some("MODEL_ARTIFACT_NOT_INSTALLED".into())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(manifest: &str, models_dir: &str) -> RuntimeConfig {
        RuntimeConfig {
            model_manifest_path: manifest.into(),
            installed_models_dir: models_dir.into(),
            audit_segment_dir: String::new(),
            quarantine_dir: String::new(),
            profiles_dir: String::new(),
            egress_mode: crate::config::EgressMode::Enforced,
            chromium_executable: None,
            max_steps: None,
            max_llm_calls: None,
            max_recovery_attempts: None,
            max_task_duration_ms: None,
            max_confirmations: None,
        }
    }

    #[test]
    fn missing_manifest_reports_unavailable_not_verified() {
        let status = current_status(&config("/nonexistent/manifest.json", "/nonexistent/models"));
        assert_eq!(status.availability, "UNAVAILABLE");
        assert_eq!(status.residency, "UNLOADED");
        assert_eq!(status.model_id, "UNKNOWN");
        assert_eq!(status.reason.as_deref(), Some("MODEL_MANIFEST_UNREADABLE"));
    }

    #[test]
    fn present_manifest_without_artifact_reports_not_installed() {
        // The repo manifest exists; the artifact does not.
        let manifest = format!(
            "{}/../models/manifests/phase1-provisional.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let status = current_status(&config(&manifest, "/nonexistent/models"));
        assert_eq!(status.availability, "UNAVAILABLE");
        assert_eq!(status.model_id, "qwen3-1.7b-q4_k_m");
        assert_eq!(status.quantization, "Q4_K_M");
        assert_eq!(
            status.sha256,
            "d2387ca2dbfee2ffabce7120d3770dadca0b293052bc2f0e138fdc940d9bc7b5"
        );
        assert_eq!(
            status.reason.as_deref(),
            Some("MODEL_ARTIFACT_NOT_INSTALLED")
        );
        // No verification claim without a load.
        assert_ne!(status.availability, "VERIFIED_LOADED");
    }
}
