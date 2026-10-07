// Mutation noise classifier.
//
// C-81: mutation-noise suppression. C-65: Phase-1 controlled mutations.
//
// Noisy mutations (timestamps, tickers, spinner frames, transient hover
// states) must not trigger epoch changes or re-perception. The epoch is
// advanced only on relevant mutations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MutationClass {
    /// Changes the semantic state graph (target appeared/disappeared,
    /// role/label/text/bounds changed, frame/loader changed).
    Relevant,
    /// Cosmetic or high-frequency churn that must not invalidate epochs.
    Noise,
    /// Cannot be classified with available evidence. Treated as
    /// Relevant (fail-closed): an unclassified mutation invalidates
    /// the epoch rather than risking a stale action.
    Uncertain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationObservation {
    pub frame_id: String,
    /// ARIA role of the mutated node, when extractable.
    pub role: Option<String>,
    /// Accessible name / rendered text after the mutation, when extractable.
    pub text_changed: bool,
    pub bounds_changed: bool,
    pub node_added: bool,
    pub node_removed: bool,
    /// Attribute-level churn candidates: style, class, aria-busy, data-* tickers.
    pub cosmetic_attribute_only: bool,
}

pub fn classify(observation: &MutationObservation) -> MutationClass {
    if observation.node_added || observation.node_removed {
        return MutationClass::Relevant;
    }
    if observation.text_changed || observation.bounds_changed {
        return MutationClass::Relevant;
    }
    if observation.cosmetic_attribute_only {
        return MutationClass::Noise;
    }
    MutationClass::Uncertain
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs() -> MutationObservation {
        MutationObservation {
            frame_id: "F1".into(),
            role: Some("button".into()),
            text_changed: false,
            bounds_changed: false,
            node_added: false,
            node_removed: false,
            cosmetic_attribute_only: false,
        }
    }

    #[test]
    fn cosmetic_only_is_noise() {
        let mut o = obs();
        o.cosmetic_attribute_only = true;
        assert_eq!(classify(&o), MutationClass::Noise);
    }

    #[test]
    fn text_change_is_relevant() {
        let mut o = obs();
        o.text_changed = true;
        assert_eq!(classify(&o), MutationClass::Relevant);
    }

    #[test]
    fn bounds_change_is_relevant() {
        let mut o = obs();
        o.bounds_changed = true;
        assert_eq!(classify(&o), MutationClass::Relevant);
    }

    #[test]
    fn unclassifiable_is_fail_closed_relevant() {
        // Nothing known about the change: epoch invalidation is the safe side.
        assert_eq!(classify(&obs()), MutationClass::Uncertain);
        assert_ne!(classify(&obs()), MutationClass::Noise);
    }

    #[test]
    fn node_removal_is_relevant() {
        let mut o = obs();
        o.node_removed = true;
        assert_eq!(classify(&o), MutationClass::Relevant);
    }
}
