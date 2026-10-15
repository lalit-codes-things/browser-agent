//! Browser-Agent audit log (BA-AUDIT-001, BA-AUDIT-002).
//!
//! Events are canonicalized to a deterministic byte form, chained with an HMAC
//! keyed from platform-protected storage, and rotated into bounded segments
//! that carry the previous segment's final MAC. The log is redacted by
//! construction: records accept only non-secret fields, and URLs pass through
//! the domain redactor before they are stored.
//!
//! A local HMAC chain is tamper evidence only while the key and the trusted
//! process boundary remain trustworthy. It is not proof against an attacker who
//! controls the process and its key, and the crate never claims otherwise.
//!
//! Audit-write failure is a fail-closed safety event: the log reports
//! [`AuditState::Degraded`] and callers must stop new non-`READ` dispatch.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![deny(missing_docs)]

use std::collections::BTreeMap;

use ba_domain::{
    ActionClass, ActionId, AuditSegmentId, BrowserContextId, TaskId, UnixMillis,
    VerificationOutcome, canonical_json, redact_url,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Maximum size of one audit segment before rotation (10 MiB).
pub const MAX_SEGMENT_BYTES: usize = 10 * 1024 * 1024;

/// Default retention window in days.
pub const DEFAULT_RETENTION_DAYS: u32 = 90;

/// Retention windows the user may choose.
pub const ALLOWED_RETENTION_DAYS: [u32; 3] = [30, 90, 365];

/// Length of an audit MAC in bytes.
pub const MAC_LEN: usize = 32;

/// The audit key could not be obtained.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuditKeyError {
    /// Platform secure storage is unavailable, so no key can be created.
    #[error("platform secure storage is unavailable; audit integrity cannot be established")]
    StorageUnavailable,
    /// The stored key had the wrong length.
    #[error("audit key has length {0}, expected 32")]
    InvalidLength(usize),
}

/// Source of the audit HMAC key.
///
/// The production implementation reads platform-protected storage. This trait
/// exists so the crate never falls back to a key baked into config, source or
/// an environment dump.
pub trait AuditKeySource {
    /// Returns the 32-byte audit key.
    ///
    /// # Errors
    ///
    /// Returns [`AuditKeyError`] when the key cannot be read.
    fn key(&self) -> Result<[u8; 32], AuditKeyError>;
}

/// A key source for tests and for injected platform adapters.
#[derive(Debug, Clone, Copy)]
pub struct StaticKeySource([u8; 32]);

impl StaticKeySource {
    /// Wraps a 32-byte key.
    #[must_use]
    pub const fn new(key: [u8; 32]) -> Self {
        Self(key)
    }
}

impl AuditKeySource for StaticKeySource {
    fn key(&self) -> Result<[u8; 32], AuditKeyError> {
        Ok(self.0)
    }
}

/// A key source that always fails, used to prove the fail-closed path.
#[derive(Debug, Clone, Copy)]
pub struct UnavailableKeySource;

impl AuditKeySource for UnavailableKeySource {
    fn key(&self) -> Result<[u8; 32], AuditKeyError> {
        Err(AuditKeyError::StorageUnavailable)
    }
}

/// Where committed segments are persisted.
pub trait AuditSink {
    /// Writes one canonical record line to `segment`.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError::Persist`] when the record could not be durably
    /// written.
    fn write(&mut self, segment: &AuditSegmentId, bytes: &[u8]) -> Result<(), AuditError>;

    /// Returns the bytes already stored for `segment`, for verification.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError::Persist`] when the segment cannot be read.
    fn read_segment(&self, segment: &AuditSegmentId) -> Result<Vec<Vec<u8>>, AuditError>;
}

/// A non-persistent sink used by tests.
#[derive(Debug, Default, Clone)]
pub struct MemorySink {
    segments: BTreeMap<String, Vec<Vec<u8>>>,
    fail_next_write: bool,
}

impl MemorySink {
    /// Creates an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes the next write fail, to exercise fault handling.
    pub fn fail_next_write(&mut self) {
        self.fail_next_write = true;
    }

