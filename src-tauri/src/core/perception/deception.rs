// Deception detection.
//
// C-86: accessible-name vs rendered-text disagreement -> UNKNOWN.
//        The node is not actionable until the disagreement is resolved
//        by a higher-fidelity perception rung.
//
// C-82 Unicode defenses (zero-width/bidi/homoglyph) live in unicode.rs.

use crate::core::perception::graph::GraphNode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeceptionVerdict {
    /// Names agree (or are trivially consistent). Node may proceed to
    /// actionability.
    Consistent,
    /// Accessible name and rendered text disagree: UNKNOWN until resolved.
    NameDisagreement,
    /// Display string contains identity-hostile characters.
    SuspiciousText,
}

pub fn assess(node: &GraphNode) -> DeceptionVerdict {
    // Zero-width / bidi controls anywhere in either string.
    let suspicious = [&node.label, &node.rendered_text, &node.accessible_name]
        .into_iter()
        .flatten()
        .any(|s| {
            crate::core::perception::unicode::contains_zero_width(s)
                || crate::core::perception::unicode::contains_bidi_controls(s)
                || crate::core::perception::unicode::looks_mixed_script(s)
        });
    if suspicious {
        return DeceptionVerdict::SuspiciousText;
    }

    match (&node.accessible_name, &node.rendered_text) {
        (Some(ax), Some(rendered)) if ax != rendered => DeceptionVerdict::NameDisagreement,
        _ => DeceptionVerdict::Consistent,
    }
}

/// Gate used by actionability filtering: a node with an unresolved name
/// disagreement is not clickable until re-perceived at higher fidelity.
pub fn is_action_safe(node: &GraphNode) -> bool {
    assess(node) == DeceptionVerdict::Consistent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::perception::graph::GeometryBounds;

    fn node(ax: Option<&str>, rendered: Option<&str>) -> GraphNode {
        GraphNode {
            role: "button".into(),
            label: None,
            rendered_text: rendered.map(|s| s.into()),
            accessible_name: ax.map(|s| s.into()),
            actionable: true,
            bounds: Some(GeometryBounds {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            }),
        }
    }

    #[test]
    fn consistent_names() {
        assert_eq!(
            assess(&node(Some("Pay"), Some("Pay"))),
            DeceptionVerdict::Consistent
        );
    }

    #[test]
    fn name_disagreement_is_unknown_not_clickable() {
        let n = node(Some("Pay"), Some("Download malware"));
        assert_eq!(assess(&n), DeceptionVerdict::NameDisagreement);
        assert!(!is_action_safe(&n));
    }

    #[test]
    fn zero_width_in_rendered_text_is_suspicious() {
        let n = node(Some("Pay"), Some("Pa\u{200B}y"));
        assert_eq!(assess(&n), DeceptionVerdict::SuspiciousText);
        assert!(!is_action_safe(&n));
    }

    #[test]
    fn mixed_script_is_suspicious() {
        // Cyrillic 'а' inside a Latin word.
        let n = node(Some("p\u{0430}y"), Some("p\u{0430}y"));
        assert_eq!(assess(&n), DeceptionVerdict::SuspiciousText);
    }

    #[test]
    fn missing_names_are_consistent_not_fabricated() {
        // Absence of both names is not a disagreement; higher rungs
        // decide actionability.
        assert_eq!(assess(&node(None, None)), DeceptionVerdict::Consistent);
    }
}
