// Unicode normalization and deception detection.
//
// C-82: Unicode normalization uses NFKC plus zero-width, bidi, homoglyph,
//        and mixed-script detection.
//
// The mixed-script detector is deliberately conservative: for characters
// outside the Common/Latin ranges we cannot yet classify, the detector
// reports "cannot exclude mixed script" rather than asserting a single
// script. A false positive routes to UNKNOWN handling; a false negative
// would defeat the homoglyph defense.

use std::collections::HashSet;

use unicode_normalization::UnicodeNormalization;

/// NFKC normalization (C-82). Applied to all page-derived display strings
/// before comparison or storage.
pub fn normalize(text: &str) -> String {
    text.nfkc().collect()
}

pub fn contains_zero_width(text: &str) -> bool {
    text.chars()
        .any(|c| matches!(c, '\u{200B}'..='\u{200D}' | '\u{FEFF}' | '\u{2060}'))
}

pub fn contains_bidi_controls(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{200E}' | '\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2066}'..='\u{2069}'
        )
    })
}

/// Conservative mixed-script detection.
///
/// Returns true when the string mixes distinct detected scripts, or when
/// it contains non-ASCII characters we cannot classify. Never fabricates
/// a script identity we cannot verify.
pub fn looks_mixed_script(text: &str) -> bool {
    let mut scripts: HashSet<&'static str> = HashSet::new();
    let mut unclassifiable = false;

    for ch in text.chars() {
        match script_of(ch) {
            // Script-Common (spaces, digits, ASCII punctuation) is
            // script-neutral under Unicode and never counts as mixing.
            Some("Common") => {}
            Some(script) => {
                scripts.insert(script);
            }
            None => {
                unclassifiable = true;
            }
        }
    }

    scripts.len() > 1
        || (unclassifiable && !scripts.is_empty())
        || (unclassifiable && !text.is_ascii())
}

/// Unicode script classification for the ranges perception actually
/// encounters in controlled mock environments and common real sites.
/// Returns None for characters we cannot classify (treated as unknown,
/// never silently as Latin).
fn script_of(c: char) -> Option<&'static str> {
    let cp = c as u32;
    match c {
        'a'..='z' | 'A'..='Z' => Some("Latin"),
        _ if (0x00C0..=0x024F).contains(&cp) => Some("Latin"), // Latin extended
        _ if (0x0370..=0x03FF).contains(&cp) => Some("Greek"),
        _ if (0x0400..=0x04FF).contains(&cp) => Some("Cyrillic"),
        _ if (0x0590..=0x05FF).contains(&cp) => Some("Hebrew"),
        _ if (0x0600..=0x06FF).contains(&cp) => Some("Arabic"),
        _ if (0x4E00..=0x9FFF).contains(&cp) => Some("Han"),
        _ if (0x3040..=0x30FF).contains(&cp) => Some("Kana"),
        _ if (0xAC00..=0xD7AF).contains(&cp) => Some("Hangul"),
        '0'..='9' => Some("Common"),
        ' ' | '\t' | '\n' | '\r' => Some("Common"),
        // ASCII punctuation and symbols are script-Common.
        _ if cp < 0x7F => Some("Common"),
        // Anything else: unclassifiable here (Math symbols, fullwidth
        // forms, rare scripts, ...). Unknown beats wrong.
        _ => None,
    }
}

/// Homoglyph-risk heuristic: normalization + script analysis flag.
///
/// True when NFKC changed the string or any bidi/zero-width controls are
/// present — both indicate the rendered form may not match the underlying
/// identity. Full skeleton-based homoglyph detection is Phase 3.
pub fn has_homoglyph_risk(text: &str) -> bool {
    let normalized = normalize(text);
    normalized != text
        || contains_zero_width(text)
        || contains_bidi_controls(text)
        || looks_mixed_script(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfkc_folds_fullwidth_and_compat() {
        // Fullwidth 'a' (U+FF41) folds to 'a' under NFKC.
        assert_eq!(normalize("\u{FF41}bc"), "abc");
        // Ligature fi folds to "fi".
        assert_eq!(normalize("\u{FB01}n"), "fin");
    }

    #[test]
    fn zero_width_detection() {
        assert!(contains_zero_width("pay\u{200B}pal"));
        assert!(contains_zero_width("\u{FEFF}x"));
        assert!(!contains_zero_width("paypal"));
    }

    #[test]
    fn bidi_control_detection() {
        assert!(contains_bidi_controls("ab\u{202E}cd"));
        assert!(contains_bidi_controls("\u{2066}x"));
        assert!(!contains_bidi_controls("abcd"));
    }

    #[test]
    fn mixed_script_detected() {
        // Latin + Cyrillic: classic homoglyph attack surface.
        assert!(looks_mixed_script("p\u{0430}ypal")); // 'a' is Cyrillic а
        assert!(!looks_mixed_script("paypal"));
        assert!(!looks_mixed_script("пример")); // single script Cyrillic
    }

    #[test]
    fn common_characters_are_script_neutral() {
        // Spaces, digits, and punctuation must not count as a script.
        assert!(!looks_mixed_script("Download malware"));
        assert!(!looks_mixed_script("Item 42 (x2)"));
        assert!(!looks_mixed_script("总 42 件")); // Han + digits/space only
                                                  // Han + Latin letters is a genuine mix and must be flagged.
        assert!(looks_mixed_script("总 42 items"));
    }

    #[test]
    fn unclassifiable_is_not_silent_latin() {
        // Contains a character outside our classification table with no
        // other script present: must not be reported as pure Latin.
        // (Detector returns true = "cannot exclude mixing".)
        assert!(looks_mixed_script("ab\u{2200}")); // ∀ unclassifiable
    }

    #[test]
    fn homoglyph_risk_flags_confusables() {
        assert!(has_homoglyph_risk("p\u{0430}ypal")); // Cyrillic а
        assert!(has_homoglyph_risk("pay\u{200B}pal")); // zero-width
        assert!(!has_homoglyph_risk("paypal"));
    }
}
