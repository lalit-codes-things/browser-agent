//! Redaction primitives (Sections 12.5, 13.5, 14.3).
//!
//! Redaction happens before data reaches presentation code, model prompts,
//! logs, task history or diagnostics. A CSS mask is not a security boundary,
//! so masking is done in trusted code at the point of construction.

use std::borrow::Cow;

/// Replacement text for a masked value.
pub const REDACTED: &str = "[redacted]";

/// Query-parameter names whose values are treated as secret material.
const SENSITIVE_QUERY_KEYS: [&str; 13] = [
    "token",
    "access_token",
    "refresh_token",
    "id_token",
    "api_key",
    "apikey",
    "password",
    "passwd",
    "pwd",
    "otp",
    "session",
    "sessionid",
    "secret",
];

/// Bare words that name secret material when used as an assignment key.
const SENSITIVE_KEYS: [&str; 12] = [
    "password",
    "passwd",
    "pwd",
    "passcode",
    "pin",
    "api_key",
    "apikey",
    "secret",
    "token",
    "otp",
    "authorization",
    "session",
];

/// One whitespace-delimited word, preserving whether it is whitespace.
struct Segment<'a> {
    is_whitespace: bool,
    text: &'a str,
}

fn segments(value: &str) -> Vec<Segment<'_>> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut current: Option<bool> = None;
    for (index, character) in value.char_indices() {
        let whitespace = character.is_whitespace();
        match current {
            Some(state) if state == whitespace => {}
            Some(state) => {
                out.push(Segment {
                    is_whitespace: state,
                    text: &value[start..index],
                });
                start = index;
                current = Some(whitespace);
            }
            None => current = Some(whitespace),
        }
    }
    if start < value.len() {
        out.push(Segment {
            is_whitespace: current.unwrap_or(false),
            text: &value[start..],
        });
    }
    out
}

/// Returns whether `word` is an assignment whose key names secret material.
fn is_sensitive_assignment(word: &str) -> bool {
    match word.find(['=', ':']) {
        Some(index) => sensitive_key(&word[..index]),
        None => false,
    }
}

/// Returns whether `text` names secret material.
fn sensitive_key(text: &str) -> bool {
    let lower = text.trim().to_ascii_lowercase();
    if lower.is_empty() || lower.len() > 24 {
        return false;
    }
    SENSITIVE_KEYS.iter().any(|key| {
        lower == *key
            || lower == format!("my {key}")
            || lower.ends_with(&format!("_{key}"))
            || lower.ends_with(&format!("-{key}"))
    })
}

/// Returns whether `value` looks like secret material that must not enter a
/// model prompt or task history (BA-TRANSPARENCY-001).
///
/// This is a conservative heuristic used to *ask the user* to use the vault
/// instead of sending a literal secret to the model. It is deliberately not
/// used as an access-control decision.
#[must_use]
pub fn is_secret_like(value: &str) -> bool {
    let trimmed = value.trim();
    // `password: hunter2`, `my token = abc` and similar assignments.
    if segments(value)
        .iter()
        .any(|segment| !segment.is_whitespace && is_sensitive_assignment(segment.text))
    {
        return true;
    }
    if trimmed.len() < 6 {
        return false;
    }
    let digits = trimmed.chars().filter(char::is_ascii_digit).count();
    // One-time codes: 6-10 digits, optionally spaced or hyphenated.
    let code_shaped = trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == ' ' || c == '-')
        && (6..=10).contains(&digits);
    if code_shaped {
        return true;
    }
    // Bearer tokens and JWT-shaped values.
    if trimmed.to_ascii_lowercase().starts_with("bearer ") {
        return true;
    }
    if trimmed.split('.').count() == 3
        && trimmed.starts_with("eyJ")
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return true;
    }
    // Long high-entropy blobs with no whitespace.
    if trimmed.len() >= 32
        && !trimmed.contains(char::is_whitespace)
        && trimmed.chars().filter(char::is_ascii_digit).count() >= 4
        && trimmed.chars().any(|c| c.is_ascii_lowercase())
        && trimmed
            .chars()
            .any(|c| c.is_ascii_uppercase() || !c.is_ascii_alphanumeric())
    {
        return true;
    }
    false
}

