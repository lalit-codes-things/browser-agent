//! Browser-Agent local model supervisor (Sections 16.1-16.4, BA-MEMORY-001).
//!
//! This crate owns the model artifact manifest, integrity verification, the
//! readiness state machine and the memory-profile parking decision. It does not
//! implement inference: the llama.cpp runner is a managed child process, and
//! this crate defines the contract that child process must satisfy.
//!
//! Nothing here starts an HTTP server or a localhost endpoint
//! (Section 14.2): the runner uses managed stdin/stdout JSONL.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use ba_domain::{ReadinessState, readiness_controls, readiness_label};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Failure modes for model artifacts and readiness.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    /// The artifact digest did not match the manifest.
    #[error("artifact digest mismatch: expected {expected}, measured {measured}")]
    DigestMismatch {
        /// Digest recorded in the manifest.
        expected: String,
        /// Digest measured from the downloaded bytes.
        measured: String,
    },
    /// The artifact size did not match the manifest.
    #[error("artifact size mismatch: expected {expected} bytes, measured {measured}")]
    SizeMismatch {
        /// Size recorded in the manifest.
        expected: u64,
        /// Measured size.
        measured: u64,
    },
    /// The manifest is not usable.
    #[error("model manifest is incomplete: {0}")]
    InvalidManifest(String),
    /// The requested readiness transition is not allowed.
    #[error("readiness cannot move from {from} to {to}")]
    IllegalTransition {
        /// Current state.
        from: ReadinessState,
        /// Requested state.
        to: ReadinessState,
    },
    /// No model is installed, so none can be started.
    #[error("no model artifact is installed")]
    NotInstalled,
}

/// The hardware memory profile used for resource ceilings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MemoryProfile {
    /// 8 GB Apple silicon reference profile.
    Compact,
    /// 16 GB Windows x64 reference profile.
    Standard,
}

impl MemoryProfile {
    /// Ceiling for the model plus inference runner, in bytes.
    #[must_use]
    pub const fn model_and_runner_cap(self) -> u64 {
        match self {
            Self::Compact => 2_500_000_000,
            Self::Standard => 4_000_000_000,
        }
    }

    /// Ceiling for all Browser-Agent processes, in bytes.
    #[must_use]
    pub const fn total_process_cap(self) -> u64 {
        match self {
            Self::Compact => 6_000_000_000,
            Self::Standard => 9_000_000_000,
        }
    }

    /// The measured level at which inference must be parked.
    #[must_use]
    pub const fn park_threshold(self) -> u64 {
        match self {
            Self::Compact => 5_500_000_000,
            Self::Standard => 8_500_000_000,
        }
    }
}

/// Whether inference must be parked now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParkDecision {
    /// Memory is within budget.
    Continue,
    /// Inference must be parked at the next safe task boundary.
    Park {
        /// Measured total process memory.
        measured_bytes: u64,
        /// Threshold that was crossed.
        threshold_bytes: u64,
    },
}

/// A frozen, integrity-checkable model artifact description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelManifest {
    /// Stable artifact identifier.
    pub id: String,
    /// File name inside the app-owned models directory.
    pub file_name: String,
    /// Lowercase hex SHA-256 of the artifact bytes.
    pub sha256: String,
    /// Artifact size in bytes.
    pub size_bytes: u64,
    /// Quantization label, for example `Q4_K_M`.
    pub quantization: String,
    /// Context limit in tokens.
    pub context_limit: u32,
    /// Runtime build the artifact was validated against.
    pub runtime_build: String,
    /// Upstream source recorded for provenance.
    pub source: String,
    /// License recorded for notices.
    pub license: String,
    /// Memory profile the artifact was measured on.
    pub profile: MemoryProfile,
    /// Whether the artifact passed the frozen benchmark.
    pub benchmark_passed: bool,
}

