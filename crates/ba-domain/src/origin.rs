//! Exact-origin parsing and comparison (Sections 2.5, 13.3, 13.5).
//!
//! Credential and egress rules require *exact* origin identity: scheme,
//! hostname and port. Subdomains, sibling domains and alternate ports are
//! different origins and never inherit an authorization.

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Maximum accepted host length in bytes.
pub const MAX_HOST_BYTES: usize = 253;

/// Scheme of a parsed origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OriginScheme {
    /// Plain HTTP. Never eligible for credential fill.
    Http,
    /// HTTPS.
    Https,
    /// The first-party local application scheme serving the app UI.
    BrowserAgent,
}

impl OriginScheme {
    /// Returns the canonical scheme text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::BrowserAgent => "browser-agent",
        }
    }

    /// Returns the default port for this scheme, when one exists.
    #[must_use]
    pub const fn default_port(self) -> Option<u16> {
        match self {
            Self::Http => Some(80),
            Self::Https => Some(443),
            Self::BrowserAgent => None,
        }
    }
}

/// Errors produced while parsing an origin.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OriginError {
    /// The input was empty.
    #[error("origin is empty")]
    Empty,
    /// The scheme was missing or not `://`-separated.
    #[error("origin is missing a scheme")]
    MissingScheme,
    /// The scheme is not one of the supported schemes.
    #[error("unsupported scheme `{0}`")]
    UnsupportedScheme(String),
    /// The authority contained userinfo, which is a credential leak.
    #[error("URL contains embedded credentials")]
    EmbeddedCredentials,
    /// The host was absent.
    #[error("origin is missing a host")]
    MissingHost,
    /// The host contained a character outside the accepted set.
    #[error("host contains a character outside the accepted set")]
    InvalidHost,
    /// The host exceeded [`MAX_HOST_BYTES`].
    #[error("host is longer than {MAX_HOST_BYTES} bytes")]
    HostTooLong,
    /// The port was not a valid `u16` or was zero.
    #[error("invalid port `{0}`")]
    InvalidPort(String),
    /// The host used non-ASCII characters. Internationalized hostnames must be
    /// supplied in their punycode (ACE) form.
    #[error("host must be ASCII punycode")]
    NonAsciiHost,
}

/// A parsed, exact origin.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Origin {
    scheme: OriginScheme,
    host: String,
    port: u16,
}

impl Origin {
    /// Parses the origin of an absolute URL, ignoring path, query and fragment.
    ///
    /// # Errors
    ///
    /// Returns [`OriginError`] for a malformed or unsupported URL.
    pub fn parse(url: &str) -> Result<Self, OriginError> {
        let url = url.trim();
        if url.is_empty() {
            return Err(OriginError::Empty);
        }
        let (scheme_text, rest) = url.split_once("://").ok_or(OriginError::MissingScheme)?;
        let scheme = match scheme_text.to_ascii_lowercase().as_str() {
            "http" => OriginScheme::Http,
            "https" => OriginScheme::Https,
            "browser-agent" => OriginScheme::BrowserAgent,
            other => return Err(OriginError::UnsupportedScheme(other.to_owned())),
        };
        let authority = rest
            .split(['/', '?', '#'])
            .next()
            .ok_or(OriginError::MissingHost)?;
        if authority.is_empty() {
            return Err(OriginError::MissingHost);
        }
        if authority.contains('@') {
            return Err(OriginError::EmbeddedCredentials);
        }

        let (host_raw, port_raw) = split_authority(authority)?;
        let host = host_raw.to_ascii_lowercase();
        if host.is_empty() {
            return Err(OriginError::MissingHost);
        }
        if host.len() > MAX_HOST_BYTES {
            return Err(OriginError::HostTooLong);
        }
        if !host.is_ascii() {
            return Err(OriginError::NonAsciiHost);
        }
        if !is_valid_host(&host) {
            return Err(OriginError::InvalidHost);
        }

        let port = match port_raw {
            Some(text) => text
                .parse::<u16>()
                .ok()
                .filter(|value| *value != 0)
                .ok_or_else(|| OriginError::InvalidPort(text.to_owned()))?,
            None => scheme.default_port().unwrap_or(0),
        };

        Ok(Self { scheme, host, port })
    }

    /// Returns the scheme.
    #[must_use]
    pub const fn scheme(&self) -> OriginScheme {
        self.scheme
    }

    /// Returns the lowercase hostname.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the effective port. `0` means the scheme has no port concept.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Returns whether this is the first-party local application scheme.
    #[must_use]
    pub const fn is_local_app_scheme(&self) -> bool {
        matches!(self.scheme, OriginScheme::BrowserAgent)
    }

    /// Returns whether the transport is HTTPS.
    #[must_use]
    pub const fn is_secure(&self) -> bool {
        matches!(self.scheme, OriginScheme::Https)
    }

    /// Returns the exact-origin comparison. Equivalent to `==`, stated
    /// explicitly so that call sites read as an authorization decision.
    #[must_use]
    pub fn same_origin_exact(&self, other: &Self) -> bool {
        self == other
    }

    /// Returns the canonical origin text: explicit non-default port only.
    #[must_use]
    pub fn canonical(&self) -> String {
        let default = self.scheme.default_port();
        if default == Some(self.port) || self.port == 0 {
            format!("{}://{}", self.scheme.as_str(), self.host)
        } else {
            format!("{}://{}:{}", self.scheme.as_str(), self.host, self.port)
        }
    }

