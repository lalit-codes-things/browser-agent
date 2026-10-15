//! Reversibility capability records (BA-REV-001).
//!
//! A `REVERSIBLE_WRITE` classification requires a capability record that names
//! the exact operation, target type, preconditions, expected postcondition, the
//! compensation operation and its postcondition, known residual effects, an
//! expiry boundary and test evidence. A UI label such as “draft”, an assumed
//! website convention or the model's assertion is not evidence.

use ba_domain::{ActionKind, UnixMillis};

/// A capability record is unusable.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CapabilityError {
    /// No test evidence is attached.
    #[error("capability `{0}` has no test evidence and cannot justify a reversible write")]
    NoTestEvidence(String),
    /// No compensation operation is described.
    #[error("capability `{0}` names no compensation operation")]
    NoCompensation(String),
    /// The record may leave an external side effect behind.
    #[error("capability `{0}` has residual external effects: {1}")]
    ResidualExternalEffect(String, String),
    /// The record's expiry boundary has passed.
    #[error("capability `{0}` expired at {1}")]
    Expired(String, i64),
    /// The record covers no action kind.
    #[error("capability `{0}` covers no action kind")]
    NoActionKinds(String),
}

/// Evidenced proof that one bounded operation can be undone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityRecord {
    /// Stable identifier of the capability.
    pub id: String,
    /// Exact operation name, for example `field.clear`.
    pub operation: String,
    /// Action kinds this record authorizes.
    pub action_kinds: Vec<ActionKind>,
    /// Target kinds the operation is proven for, for example `textbox`.
    pub target_kinds: Vec<String>,
    /// Preconditions that must hold before use.
    pub preconditions: Vec<String>,
    /// Whether the record identifies a stable resource identity.
    pub stable_resource_identity: bool,
    /// Expected postcondition after the write.
    pub expected_postcondition: String,
    /// Compensation operation that restores the prior state.
    pub compensation_operation: String,
    /// Expected postcondition after compensation.
    pub compensation_postcondition: String,
    /// Residual effects that remain after compensation. Must be empty for a
    /// `REVERSIBLE_WRITE` classification.
    pub residual_effects: Vec<String>,
    /// Expiry boundary for this evidence.
    pub expires_at_unix_ms: UnixMillis,
    /// Identifier of the executed test that produced this evidence.
    pub test_evidence_id: String,
}

impl CapabilityRecord {
    /// Returns whether this record covers `kind`.
    #[must_use]
    pub fn matches_action(&self, kind: ActionKind) -> bool {
        self.action_kinds.contains(&kind)
    }

    /// Returns whether this record covers `kind` for `target_kind`.
    #[must_use]
    pub fn matches(&self, kind: ActionKind, target_kind: &str) -> bool {
        self.matches_action(kind)
            && self
                .target_kinds
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(target_kind))
    }

    /// Validates the record's own integrity and expiry.
    ///
    /// # Errors
    ///
    /// Returns [`CapabilityError`] for any violation. A failure here downgrades
    /// the action to `UNKNOWN`, never to `REVERSIBLE_WRITE`.
    pub fn check_valid_at(&self, now: UnixMillis) -> Result<(), CapabilityError> {
        if self.test_evidence_id.trim().is_empty() {
            return Err(CapabilityError::NoTestEvidence(self.id.clone()));
        }
        if self.compensation_operation.trim().is_empty() {
            return Err(CapabilityError::NoCompensation(self.id.clone()));
        }
        if let Some(residual) = self.residual_effects.first() {
            return Err(CapabilityError::ResidualExternalEffect(
                self.id.clone(),
                residual.clone(),
            ));
        }
        if self.action_kinds.is_empty() {
            return Err(CapabilityError::NoActionKinds(self.id.clone()));
        }
        if now.get() >= self.expires_at_unix_ms.get() {
            return Err(CapabilityError::Expired(
                self.id.clone(),
                self.expires_at_unix_ms.get(),
            ));
        }
        Ok(())
    }
}

/// The set of capability records the runtime may use.
#[derive(Debug, Default, Clone)]
pub struct CapabilityRegistry {
    records: Vec<CapabilityRecord>,
}

/// Expiry used by the built-in, test-evidence-backed field capabilities.
const BUILTIN_EXPIRY: i64 = 4_102_444_800_000; // 2100-01-01T00:00:00Z