impl ModelManifest {
    /// Validates the manifest's own completeness.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::InvalidManifest`] when a required field is empty or
    /// the digest is not a 64-character hex string.
    pub fn validate(&self) -> Result<(), ModelError> {
        if self.id.trim().is_empty() || self.file_name.trim().is_empty() {
            return Err(ModelError::InvalidManifest("missing identity".to_owned()));
        }
        if self.sha256.len() != 64 || !self.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ModelError::InvalidManifest(
                "digest is not SHA-256 hex".to_owned(),
            ));
        }
        if self.file_name.contains('/')
            || self.file_name.contains('\\')
            || self.file_name.contains("..")
        {
            return Err(ModelError::InvalidManifest(
                "file name must be a plain name inside the app data directory".to_owned(),
            ));
        }
        if self.size_bytes == 0 || self.context_limit == 0 {
            return Err(ModelError::InvalidManifest(
                "missing size or context limit".to_owned(),
            ));
        }
        if self.runtime_build.trim().is_empty() || self.source.trim().is_empty() {
            return Err(ModelError::InvalidManifest(
                "missing runtime build or source".to_owned(),
            ));
        }
        if !self.benchmark_passed {
            return Err(ModelError::InvalidManifest(
                "the artifact has not passed the frozen benchmark".to_owned(),
            ));
        }
        Ok(())
    }

    /// Verifies downloaded bytes against the manifest.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] on a size or digest mismatch. A failed
    /// verification never yields a loadable artifact.
    pub fn verify_bytes(&self, bytes: &[u8]) -> Result<(), ModelError> {
        self.validate()?;
        let measured_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if measured_size != self.size_bytes {
            return Err(ModelError::SizeMismatch {
                expected: self.size_bytes,
                measured: measured_size,
            });
        }
        let measured = hex(&Sha256::digest(bytes));
        if measured != self.sha256.to_ascii_lowercase() {
            return Err(ModelError::DigestMismatch {
                expected: self.sha256.to_ascii_lowercase(),
                measured,
            });
        }
        Ok(())
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Measured progress of an installation or verification pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeasuredProgress {
    /// Bytes processed so far.
    pub done_bytes: u64,
    /// Total bytes expected.
    pub total_bytes: u64,
}

impl MeasuredProgress {
    /// Returns the percentage completed, saturating at 100.
    #[must_use]
    pub fn percent(self) -> u8 {
        if self.total_bytes == 0 {
            return 0;
        }
        let value = self.done_bytes.saturating_mul(100) / self.total_bytes;
        u8::try_from(value.min(100)).unwrap_or(100)
    }
}

/// The readiness state machine.
#[derive(Debug, Clone)]
pub struct ReadinessMachine {
    state: ReadinessState,
    installed: Option<ModelManifest>,
    progress: Option<MeasuredProgress>,
    last_cause: Option<String>,
}

impl Default for ReadinessMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadinessMachine {
    /// Creates a machine in `NOT_INSTALLED`.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: ReadinessState::NotInstalled,
            installed: None,
            progress: None,
            last_cause: None,
        }
    }

    /// Returns the current readiness state.
    #[must_use]
    pub const fn state(&self) -> ReadinessState {
        self.state
    }

    /// Returns the user-facing label for the current state.
    #[must_use]
    pub fn label(&self) -> &'static str {
        readiness_label(self.state)
    }

    /// Returns the controls the current state actually supports.
    #[must_use]
    pub fn controls(&self) -> &'static [&'static str] {
        readiness_controls(self.state)
    }

    /// Returns whether tasks may be accepted.
    #[must_use]
    pub const fn accepts_tasks(&self) -> bool {
        self.state.accepts_tasks()
    }

    /// Returns the installed manifest, when one is verified and present.
    #[must_use]
    pub const fn installed(&self) -> Option<&ModelManifest> {
        self.installed.as_ref()
    }

    /// Returns current measured progress.
    #[must_use]
    pub const fn progress(&self) -> Option<MeasuredProgress> {
        self.progress
    }

    /// Returns the plain-language cause of the current state.
    #[must_use]
    pub fn cause(&self) -> Option<&str> {
        self.last_cause.as_deref()
    }

    /// Starts or restarts an installation.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::IllegalTransition`] from a state that cannot begin
    /// an installation.
    pub fn start_install(&mut self) -> Result<(), ModelError> {
        match self.state {
            ReadinessState::NotInstalled | ReadinessState::IntegrityFailed => {
                self.state = ReadinessState::Installing;
                self.progress = Some(MeasuredProgress {
                    done_bytes: 0,
                    total_bytes: 0,
                });
                self.last_cause = None;
                Ok(())
            }
            from => Err(ModelError::IllegalTransition {
                from,
                to: ReadinessState::Installing,
            }),
        }
    }

    /// Reports measured installation progress.
    pub fn report_progress(&mut self, progress: MeasuredProgress) {
        if self.state == ReadinessState::Installing {
            self.progress = Some(progress);
        }
    }

    /// Cancels an installation, discarding the partial artifact.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::IllegalTransition`] when no installation is
    /// running.
    pub fn cancel_install(&mut self) -> Result<(), ModelError> {
        if self.state != ReadinessState::Installing {
            return Err(ModelError::IllegalTransition {
                from: self.state,
                to: ReadinessState::NotInstalled,
            });
        }
        self.state = ReadinessState::NotInstalled;
        self.progress = None;
        self.last_cause =
            Some("installation cancelled; the partial artifact was removed".to_owned());
        Ok(())
    }

    /// Records the outcome of an integrity verification.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] when the artifact does not verify; the state
    /// becomes `INTEGRITY_FAILED` so the artifact is never loaded.
    pub fn complete_install(
        &mut self,
        manifest: ModelManifest,
        bytes: &[u8],
    ) -> Result<(), ModelError> {
        if self.state != ReadinessState::Installing {
            return Err(ModelError::IllegalTransition {
                from: self.state,
                to: ReadinessState::Ready,
            });
        }
        match manifest.verify_bytes(bytes) {
            Ok(()) => {
                self.progress = Some(MeasuredProgress {
                    done_bytes: manifest.size_bytes,
                    total_bytes: manifest.size_bytes,
                });
                self.installed = Some(manifest);
                self.state = ReadinessState::Loading;
                self.last_cause = None;
                Ok(())
            }
            Err(error) => {
                self.state = ReadinessState::IntegrityFailed;
                self.installed = None;
                self.last_cause = Some(error.to_string());
                Err(error)
            }
        }
    }

    /// Re-verifies the installed artifact.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError`] when nothing is installed or verification fails.
    pub fn reverify(&mut self, bytes: &[u8]) -> Result<(), ModelError> {
        let manifest = self.installed.clone().ok_or(ModelError::NotInstalled)?;
        match manifest.verify_bytes(bytes) {
            Ok(()) => {
                self.last_cause = Some("integrity verified".to_owned());
                Ok(())
            }
            Err(error) => {
                self.state = ReadinessState::IntegrityFailed;
                self.last_cause = Some(error.to_string());
                Err(error)
            }
        }
    }

    /// Marks the model as ready after runner health and schema checks pass.
    ///
    /// # Errors
    ///
    /// Returns [`ModelError::NotInstalled`] when no artifact is installed.
    pub fn mark_ready(&mut self) -> Result<(), ModelError> {
        if self.installed.is_none() {
            return Err(ModelError::NotInstalled);
        }
        if self.state != ReadinessState::Loading && self.state != ReadinessState::Parked {
            return Err(ModelError::IllegalTransition {
                from: self.state,
                to: ReadinessState::Ready,
            });
        }
        self.state = ReadinessState::Ready;
        self.last_cause = None;
        Ok(())
    }

    /// Records an incompatible hard result.
    pub fn mark_incompatible(&mut self, cause: &str) {
        self.state = ReadinessState::Incompatible;
        self.last_cause = Some(cause.to_owned());
    }

    /// Parks inference to free memory.
    pub fn park(&mut self, cause: &str) {
        if self.state == ReadinessState::Ready || self.state == ReadinessState::Loading {
            self.state = ReadinessState::Parked;
            self.last_cause = Some(cause.to_owned());
        }
    }

    /// Records that the runner stopped unexpectedly.
    pub fn runner_failed(&mut self, cause: &str) {
        self.state = ReadinessState::RunnerFailed;
        self.last_cause = Some(cause.to_owned());
    }

    /// Records a resource stop that protects system performance.
    pub fn resource_stopped(&mut self, cause: &str) {
        self.state = ReadinessState::ResourceStopped;
        self.last_cause = Some(cause.to_owned());
    }

    /// Returns whether the task may start now.
    ///
    /// A readiness failure never renders as a task failure: the caller keeps the
    /// unsent instruction and offers only controls the state supports.
    #[must_use]
    pub const fn can_accept_task(&self) -> bool {
        matches!(self.state, ReadinessState::Ready)
    }
}

