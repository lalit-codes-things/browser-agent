//! Versioned event envelopes and the authoritative task event (Section 20.2,
//! BA-STATE-001/002).

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::copy::task_state_label;
use crate::ids::{ActionId, Revision, Sequence, TaskId};
use crate::task::{ActionClass, TaskState, VerificationOutcome, WaitReason};

/// Wall-clock instant in milliseconds since the Unix epoch (UTC).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixMillis(i64);

/// The system clock could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ClockError {
    /// The reported time was before the Unix epoch.
    #[error("system clock is before the Unix epoch")]
    BeforeEpoch,
}

impl UnixMillis {
    /// Wraps a raw millisecond value.
    #[must_use]
    pub const fn from_millis(value: i64) -> Self {
        Self(value)
    }

    /// Reads the wall clock.
    ///
    /// # Errors
    ///
    /// Returns [`ClockError`] when the system clock is set before the epoch.
    /// The caller must not substitute a fabricated timestamp.
    pub fn now() -> Result<Self, ClockError> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| ClockError::BeforeEpoch)?;
        i64::try_from(duration.as_millis())
            .map(Self)
            .map_err(|_| ClockError::BeforeEpoch)
    }

    /// Returns the raw millisecond value.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// Renders an RFC 3339 UTC timestamp without a fractional part.
    ///
    /// Implemented locally so the trusted core has no date-library dependency
    /// and timestamps stay deterministic and testable.
    #[must_use]
    pub fn to_rfc3339_utc(self) -> String {
        let millis = self.0;
        let days = millis.div_euclid(86_400_000);
        let rem = millis.rem_euclid(86_400_000);
        let seconds_of_day = rem / 1_000;
        let (hour, minute, second) = (
            seconds_of_day / 3_600,
            (seconds_of_day % 3_600) / 60,
            seconds_of_day % 60,
        );
        let (year, month, day) = civil_from_days(days);
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }
}

/// Converts days since the Unix epoch to a proleptic Gregorian date.
///
/// This is Howard Hinnant's `civil_from_days` algorithm, which is exact for the
/// full `i64` day range we accept and has no lookup tables.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (
        year,
        u32::try_from(m).unwrap_or(1),
        u32::try_from(d).unwrap_or(1),
    )
}

/// One budget dimension: ceiling and consumed amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetField {
    /// Current ceiling, including any approved 50% extension.
    pub limit: u64,
    /// Amount consumed so far.
    pub used: u64,
}

impl BudgetField {
    /// Returns the remaining amount, saturating at zero.
    #[must_use]
    pub const fn remaining(self) -> u64 {
        self.limit.saturating_sub(self.used)
    }

    /// Returns whether the ceiling is reached or exceeded.
    #[must_use]
    pub const fn exhausted(self) -> bool {
        self.used >= self.limit
    }

    /// Consumes `amount` if it fits.
    ///
    /// # Errors
    ///
    /// Returns `false` when the ceiling would be exceeded; the caller must not
    /// treat exhaustion as authorization to continue.
    pub fn try_consume(&mut self, amount: u64) -> bool {
        match self.used.checked_add(amount) {
            Some(next) if next <= self.limit => {
                self.used = next;
                true
            }
            _ => false,
        }
    }
}

/// Task budget snapshot shown in the Agent panel (BA-BUDGET-001).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskBudgetView {
    /// Policy-evaluated action proposals.
    pub action_proposals: BudgetField,
    /// Dispatched actions.
    pub dispatched_actions: BudgetField,
    /// Agent-initiated top-level navigations.
    pub navigations: BudgetField,
    /// Cumulative model input tokens.
    pub input_tokens: BudgetField,
    /// Cumulative model output tokens.
    pub output_tokens: BudgetField,
    /// Active execution time in milliseconds.
    pub active_ms: BudgetField,
    /// Wall-clock time spent waiting or paused, shown separately.
    pub waiting_ms: u64,
    /// Number of 50% extensions already approved for this task.
    pub extensions_used: u8,
}

/// An authoritative task event produced by the Rust orchestrator.
///
/// `label` is not free text: deserialization rejects any value that does not
/// equal the canonical table entry for `state`/`wait_reason`, so a compromised
/// or confused producer cannot inject arbitrary status copy into app-owned UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TaskEventWire")]
pub struct TaskEvent {
    /// Owning task.
    pub task_id: TaskId,
    /// Monotonic task revision.
    pub revision: Revision,
    /// Monotonic process-wide event sequence.
    pub sequence: Sequence,
    /// Lifecycle state.
    pub state: TaskState,
    /// Exactly one typed reason while `state` is `WAITING_FOR_USER`.
    pub wait_reason: Option<WaitReason>,
    /// Action this event refers to, when applicable.
    pub action_id: Option<ActionId>,
    /// Policy result for the action, when known.
    pub action_class: Option<ActionClass>,
    /// Verification outcome. Absent until an outcome evaluation is applicable.
    pub verification: Option<VerificationOutcome>,
    /// Canonical user-facing status label.
    pub label: String,
    /// Plain-language detail rendered from trusted runtime fields.
    pub detail: String,
    /// UTC wall-clock timestamp.
    pub at_unix_ms: UnixMillis,
    /// Monotonic timestamp for elapsed measurements.
    pub monotonic_ms: u64,
    /// Active execution time accumulated so far.
    pub elapsed_active_ms: u64,
    /// Waiting/paused wall time accumulated so far.
    pub elapsed_waiting_ms: u64,
    /// Budget snapshot while a task is active.
    pub budget: Option<TaskBudgetView>,
}