impl CapabilityRegistry {
    /// Creates an empty registry. Used by tests that must prove the
    /// fail-closed path.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Creates the registry with the built-in, evidenced capabilities.
    ///
    /// Each built-in record corresponds to a deterministic operation the
    /// executor implements (`field.clear`, `control.restore_selection`), so the
    /// compensation is a real operation rather than a promise.
    #[must_use]
    pub fn builtin() -> Self {
        Self {
            records: vec![
                CapabilityRecord {
                    id: "builtin.field-clear".to_owned(),
                    operation: "field.clear".to_owned(),
                    action_kinds: vec![ActionKind::Type],
                    target_kinds: vec!["textbox".to_owned(), "searchbox".to_owned()],
                    preconditions: vec![
                        "field is focused in the observed frame".to_owned(),
                        "field value before the write was recorded in the same epoch".to_owned(),
                    ],
                    stable_resource_identity: true,
                    expected_postcondition: "field contains the intended value".to_owned(),
                    compensation_operation: "field.clear".to_owned(),
                    compensation_postcondition: "field is empty as recorded before the write"
                        .to_owned(),
                    residual_effects: Vec::new(),
                    expires_at_unix_ms: UnixMillis::from_millis(BUILTIN_EXPIRY),
                    test_evidence_id: "crates/ba-executor/tests/field_clear.rs".to_owned(),
                },
                CapabilityRecord {
                    id: "builtin.control-restore-selection".to_owned(),
                    operation: "control.restore_selection".to_owned(),
                    action_kinds: vec![ActionKind::Select],
                    target_kinds: vec!["combobox".to_owned(), "listbox".to_owned()],
                    preconditions: vec![
                        "control exposes its previous selection in the same epoch".to_owned(),
                    ],
                    stable_resource_identity: true,
                    expected_postcondition: "control reports the intended option".to_owned(),
                    compensation_operation: "control.restore_selection".to_owned(),
                    compensation_postcondition: "control reports the previously selected option"
                        .to_owned(),
                    residual_effects: Vec::new(),
                    expires_at_unix_ms: UnixMillis::from_millis(BUILTIN_EXPIRY),
                    test_evidence_id: "crates/ba-executor/tests/restore_selection.rs".to_owned(),
                },
            ],
        }
    }

    /// Adds a reviewed record.
    ///
    /// # Errors
    ///
    /// Returns [`CapabilityError`] when the record is not usable at `now`.
    pub fn insert(
        &mut self,
        record: CapabilityRecord,
        now: UnixMillis,
    ) -> Result<(), CapabilityError> {
        record.check_valid_at(now)?;
        self.records.push(record);
        Ok(())
    }

    /// Removes a record by identifier, returning whether it existed.
    pub fn revoke(&mut self, id: &str) -> bool {
        let before = self.records.len();
        self.records.retain(|record| record.id != id);
        before != self.records.len()
    }

    /// Returns the record that covers `kind` and `target_kind`.
    #[must_use]
    pub fn select(&self, kind: ActionKind, target_kind: &str) -> Option<&CapabilityRecord> {
        self.records
            .iter()
            .find(|record| record.matches(kind, target_kind))
    }

    /// Returns every record, in insertion order.
    #[must_use]
    pub fn records(&self) -> &[CapabilityRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> UnixMillis {
        UnixMillis::from_millis(1_700_000_000_000)
    }

    #[test]
    fn builtin_records_are_valid_and_selectable() {
        let registry = CapabilityRegistry::builtin();
        for record in registry.records() {
            assert_eq!(record.check_valid_at(now()), Ok(()));
        }
        let selected = registry.select(ActionKind::Type, "textbox");
        assert!(selected.is_some());
        assert_eq!(selected.map(|r| r.operation.as_str()), Some("field.clear"));
        assert!(registry.select(ActionKind::Click, "textbox").is_none());
    }

    #[test]
    fn records_without_evidence_are_rejected() {
        let registry = CapabilityRegistry::empty();
        assert!(registry.select(ActionKind::Type, "textbox").is_none());

        let mut record = CapabilityRegistry::builtin().records()[0].clone();
        record.test_evidence_id = String::new();
        assert!(matches!(
            record.check_valid_at(now()),
            Err(CapabilityError::NoTestEvidence(_))
        ));
    }

    #[test]
    fn residual_external_effects_disqualify_a_record() {
        let mut record = CapabilityRegistry::builtin().records()[0].clone();
        record.residual_effects = vec!["a notification email is sent".to_owned()];
        assert!(matches!(
            record.check_valid_at(now()),
            Err(CapabilityError::ResidualExternalEffect(_, _))
        ));
    }

    #[test]
    fn expired_records_are_rejected() {
        let mut record = CapabilityRegistry::builtin().records()[0].clone();
        record.expires_at_unix_ms = UnixMillis::from_millis(now().get() - 1);
        assert!(matches!(
            record.check_valid_at(now()),
            Err(CapabilityError::Expired(_, _))
        ));
    }

    #[test]
    fn revoking_a_record_removes_it() {
        let mut registry = CapabilityRegistry::builtin();
        assert!(registry.revoke("builtin.field-clear"));
        assert!(!registry.revoke("builtin.field-clear"));
        assert!(registry.select(ActionKind::Type, "textbox").is_none());
    }
}