    /// Returns the stored records for a segment.
    #[must_use]
    pub fn records(&self, segment: &AuditSegmentId) -> Vec<Vec<u8>> {
        self.segments
            .get(segment.as_str())
            .cloned()
            .unwrap_or_default()
    }

    /// Returns the number of stored segments.
    #[must_use]
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

impl AuditSink for MemorySink {
    fn write(&mut self, segment: &AuditSegmentId, bytes: &[u8]) -> Result<(), AuditError> {
        if self.fail_next_write {
            self.fail_next_write = false;
            return Err(AuditError::Persist("injected write failure".to_owned()));
        }
        self.segments
            .entry(segment.as_str().to_owned())
            .or_default()
            .push(bytes.to_vec());
        Ok(())
    }

    fn read_segment(&self, segment: &AuditSegmentId) -> Result<Vec<Vec<u8>>, AuditError> {
        Ok(self.records(segment))
    }
}

/// Audit failures.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AuditError {
    /// The key could not be obtained.
    #[error("audit key unavailable: {0}")]
    Key(#[from] AuditKeyError),
    /// A record could not be durably written.
    #[error("audit persistence failed: {0}")]
    Persist(String),
    /// The record could not be canonicalized.
    #[error("audit record could not be canonicalized")]
    Canonicalize,
    /// The log is degraded and refuses further consequential records.
    #[error("audit log is degraded: {0}")]
    Degraded(String),
}

/// Why the log is degraded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegradationCause {
    /// The key was unavailable.
    MissingKey,
    /// A write failed.
    WriteFailed(String),
    /// Verification found a broken chain.
    IntegrityFailure {
        /// Segment where the chain broke.
        segment: String,
        /// Index of the first bad record, when known.
        index: usize,
    },
}

/// Current audit health.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditState {
    /// Records are being written and verified.
    Healthy,
    /// Consequential records are refused until the user intervenes.
    Degraded(DegradationCause),
}

/// Categories of auditable event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "kind")]
pub enum AuditEventKind {
    /// A task transitioned.
    TaskTransition {
        /// State code.
        state: String,
        /// Wait reason code, when applicable.
        wait_reason: Option<String>,
    },
    /// Policy produced a decision.
    PolicyDecision {
        /// Rule code.
        rule: String,
        /// Decision class code.
        decision: String,
    },
    /// An authorization was consumed.
    AuthorizationConsumed {
        /// Authorization identifier.
        authorization: String,
        /// Digest of the dispatched action.
        digest: String,
    },
    /// The executor reported a dispatch result.
    DispatchResult {
        /// Result code.
        result: String,
        /// Plain-language detail.
        detail: String,
    },
    /// An approval was granted, cancelled, expired or invalidated.
    Approval {
        /// Lifecycle stage of the request.
        stage: String,
        /// Action identifier.
        action_id: String,
    },
    /// A masked value was deliberately revealed (BA-UNMASK-001).
    Unmask {
        /// Record category that was revealed.
        record_category: String,
        /// Who initiated the reveal.
        initiated_by: String,
    },
    /// The log transitioned health states.
    Health {
        /// New state.
        state: String,
        /// Plain-language cause.
        cause: String,
    },
}

/// One redacted audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditRecord {
    /// Owning task, when the event belongs to one.
    pub task_id: Option<TaskId>,
    /// Action the event refers to.
    pub action_id: Option<ActionId>,
    /// Trusted consequence class, when known.
    pub action_class: Option<ActionClass>,
    /// Redacted target or origin identity.
    pub target: Option<String>,
    /// Policy decision code, when the event has one.
    pub policy_decision: Option<String>,
    /// Execution result code.
    pub execution_result: Option<String>,
    /// Independent verification state. `None` means verification did not run.
    pub verification: Option<VerificationOutcome>,
    /// Pseudonymous browser context identity.
    pub browser_context: Option<BrowserContextId>,
    /// UTC event time.
    pub at_unix_ms: UnixMillis,
    /// Event payload.
    pub kind: AuditEventKind,
}

