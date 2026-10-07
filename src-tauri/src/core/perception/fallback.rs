// Perception fallback ladder.
//
// C-84: when the preferred perception path fails or is insufficient, a
//        defined fallback ladder is used. Each rung is explicit; the UI
//        can surface the active rung. The ladder never silently pretends
//        full-fidelity perception.
//
// Ordering is specification authority: A11Y first, then DOM, then
// geometry-only, then manual takeover. Cost/coverage trade-offs live in
// the ADR for perception (docs/adr).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum PerceptionRung {
    /// Accessibility-tree extraction (preferred).
    A11y,
    /// DOM extraction.
    Dom,
    /// Geometry/layout only; no semantic names available.
    GeometryOnly,
    /// Human takeover. Terminal rung: no automated perception is claimed.
    ManualTakeover,
}

impl PerceptionRung {
    /// Next rung when the current one fails. Manual takeover is terminal.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::A11y => Some(Self::Dom),
            Self::Dom => Some(Self::GeometryOnly),
            Self::GeometryOnly => Some(Self::ManualTakeover),
            Self::ManualTakeover => None,
        }
    }

    /// True when the rung can produce semantic references (C-85).
    /// GeometryOnly cannot name targets; actions against it are limited.
    pub fn produces_semantic_references(self) -> bool {
        matches!(self, Self::A11y | Self::Dom)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackState {
    pub active_rung: PerceptionRung,
    /// Backend-provided reason the previous rung was abandoned.
    pub reason: Option<String>,
}

impl FallbackState {
    pub fn initial() -> Self {
        Self { active_rung: PerceptionRung::A11y, reason: None }
    }

    pub fn descend(&mut self, reason: String) -> Result<PerceptionRung, crate::Error> {
        let next = self
            .active_rung
            .next()
            .ok_or_else(|| crate::Error::StateMismatch("fallback ladder exhausted; manual takeover required".into()))?;
        self.active_rung = next;
        self.reason = Some(reason);
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_order_is_spec_ordered() {
        let mut state = FallbackState::initial();
        assert_eq!(state.active_rung, PerceptionRung::A11y);

        let r1 = state.descend("a11y extraction failed".into()).unwrap();
        assert_eq!(r1, PerceptionRung::Dom);

        let r2 = state.descend("dom extraction failed".into()).unwrap();
        assert_eq!(r2, PerceptionRung::GeometryOnly);

        let r3 = state.descend("geometry insufficient".into()).unwrap();
        assert_eq!(r3, PerceptionRung::ManualTakeover);

        // Terminal: descending past manual takeover is an error, not a loop.
        assert!(state.descend("again".into()).is_err());
    }

    #[test]
    fn geometry_only_does_not_claim_semantics() {
        assert!(PerceptionRung::A11y.produces_semantic_references());
        assert!(PerceptionRung::Dom.produces_semantic_references());
        assert!(!PerceptionRung::GeometryOnly.produces_semantic_references());
        assert!(!PerceptionRung::ManualTakeover.produces_semantic_references());
    }

    #[test]
    fn reason_is_preserved_for_ui() {
        let mut state = FallbackState::initial();
        state.descend("ax tree unavailable".into()).unwrap();
        assert_eq!(state.reason.as_deref(), Some("ax tree unavailable"));
    }
}