/// Evaluates the memory profile against a measured total.
#[must_use]
pub fn park_decision(profile: MemoryProfile, measured_total_bytes: u64) -> ParkDecision {
    let threshold = profile.park_threshold();
    if measured_total_bytes >= threshold {
        ParkDecision::Park {
            measured_bytes: measured_total_bytes,
            threshold_bytes: threshold,
        }
    } else {
        ParkDecision::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_for(bytes: &[u8]) -> ModelManifest {
        ModelManifest {
            id: "qwen3-1.7b-q4_k_m".to_owned(),
            file_name: "Qwen3-1.7B-Q4_K_M.gguf".to_owned(),
            sha256: hex(&Sha256::digest(bytes)),
            size_bytes: u64::try_from(bytes.len()).unwrap(),
            quantization: "Q4_K_M".to_owned(),
            context_limit: 8_192,
            runtime_build: "llama.cpp-b6000".to_owned(),
            source: "local fixture".to_owned(),
            license: "Apache-2.0".to_owned(),
            profile: MemoryProfile::Compact,
            benchmark_passed: true,
        }
    }

    #[test]
    fn artifacts_are_only_accepted_when_the_bytes_match() {
        let bytes = b"model-weights".to_vec();
        let manifest = manifest_for(&bytes);
        assert_eq!(manifest.verify_bytes(&bytes), Ok(()));

        let truncated = b"model-weight".to_vec();
        assert!(matches!(
            manifest.verify_bytes(&truncated),
            Err(ModelError::SizeMismatch { .. })
        ));

        let mut same_size = bytes.clone();
        same_size[0] = b'M';
        assert!(matches!(
            manifest.verify_bytes(&same_size),
            Err(ModelError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn manifests_must_be_complete_and_benchmarked() {
        let bytes = b"model-weights".to_vec();
        let mut manifest = manifest_for(&bytes);
        manifest.benchmark_passed = false;
        assert!(matches!(
            manifest.validate(),
            Err(ModelError::InvalidManifest(_))
        ));

        let mut manifest = manifest_for(&bytes);
        manifest.file_name = "../escape.gguf".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ModelError::InvalidManifest(_))
        ));

        let mut manifest = manifest_for(&bytes);
        manifest.sha256 = "not-a-digest".to_owned();
        assert!(matches!(
            manifest.validate(),
            Err(ModelError::InvalidManifest(_))
        ));
    }

    #[test]
    fn a_failed_integrity_check_never_reaches_ready() {
        let bytes = b"model-weights".to_vec();
        let mut machine = ReadinessMachine::new();
        assert_eq!(machine.state(), ReadinessState::NotInstalled);
        assert!(!machine.can_accept_task());
        machine.start_install().unwrap();
        assert_eq!(machine.label(), "Installing model");
        assert_eq!(machine.controls(), &["Cancel"]);
        let tampered = b"model-weightz".to_vec();
        assert!(
            machine
                .complete_install(manifest_for(&bytes), &tampered)
                .is_err()
        );
        assert_eq!(machine.state(), ReadinessState::IntegrityFailed);
        assert_eq!(machine.label(), "Model failed its integrity check");
        assert_eq!(machine.controls(), &["Retry verification", "Remove model"]);
        assert!(!machine.accepts_tasks());
    }

    #[test]
    fn a_verified_install_moves_through_loading_to_ready() {
        let bytes = b"model-weights".to_vec();
        let mut machine = ReadinessMachine::new();
        machine.start_install().unwrap();
        machine.report_progress(MeasuredProgress {
            done_bytes: 6,
            total_bytes: 13,
        });
        assert_eq!(machine.progress().map(MeasuredProgress::percent), Some(46));
        machine
            .complete_install(manifest_for(&bytes), &bytes)
            .unwrap();
        assert_eq!(machine.state(), ReadinessState::Loading);
        machine.mark_ready().unwrap();
        assert_eq!(machine.state(), ReadinessState::Ready);
        assert_eq!(machine.label(), "Model ready");
        assert!(machine.accepts_tasks());
    }

    #[test]
    fn cancelling_an_install_removes_the_partial_artifact() {
        let mut machine = ReadinessMachine::new();
        machine.start_install().unwrap();
        machine.cancel_install().unwrap();
        assert_eq!(machine.state(), ReadinessState::NotInstalled);
        assert!(
            machine
                .cause()
                .unwrap_or_default()
                .contains("partial artifact")
        );
        assert!(matches!(
            machine.cancel_install(),
            Err(ModelError::IllegalTransition { .. })
        ));
    }

    #[test]
    fn parking_and_runner_failures_keep_their_own_copy() {
        let bytes = b"model-weights".to_vec();
        let mut machine = ReadinessMachine::new();
        machine.start_install().unwrap();
        machine
            .complete_install(manifest_for(&bytes), &bytes)
            .unwrap();
        machine.mark_ready().unwrap();

        machine.park("total process memory reached the safe limit");
        assert_eq!(machine.state(), ReadinessState::Parked);
        assert_eq!(machine.label(), "Model paused to free memory");
        assert_eq!(machine.controls(), &["Resume model"]);
        assert!(!machine.can_accept_task());
        machine.mark_ready().unwrap();
        assert!(machine.can_accept_task());

        machine.runner_failed("the runner exited unexpectedly");
        assert_eq!(machine.controls(), &["Restart model for the next task"]);
        machine.resource_stopped("memory pressure");
        assert_eq!(
            machine.label(),
            "Model stopped to protect system performance"
        );
        assert!(machine.controls().is_empty());
    }

    #[test]
    fn reverify_uses_the_installed_manifest() {
        let bytes = b"model-weights".to_vec();
        let mut machine = ReadinessMachine::new();
        assert!(matches!(
            machine.reverify(&bytes),
            Err(ModelError::NotInstalled)
        ));
        machine.start_install().unwrap();
        machine
            .complete_install(manifest_for(&bytes), &bytes)
            .unwrap();
        assert_eq!(machine.reverify(&bytes), Ok(()));
        assert!(
            machine
                .cause()
                .unwrap_or_default()
                .contains("integrity verified")
        );
    }

    #[test]
    fn memory_profiles_park_before_the_total_cap() {
        assert_eq!(
            park_decision(MemoryProfile::Compact, 5_600_000_000),
            ParkDecision::Park {
                measured_bytes: 5_600_000_000,
                threshold_bytes: 5_500_000_000,
            }
        );
        assert_eq!(
            park_decision(MemoryProfile::Compact, 5_400_000_000),
            ParkDecision::Continue
        );
        assert_eq!(
            park_decision(MemoryProfile::Standard, 8_600_000_000),
            ParkDecision::Park {
                measured_bytes: 8_600_000_000,
                threshold_bytes: 8_500_000_000,
            }
        );
        assert!(
            MemoryProfile::Compact.park_threshold() < MemoryProfile::Compact.total_process_cap()
        );
    }
}