impl AuditRecord {
    /// Redacts a URL or origin-shaped string for storage.
    #[must_use]
    pub fn redact_target(value: &str) -> String {
        redact_url(value)
    }

    /// Returns a copy whose target has been redacted.
    #[must_use]
    pub fn with_redacted_target(mut self, target: &str) -> Self {
        self.target = Some(Self::redact_target(target));
        self
    }
}

/// One chained record as persisted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChainedRecord {
    schema_version: u16,
    segment: AuditSegmentId,
    index: u64,
    previous_mac: String,
    record: AuditRecord,
    mac: String,
}

/// Outcome of verifying a segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrityStatus {
    /// Every record's MAC matches and links correctly.
    Verified {
        /// Number of records verified in the retained range.
        records: usize,
    },
    /// The chain is broken.
    Broken {
        /// Segment where it broke.
        segment: String,
        /// Index of the first record that failed.
        index: usize,
    },
}

impl IntegrityStatus {
    /// Returns the user-facing sentence for this status.
    ///
    /// A broken chain is never described as verified.
    #[must_use]
    pub fn label(&self, pruned: bool) -> String {
        match self {
            Self::Verified { records } => {
                if pruned {
                    format!("Integrity verified for retained range ({records} records)")
                } else {
                    format!("Integrity verified ({records} records)")
                }
            }
            Self::Broken { segment, index } => {
                format!("Integrity check failed in segment {segment} at record {index}")
            }
        }
    }
}

/// The chained audit log.
///
/// The sink is a trait object and therefore not `Debug`; the log exposes
/// [`AuditLog::state`] instead of a derived debug representation so a debug
/// print can never leak record contents.
pub struct AuditLog {
    key: [u8; 32],
    segment: AuditSegmentId,
    segment_started_on_day: i64,
    index: u64,
    previous_mac: [u8; MAC_LEN],
    segment_bytes: usize,
    state: AuditState,
    pruned: bool,
    sink: Box<dyn AuditSink>,
}

impl AuditLog {
    /// Opens the log, failing closed when the key is unavailable.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError`] when the key cannot be read. The caller must
    /// treat this as [`AuditState::Degraded`] and refuse consequential work.
    pub fn open(
        source: &dyn AuditKeySource,
        sink: Box<dyn AuditSink>,
        now: UnixMillis,
    ) -> Result<Self, AuditError> {
        let key = source.key()?;
        let day = now.get().div_euclid(86_400_000);
        Ok(Self {
            key,
            segment: segment_for_day(day)?,
            segment_started_on_day: day,
            index: 0,
            previous_mac: [0u8; MAC_LEN],
            segment_bytes: 0,
            state: AuditState::Healthy,
            pruned: false,
            sink,
        })
    }

    /// Returns the current health.
    #[must_use]
    pub const fn state(&self) -> &AuditState {
        &self.state
    }

    /// Returns whether consequential work must stop.
    #[must_use]
    pub fn is_degraded(&self) -> bool {
        matches!(self.state, AuditState::Degraded(_))
    }

    /// Returns the active segment identity.
    #[must_use]
    pub const fn segment(&self) -> &AuditSegmentId {
        &self.segment
    }

