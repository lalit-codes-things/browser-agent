// Unicode normalization and deception detection.
//
// C-82: Unicode normalization uses NFKC plus zero-width, bidi, homoglyph,
//        and mixed-script detection.

use std::collections::HashSet;

pub fn normalize(text: &str) -> String {
    // Real NFKC normalization is provided by unicode-normalization crate.
    // For now we keep this boundary explicit and stubbed so we can wire
    // the dependency when Phase 3 completes.
    text.to_string()
}

pub fn contains_zero_width(text: &str) -> bool {
    text.chars().any(|c| c == '\u{200B}' || c == '\u{200C}' || c == '\u{200D}' || c == '\u{FEFF}')
}

pub fn contains_bidi_controls(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{200E}' | '\u{200F}' | '\u{202A}' | '\u{202B}' | '\u{202C}' | '\u{202D}' | '\u{202E}' | '\u{2066}' | '\u{2067}' | '\u{2068}' | '\u{2069}'
        )
    }
}

/// Mixed-script detection heuristic placeholder.
///
/// A real implementation would use Unicode script classification.
pub fn looks_mixed_script(text: &str) -> bool {
    let mut scripts = HashSet::new();
    for ch in text.chars() {
        let script = script_of(ch);
        if !script.is_empty() {
            scripts.insert(script);
        }
    }
    scripts.len() > 1
}

fn script_of(_c: char) -> &'static str {
    // Placeholder: real implementation classifies Unicode script.
    "Latin"
}