/// Schema violations in a task event.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EventValidationError {
    /// `WAITING_FOR_USER` was emitted without a wait reason (BA-STATE-002).
    #[error("WAITING_FOR_USER requires exactly one wait reason")]
    WaitReasonRequired,
    /// A wait reason was attached to a state that does not accept one.
    #[error("state {0} must not carry a wait reason")]
    WaitReasonNotAllowed(TaskState),
    /// The label did not match the canonical copy table.
    #[error("status label does not match the canonical table for {0}")]
    LabelMismatch(TaskState),
    /// The detail text was too long or contained control characters.
    #[error("event detail is not renderable: {0}")]
    DetailInvalid(String),
    /// `BLOCKED` must not claim a verification outcome.
    #[error("BLOCKED must report `Verification not run`, not an outcome")]
    VerificationNotAllowedWhenBlocked,
}

impl TaskEvent {
    /// Derives the canonical label for the event's state and wait reason.
    ///
    /// # Errors
    ///
    /// Returns [`EventValidationError`] when the state/reason combination has no
    /// canonical entry.
    pub fn canonical_label(&self) -> Result<&'static str, EventValidationError> {
        task_state_label(self.state, self.wait_reason)
            .ok_or(EventValidationError::LabelMismatch(self.state))
    }

    /// Validates the event against the lifecycle and copy contracts.
    ///
    /// # Errors
    ///
    /// Returns [`EventValidationError`] on any violation. Callers must fail
    /// closed rather than rendering a partially valid event.
    pub fn validate(&self) -> Result<(), EventValidationError> {
        match (self.state, self.wait_reason) {
            (TaskState::WaitingForUser, None) => {
                return Err(EventValidationError::WaitReasonRequired);
            }
            (TaskState::WaitingForUser, Some(_)) => {}
            (_, Some(_)) => return Err(EventValidationError::WaitReasonNotAllowed(self.state)),
            (_, None) => {}
        }
        if self.state == TaskState::Blocked && self.verification.is_some() {
            return Err(EventValidationError::VerificationNotAllowedWhenBlocked);
        }
        if self.label != self.canonical_label()? {
            return Err(EventValidationError::LabelMismatch(self.state));
        }
        if self.detail.chars().count() > 512
            || self
                .detail
                .chars()
                .any(|c| c == '\0' || (c.is_control() && c != '\n'))
        {
            return Err(EventValidationError::DetailInvalid(self.detail.clone()));
        }
        Ok(())
    }

    /// Returns whether this event terminates the task.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskEventWire {
    task_id: TaskId,
    revision: Revision,
    sequence: Sequence,
    state: TaskState,
    wait_reason: Option<WaitReason>,
    action_id: Option<ActionId>,
    action_class: Option<ActionClass>,
    verification: Option<VerificationOutcome>,
    label: String,
    detail: String,
    at_unix_ms: UnixMillis,
    monotonic_ms: u64,
    elapsed_active_ms: u64,
    elapsed_waiting_ms: u64,
    budget: Option<TaskBudgetView>,
}

impl TryFrom<TaskEventWire> for TaskEvent {
    type Error = EventValidationError;

    fn try_from(wire: TaskEventWire) -> Result<Self, Self::Error> {
        let event = Self {
            task_id: wire.task_id,
            revision: wire.revision,
            sequence: wire.sequence,
            state: wire.state,
            wait_reason: wire.wait_reason,
            action_id: wire.action_id,
            action_class: wire.action_class,
            verification: wire.verification,
            label: wire.label,
            detail: wire.detail,
            at_unix_ms: wire.at_unix_ms,
            monotonic_ms: wire.monotonic_ms,
            elapsed_active_ms: wire.elapsed_active_ms,
            elapsed_waiting_ms: wire.elapsed_waiting_ms,
            budget: wire.budget,
        };
        event.validate()?;
        Ok(event)
    }
}

/// A versioned envelope around any boundary payload (Section 20.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope<P> {
    /// Wire schema version; must equal [`crate::SCHEMA_VERSION`].
    pub schema_version: u16,
    /// Monotonic event sequence.
    pub sequence: Sequence,
    /// UTC emission time.
    pub at_unix_ms: UnixMillis,
    /// Monotonic emission time for latency measurement.
    pub monotonic_ms: u64,
    /// Task identity when the payload belongs to a task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    /// Task revision when the payload belongs to a task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<Revision>,
    /// Payload.
    pub payload: P,
}