    /// Returns the number of records in the active segment.
    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.index
    }

    /// Appends a record and returns its MAC.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError`] when the log is degraded, when canonicalization
    /// fails, or when the record could not be persisted. A persistence failure
    /// degrades the log permanently for this process.
    pub fn append(&mut self, record: &AuditRecord, now: UnixMillis) -> Result<String, AuditError> {
        if let AuditState::Degraded(cause) = &self.state {
            return Err(AuditError::Degraded(describe_cause(cause)));
        }
        self.rotate_if_needed(now)?;

        let chained = self.prepare(record)?;
        let bytes = canonical_json(&chained).map_err(|_| AuditError::Canonicalize)?;
        match self.sink.write(&chained.segment, bytes.as_bytes()) {
            Ok(()) => {}
            Err(error) => {
                let cause = DegradationCause::WriteFailed(error.to_string());
                self.state = AuditState::Degraded(cause.clone());
                return Err(AuditError::Degraded(describe_cause(&cause)));
            }
        }
        self.previous_mac = decode_mac(&chained.mac).unwrap_or([0u8; MAC_LEN]);
        self.index = self.index.saturating_add(1);
        self.segment_bytes = self.segment_bytes.saturating_add(bytes.len());
        Ok(chained.mac)
    }

    fn prepare(&self, record: &AuditRecord) -> Result<ChainedRecord, AuditError> {
        let mut chained = ChainedRecord {
            schema_version: ba_domain::SCHEMA_VERSION,
            segment: self.segment.clone(),
            index: self.index,
            previous_mac: encode_hex(&self.previous_mac),
            record: record.clone(),
            mac: String::new(),
        };
        let bytes = canonical_json(&chained).map_err(|_| AuditError::Canonicalize)?;
        let mac = self.compute_mac(bytes.as_bytes())?;
        chained.mac = encode_hex(&mac);
        Ok(chained)
    }

    fn rotate_if_needed(&mut self, now: UnixMillis) -> Result<(), AuditError> {
        let day = now.get().div_euclid(86_400_000);
        let needs_rotation =
            day != self.segment_started_on_day || self.segment_bytes >= MAX_SEGMENT_BYTES;
        if !needs_rotation {
            return Ok(());
        }
        // The new segment carries the previous segment's final MAC so the chain
        // remains verifiable across the boundary.
        self.write_segment_boundary(now)?;
        self.segment = segment_for_day(day)?;
        self.segment_started_on_day = day;
        self.index = 0;
        self.segment_bytes = 0;
        Ok(())
    }

    fn write_segment_boundary(&mut self, now: UnixMillis) -> Result<(), AuditError> {
        let boundary = AuditRecord {
            task_id: None,
            action_id: None,
            action_class: None,
            target: None,
            policy_decision: None,
            execution_result: None,
            verification: None,
            browser_context: None,
            at_unix_ms: now,
            kind: AuditEventKind::Health {
                state: "SEGMENT_ROTATED".to_owned(),
                cause: format!("continuing from {}", self.segment.as_str()),
            },
        };
        let chained = self.prepare(&boundary)?;
        let bytes = canonical_json(&chained).map_err(|_| AuditError::Canonicalize)?;
        match self.sink.write(&chained.segment, bytes.as_bytes()) {
            Ok(()) => {
                self.previous_mac = decode_mac(&chained.mac).unwrap_or([0u8; MAC_LEN]);
                Ok(())
            }
            Err(error) => {
                let cause = DegradationCause::WriteFailed(error.to_string());
                self.state = AuditState::Degraded(cause.clone());
                Err(AuditError::Degraded(describe_cause(&cause)))
            }
        }
    }

    fn compute_mac(&self, bytes: &[u8]) -> Result<[u8; MAC_LEN], AuditError> {
        let mut mac =
            <HmacSha256 as Mac>::new_from_slice(&self.key).map_err(|_| AuditError::Canonicalize)?;
        mac.update(bytes);
        let output = mac.finalize().into_bytes();
        let mut out = [0u8; MAC_LEN];
        out.copy_from_slice(&output);
        Ok(out)
    }

    /// Verifies the retained chain.
    ///
    /// # Errors
    ///
    /// Returns [`AuditError`] when stored segments cannot be read.
    pub fn verify(&mut self) -> Result<IntegrityStatus, AuditError> {
        let segments = self.sink.read_segment(&self.segment)?;
        let mut previous = encode_hex(&[0u8; MAC_LEN]);
        for (index, bytes) in segments.iter().enumerate() {
            let parsed: ChainedRecord = match serde_json::from_slice(bytes) {
                Ok(parsed) => parsed,
                Err(_) => {
                    let status = IntegrityStatus::Broken {
                        segment: self.segment.as_str().to_owned(),
                        index,
                    };
                    self.state = AuditState::Degraded(DegradationCause::IntegrityFailure {
                        segment: self.segment.as_str().to_owned(),
                        index,
                    });
                    return Ok(status);
                }
            };
            if parsed.previous_mac != previous {
                let status = IntegrityStatus::Broken {
                    segment: parsed.segment.as_str().to_owned(),
                    index,
                };
                self.state = AuditState::Degraded(DegradationCause::IntegrityFailure {
                    segment: parsed.segment.as_str().to_owned(),
                    index,
                });
                return Ok(status);
            }
            let mut candidate = parsed.clone();
            let expected_mac = candidate.mac.clone();
            candidate.mac = String::new();
            let bytes = canonical_json(&candidate).map_err(|_| AuditError::Canonicalize)?;
            let mac = self.compute_mac(bytes.as_bytes())?;
            if encode_hex(&mac) != expected_mac {
                let status = IntegrityStatus::Broken {
                    segment: parsed.segment.as_str().to_owned(),
                    index,
                };
                self.state = AuditState::Degraded(DegradationCause::IntegrityFailure {
                    segment: parsed.segment.as_str().to_owned(),
                    index,
                });
                return Ok(status);
            }
            previous = expected_mac;
        }
        Ok(IntegrityStatus::Verified {
            records: segments.len(),
        })
    }

    /// Records that old segments were purged by explicit user action.
    ///
    /// After pruning the log reports verification for the retained range only,
    /// rather than claiming a complete chain over deleted records.
    pub fn mark_pruned(&mut self) {
        self.pruned = true;
    }

    /// Returns whether pruning has been recorded.
    #[must_use]
    pub const fn is_pruned(&self) -> bool {
        self.pruned
    }

    /// Returns the retention window, validated against the allowed set.
    #[must_use]
    pub fn retention_days(requested: u32) -> u32 {
        if ALLOWED_RETENTION_DAYS.contains(&requested) {
            requested
        } else {
            DEFAULT_RETENTION_DAYS
        }
    }
}

