// Navigation request validation.
//
// The omnibox submits a user-authored navigation target. The backend is
// the authority for whether a navigation is dispatchable: URLs are
// validated here before any CDP dispatch path consumes them. The frontend
// never treats a submitted string as a validated navigation.
//
// Rules:
//  - scheme must be http or https (arbitrary schemes are not dispatchable
//    navigation targets for the managed browser);
//  - the URL must be bounded in length and free of control characters and
//    whitespace;
//  - embedded credentials (user:pass@) are rejected — they are a phishing
//    vector and are never a legitimate agent navigation target.

use crate::error::Error;

pub const MAX_NAVIGATION_URL_BYTES: usize = 2_048;

/// Validate a navigation URL for dispatch against the managed browser.
///
/// Returns the trimmed URL on success. This is pure input validation; it
/// does not assert that any browser runtime is available to dispatch to.
pub fn validate_navigation_url(raw: &str) -> Result<String, Error> {
    let url = raw.trim();
    if url.is_empty() {
        return Err(Error::InvalidParameter("navigation url is required".into()));
    }
    if url.len() > MAX_NAVIGATION_URL_BYTES {
        return Err(Error::InvalidParameter(format!(
            "navigation url exceeds {MAX_NAVIGATION_URL_BYTES} bytes"
        )));
    }
    if url.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(Error::InvalidParameter(
            "navigation url contains control or whitespace characters".into(),
        ));
    }

    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| Error::InvalidParameter("navigation url requires a scheme".into()))?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(Error::InvalidParameter(
            "navigation scheme must be http or https".into(),
        ));
    }

    // Reject embedded credentials before the authority component.
    if rest.contains('@') {
        return Err(Error::InvalidParameter(
            "navigation url must not contain embedded credentials".into(),
        ));
    }

    let authority = rest.split('/').next().unwrap_or("");
    let host = authority.split(':').next().unwrap_or("");
    if host.is_empty() {
        return Err(Error::InvalidParameter(
            "navigation url requires a host".into(),
        ));
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_https_url() {
        assert_eq!(
            validate_navigation_url("https://example.test/path").unwrap(),
            "https://example.test/path"
        );
    }

    #[test]
    fn accepts_http_url_with_port() {
        assert!(validate_navigation_url("http://localhost:8080/").is_ok());
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(
            validate_navigation_url("  https://example.test  ").unwrap(),
            "https://example.test"
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert!(validate_navigation_url("   ").is_err());
    }

    #[test]
    fn rejects_missing_scheme() {
        assert!(validate_navigation_url("example.test/page").is_err());
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert!(validate_navigation_url("javascript:alert(1)").is_err());
        assert!(validate_navigation_url("file:///etc/passwd").is_err());
        assert!(validate_navigation_url("data:text/html,<script>").is_err());
    }

    #[test]
    fn rejects_embedded_credentials() {
        assert!(validate_navigation_url("https://user:pass@example.test/").is_err());
        assert!(validate_navigation_url("https://user@example.test/").is_err());
    }

    #[test]
    fn rejects_control_and_whitespace_characters() {
        assert!(validate_navigation_url("https://example.test/a b").is_err());
        assert!(validate_navigation_url("https://example.test/a\nb").is_err());
    }

    #[test]
    fn rejects_overlong_url() {
        let long = format!(
            "https://example.test/{}",
            "a".repeat(MAX_NAVIGATION_URL_BYTES)
        );
        assert!(validate_navigation_url(&long).is_err());
    }

    #[test]
    fn rejects_url_without_host() {
        assert!(validate_navigation_url("https://").is_err());
    }
}