/// Masks secret-shaped runs in `value`.
///
/// Returns a borrowed value when nothing matched, so the common path allocates
/// nothing.
#[must_use]
pub fn mask_secret_like(value: &str) -> Cow<'_, str> {
    let mut output = String::with_capacity(value.len());
    let mut changed = false;
    let mut mask_next_word = false;

    for segment in segments(value) {
        if segment.is_whitespace {
            output.push_str(segment.text);
            continue;
        }
        if mask_next_word {
            output.push_str(REDACTED);
            changed = true;
            mask_next_word = false;
            continue;
        }
        let word = segment.text;
        if is_sensitive_assignment(word)
            && let Some(index) = word.find(['=', ':'])
        {
            let (key, value_part) = word.split_at(index + 1);
            output.push_str(key);
            if value_part.is_empty() {
                // `password:` — the value is the next word.
                mask_next_word = true;
            } else {
                output.push_str(REDACTED);
                changed = true;
            }
            continue;
        }
        if is_secret_like(word) {
            output.push_str(REDACTED);
            changed = true;
        } else {
            output.push_str(word);
        }
    }

    if changed {
        Cow::Owned(output)
    } else {
        Cow::Borrowed(value)
    }
}

/// Redacts a URL for storage or display: userinfo is dropped and
/// credential-bearing query parameters are replaced.
///
/// The scheme, host, port and path are preserved because origin identity is
/// security-relevant and must not be hidden.
#[must_use]
pub fn redact_url(url: &str) -> String {
    let (scheme, rest) = match url.split_once("://") {
        Some(parts) => parts,
        None => ("", url),
    };
    let (authority_raw, tail) = match rest.find(['/', '?', '#']) {
        Some(index) => rest.split_at(index),
        None => (rest, ""),
    };
    let authority = match authority_raw.split_once('@') {
        Some((_, host)) => host,
        None => authority_raw,
    };

    let mut tail_out = String::with_capacity(tail.len());
    if let Some((path, query)) = tail.split_once('?') {
        tail_out.push_str(path);
        tail_out.push('?');
        let (query, fragment) = match query.split_once('#') {
            Some((q, f)) => (q, Some(f)),
            None => (query, None),
        };
        let mut first = true;
        for pair in query.split('&') {
            if !first {
                tail_out.push('&');
            }
            first = false;
            let key = pair.split('=').next().unwrap_or_default();
            if is_sensitive_query_key(key) {
                tail_out.push_str(key);
                tail_out.push('=');
                tail_out.push_str(REDACTED);
            } else {
                tail_out.push_str(pair);
            }
        }
        if let Some(fragment) = fragment {
            tail_out.push('#');
            tail_out.push_str(fragment);
        }
    } else {
        tail_out.push_str(tail);
    }

    if scheme.is_empty() {
        format!("{authority}{tail_out}")
    } else {
        format!("{scheme}://{authority}{tail_out}")
    }
}

fn is_sensitive_query_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    SENSITIVE_QUERY_KEYS.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_otp_and_token_shapes() {
        assert!(is_secret_like("123456"));
        assert!(is_secret_like("123 456"));
        assert!(is_secret_like("Bearer abc123def456"));
        assert!(is_secret_like("password=hunter2"));
        assert!(is_secret_like("api_key: abcd1234"));
        assert!(is_secret_like("my password: hunter2"));
        assert!(is_secret_like(
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc-_123"
        ));
        assert!(!is_secret_like("buy a pair of running shoes"));
        assert!(!is_secret_like("2026"));
    }

    #[test]
    fn masks_values_after_a_sensitive_assignment_key() {
        assert_eq!(
            mask_secret_like("my password: hunter2"),
            format!("my password: {REDACTED}")
        );
        assert_eq!(
            mask_secret_like("token=abc123"),
            format!("token={REDACTED}")
        );
        assert_eq!(
            mask_secret_like("password: hunter2 extra words"),
            format!("password: {REDACTED} extra words")
        );
    }

    #[test]
    fn masks_only_the_secret_token() {
        let masked = mask_secret_like("my OTP is 123456 please");
        assert_eq!(masked, format!("my OTP is {REDACTED} please"));
        assert_eq!(
            mask_secret_like("nothing sensitive here"),
            "nothing sensitive here"
        );
    }

    #[test]
    fn redacts_userinfo_and_sensitive_query_values() {
        assert_eq!(
            redact_url("https://user:pw@example.com/login?token=abc&q=shoes#frag"),
            format!("https://example.com/login?token={REDACTED}&q=shoes#frag")
        );
        assert_eq!(
            redact_url("https://example.com/path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn redaction_keeps_origin_visible() {
        let redacted = redact_url("http://user:pw@insecure.test/checkout?otp=1");
        assert!(redacted.starts_with("http://insecure.test/checkout"));
        assert!(!redacted.contains("user"));
        assert!(!redacted.contains("otp=1"));
    }
}