/// Builds the segment identity for a UTC day number.
///
/// The derived text is `segment-<day>`, which is always a valid identifier, so
/// the error path is unreachable in practice and returns a typed error rather
/// than panicking.
fn segment_for_day(day: i64) -> Result<AuditSegmentId, AuditError> {
    AuditSegmentId::parse(format!("segment-{day}")).map_err(|_| AuditError::Canonicalize)
}

fn describe_cause(cause: &DegradationCause) -> String {
    match cause {
        DegradationCause::MissingKey => "the audit key is unavailable".to_owned(),
        DegradationCause::WriteFailed(detail) => format!("a record could not be written: {detail}"),
        DegradationCause::IntegrityFailure { segment, index } => {
            format!("integrity check failed in {segment} at record {index}")
        }
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn decode_mac(hex: &str) -> Option<[u8; MAC_LEN]> {
    if hex.len() != MAC_LEN * 2 {
        return None;
    }
    let mut out = [0u8; MAC_LEN];
    let (pairs, _) = hex.as_bytes().as_chunks::<2>();
    for (index, pair) in pairs.iter().enumerate() {
        let text = core::str::from_utf8(pair).ok()?;
        out[index] = u8::from_str_radix(text, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> UnixMillis {
        UnixMillis::from_millis(1_700_000_000_000)
    }

    fn log(sink: MemorySink) -> AuditLog {
        AuditLog::open(&StaticKeySource::new([7u8; 32]), Box::new(sink), now()).unwrap()
    }

    fn record() -> AuditRecord {
        AuditRecord {
            task_id: Some(TaskId::parse("task-1").unwrap()),
            action_id: None,
            action_class: Some(ActionClass::Read),
            target: Some("https://example.com".to_owned()),
            policy_decision: Some("BA-POLICY-001.allowed".to_owned()),
            execution_result: None,
            verification: None,
            browser_context: Some(BrowserContextId::parse("ctx-normal").unwrap()),
            at_unix_ms: now(),
            kind: AuditEventKind::TaskTransition {
                state: "CREATED".to_owned(),
                wait_reason: None,
            },
        }
    }

    #[test]
    fn missing_key_is_fail_closed() {
        let error = AuditLog::open(&UnavailableKeySource, Box::new(MemorySink::new()), now());
        assert!(matches!(error, Err(AuditError::Key(_))));
    }

    #[test]
    fn records_chain_and_verify() {
        let mut audit = log(MemorySink::new());
        let first = audit.append(&record(), now()).unwrap();
        let second = audit.append(&record(), now()).unwrap();
        assert_ne!(first, second);
        assert_eq!(audit.record_count(), 2);
        let status = audit.verify().unwrap();
        assert_eq!(status, IntegrityStatus::Verified { records: 2 });
        assert!(status.label(false).starts_with("Integrity verified ("));
    }

    #[test]
    fn tampering_breaks_the_chain_and_degrades_the_log() {
        let mut inner = MemorySink::new();
        let segment = AuditSegmentId::parse("segment-0").unwrap();
        inner
            .write(
                &segment,
                b"{\"schema_version\":1,\"record\":{\"tampered\":true}}",
            )
            .unwrap();
        let mut audit = AuditLog::open(
            &StaticKeySource::new([7u8; 32]),
            Box::new(inner),
            UnixMillis::from_millis(0),
        )
        .unwrap();
        assert!(matches!(
            audit.verify().unwrap(),
            IntegrityStatus::Broken { .. }
        ));
        assert!(audit.is_degraded());
        assert!(matches!(
            audit.append(&record(), now()),
            Err(AuditError::Degraded(_))
        ));
        let broken = audit.verify().unwrap();
        assert!(!broken.label(false).starts_with("Integrity verified"));
    }

    #[test]
    fn a_write_failure_degrades_and_blocks_further_records() {
        let mut sink = MemorySink::new();
        sink.fail_next_write();
        let mut audit = log(sink);
        assert!(matches!(
            audit.append(&record(), now()),
            Err(AuditError::Degraded(_))
        ));
        assert!(audit.is_degraded());
        assert!(matches!(
            audit.append(&record(), now()),
            Err(AuditError::Degraded(_))
        ));
    }

    #[test]
    fn rotation_starts_a_new_segment_carrying_the_previous_mac() {
        let mut audit = log(MemorySink::new());
        audit.append(&record(), now()).unwrap();
        let first_segment = audit.segment().as_str().to_owned();
        let tomorrow = UnixMillis::from_millis(now().get() + 86_400_000);
        audit.append(&record(), tomorrow).unwrap();
        assert_ne!(audit.segment().as_str(), first_segment);
    }

    #[test]
    fn retention_windows_are_validated() {
        assert_eq!(AuditLog::retention_days(30), 30);
        assert_eq!(AuditLog::retention_days(365), 365);
        assert_eq!(AuditLog::retention_days(7), DEFAULT_RETENTION_DAYS);
    }

    #[test]
    fn pruning_changes_the_integrity_claim() {
        let mut audit = log(MemorySink::new());
        audit.append(&record(), now()).unwrap();
        let before = audit.verify().unwrap();
        audit.mark_pruned();
        assert!(before.label(audit.is_pruned()).contains("retained range"));
    }

    #[test]
    fn targets_are_redacted_before_storage() {
        let redacted = AuditRecord::redact_target("https://user:pw@example.com/x?token=abc");
        assert!(!redacted.contains("pw"));
        assert!(!redacted.contains("token=abc"));
        assert!(redacted.starts_with("https://example.com/x"));
    }

    #[test]
    fn macs_survive_hex_round_trips() {
        let bytes = [0xabu8; MAC_LEN];
        assert_eq!(decode_mac(&encode_hex(&bytes)), Some(bytes));
        assert_eq!(decode_mac("zz"), None);
    }
}