impl<P> Envelope<P> {
    /// Wraps `payload` in an envelope with the current schema version.
    #[must_use]
    pub const fn new(
        sequence: Sequence,
        at_unix_ms: UnixMillis,
        monotonic_ms: u64,
        task_id: Option<TaskId>,
        revision: Option<Revision>,
        payload: P,
    ) -> Self {
        Self {
            schema_version: crate::SCHEMA_VERSION,
            sequence,
            at_unix_ms,
            monotonic_ms,
            task_id,
            revision,
            payload,
        }
    }
}

/// Serializes `value` to a deterministic byte form for integrity metadata.
///
/// Object keys are emitted in sorted order, so the same logical value always
/// produces the same bytes (BA-AUDIT-001).
///
/// # Errors
///
/// Returns the underlying [`serde_json::Error`] when the value is not
/// serializable; the caller must fail closed rather than hash a partial form.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    serde_json::to_string(&value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::copy::task_state_label;

    fn event(state: TaskState, wait_reason: Option<WaitReason>) -> TaskEvent {
        // Waiting without a reason intentionally has no canonical label; these
        // tests set the label explicitly to exercise the rejection path.
        let label = task_state_label(state, wait_reason)
            .unwrap_or_default()
            .to_owned();
        TaskEvent {
            task_id: TaskId::parse("task-1").unwrap(),
            revision: Revision::new(1),
            sequence: Sequence::new(1),
            state,
            wait_reason,
            action_id: None,
            action_class: None,
            verification: None,
            label,
            detail: String::new(),
            at_unix_ms: UnixMillis::from_millis(1_700_000_000_000),
            monotonic_ms: 0,
            elapsed_active_ms: 0,
            elapsed_waiting_ms: 0,
            budget: None,
        }
    }

    #[test]
    fn waiting_for_user_requires_exactly_one_reason() {
        let mut invalid = event(TaskState::WaitingForUser, None);
        invalid.label = task_state_label(TaskState::WaitingForUser, Some(WaitReason::UserInput))
            .unwrap()
            .to_owned();
        assert_eq!(
            invalid.validate(),
            Err(EventValidationError::WaitReasonRequired)
        );
    }

    #[test]
    fn wait_reason_is_rejected_outside_waiting_state() {
        let mut invalid = event(TaskState::Observing, Some(WaitReason::UserInput));
        invalid.label = task_state_label(TaskState::Observing, None)
            .unwrap_or_default()
            .to_owned();
        assert_eq!(
            invalid.validate(),
            Err(EventValidationError::WaitReasonNotAllowed(
                TaskState::Observing
            ))
        );
    }

    #[test]
    fn labels_must_come_from_the_canonical_table() {
        let mut invented = event(TaskState::Observing, None);
        invented.label = "Thinking hard".to_owned();
        assert_eq!(
            invented.validate(),
            Err(EventValidationError::LabelMismatch(TaskState::Observing))
        );
    }

    #[test]
    fn deserialization_reenforces_the_copy_contract() {
        let valid = event(TaskState::Executing, None);
        let json = serde_json::to_string(&valid).unwrap();
        assert!(serde_json::from_str::<TaskEvent>(&json).is_ok());

        let tampered = json.replace("Performing action", "Almost done");
        assert!(serde_json::from_str::<TaskEvent>(&tampered).is_err());
    }

    #[test]
    fn deserialization_rejects_unknown_fields() {
        let valid = event(TaskState::Created, None);
        let json = serde_json::to_string(&valid).unwrap();
        let tampered = json.replace('{', r#"{"authorized":true,"#);
        assert!(serde_json::from_str::<TaskEvent>(&tampered).is_err());
    }

    #[test]
    fn blocked_must_not_claim_a_verification_outcome() {
        let mut invalid = event(TaskState::Blocked, None);
        invalid.verification = Some(VerificationOutcome::VerifiedSuccess);
        assert_eq!(
            invalid.validate(),
            Err(EventValidationError::VerificationNotAllowedWhenBlocked)
        );
    }

    #[test]
    fn budget_fields_saturate_instead_of_going_negative() {
        let mut field = BudgetField { limit: 2, used: 3 };
        assert_eq!(field.remaining(), 0);
        assert!(field.exhausted());
        assert!(!field.try_consume(1));
        field.used = 0;
        assert!(field.try_consume(2));
        assert!(!field.try_consume(1));
    }

    #[test]
    fn canonical_json_sorts_keys() {
        let value = serde_json::json!({"b": 1, "a": {"d": 2, "c": 3}});
        assert_eq!(
            canonical_json(&value).unwrap(),
            r#"{"a":{"c":3,"d":2},"b":1}"#
        );
    }

    #[test]
    fn rfc3339_rendering_is_utc_and_exact() {
        assert_eq!(
            UnixMillis::from_millis(0).to_rfc3339_utc(),
            "1970-01-01T00:00:00Z"
        );
        assert_eq!(
            UnixMillis::from_millis(1_700_000_000_000).to_rfc3339_utc(),
            "2023-11-14T22:13:20Z"
        );
        assert_eq!(
            UnixMillis::from_millis(1_767_225_600_000).to_rfc3339_utc(),
            "2026-01-01T00:00:00Z"
        );
    }
}