    /// Returns the plain-language transport label used in app-owned chrome.
    #[must_use]
    pub const fn security_label(&self) -> &'static str {
        match self.scheme {
            OriginScheme::Https => "Secure",
            OriginScheme::Http => "Not secure \u{2014} HTTP",
            OriginScheme::BrowserAgent => "Local application page",
        }
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical())
    }
}

impl FromStr for Origin {
    type Err = OriginError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for Origin {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.canonical())
    }
}

impl<'de> Deserialize<'de> for Origin {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

fn split_authority(authority: &str) -> Result<(&str, Option<&str>), OriginError> {
    if let Some(rest) = authority.strip_prefix('[') {
        let (host, tail) = rest.split_once(']').ok_or(OriginError::InvalidHost)?;
        let port = match tail.strip_prefix(':') {
            Some(text) => Some(text),
            None if tail.is_empty() => None,
            None => return Err(OriginError::InvalidHost),
        };
        return Ok((host, port));
    }
    match authority.split_once(':') {
        Some((host, port)) => Ok((host, Some(port))),
        None => Ok((authority, None)),
    }
}

fn is_valid_host(host: &str) -> bool {
    if host.starts_with('[') || host.ends_with(']') {
        return false;
    }
    if host.contains("::") || host.chars().all(|c| c.is_ascii_hexdigit() || c == ':') {
        // IPv6 literal form is accepted as a bare host without brackets.
        return !host.is_empty();
    }
    host.split('.').all(|label| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    })
}

/// Returns whether the URL carries userinfo (`https://user:pw@host`) or a
/// credential-bearing query parameter.
#[must_use]
pub fn url_discloses_credentials(url: &str) -> bool {
    let authority = url.split_once("://").map_or(url, |(_, rest)| rest);
    let authority = authority.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return true;
    }
    let query = url.split_once('?').map_or("", |(_, rest)| rest);
    let query = query.split('#').next().unwrap_or_default();
    query.split('&').any(|pair| {
        let key = pair
            .split('=')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        matches!(
            key.as_str(),
            "token"
                | "access_token"
                | "api_key"
                | "apikey"
                | "password"
                | "passwd"
                | "otp"
                | "session"
                | "sessionid"
                | "secret"
                | "code"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalises_default_ports_and_case() {
        let origin = Origin::parse("HTTPS://Example.COM/a/b?x=1").unwrap();
        assert_eq!(origin.canonical(), "https://example.com");
        assert_eq!(origin.port(), 443);
        assert!(origin.is_secure());
    }

    #[test]
    fn keeps_explicit_non_default_ports() {
        let origin = Origin::parse("https://example.com:8443/x").unwrap();
        assert_eq!(origin.canonical(), "https://example.com:8443");
    }

    #[test]
    fn exact_origin_comparison_is_strict() {
        let base = Origin::parse("https://example.com").unwrap();
        assert!(base.same_origin_exact(&Origin::parse("https://example.com/login").unwrap()));
        assert!(!base.same_origin_exact(&Origin::parse("https://www.example.com").unwrap()));
        assert!(!base.same_origin_exact(&Origin::parse("https://example.com:8443").unwrap()));
        assert!(!base.same_origin_exact(&Origin::parse("http://example.com").unwrap()));
        assert!(!base.same_origin_exact(&Origin::parse("https://evil.test").unwrap()));
    }

    #[test]
    fn http_is_labelled_insecure() {
        let origin = Origin::parse("http://example.com").unwrap();
        assert_eq!(origin.security_label(), "Not secure \u{2014} HTTP");
    }

    #[test]
    fn rejects_hostile_input() {
        assert_eq!(Origin::parse(""), Err(OriginError::Empty));
        assert_eq!(
            Origin::parse("example.com"),
            Err(OriginError::MissingScheme)
        );
        assert_eq!(
            Origin::parse("ftp://example.com"),
            Err(OriginError::UnsupportedScheme("ftp".to_owned()))
        );
        assert_eq!(
            Origin::parse("https://user:pw@example.com"),
            Err(OriginError::EmbeddedCredentials)
        );
        assert_eq!(
            Origin::parse("https://example.com:0"),
            Err(OriginError::InvalidPort("0".to_owned()))
        );
        assert_eq!(
            Origin::parse("https://example.com:70000"),
            Err(OriginError::InvalidPort("70000".to_owned()))
        );
        assert_eq!(
            Origin::parse("https://ex\u{e4}mple.com"),
            Err(OriginError::NonAsciiHost)
        );
        assert_eq!(
            Origin::parse("https://exa mple.com"),
            Err(OriginError::InvalidHost)
        );
    }

    #[test]
    fn accepts_local_app_scheme_and_ipv6() {
        let app = Origin::parse("browser-agent://ui/").unwrap();
        assert!(app.is_local_app_scheme());
        assert_eq!(app.canonical(), "browser-agent://ui");
        let v6 = Origin::parse("http://[::1]:8080/x").unwrap();
        assert_eq!(v6.canonical(), "http://::1:8080");
    }

    #[test]
    fn detects_credential_disclosure_in_urls() {
        assert!(url_discloses_credentials("https://a:b@example.com/"));
        assert!(url_discloses_credentials("https://example.com/?token=abc"));
        assert!(url_discloses_credentials(
            "https://example.com/search?q=x&api_key=1"
        ));
        assert!(!url_discloses_credentials("https://example.com/?q=shoes"));
    }
}
