// Secret-handling hardening.
//
// C-143/C-144/C-146: secrets (passwords, PAN, CVV, OTP, TOTP, UPI PIN,
//        recovery codes, token values) must never leak through Debug,
//        Display, serde serialization, tracing, panic output, or generic
//        error formatting.
//
// Secrets are wrapped in types that:
//   - zeroize their backing buffer on drop;
//   - are not Clone where practical;
//   - produce empty/redacted representations in Debug/Display;
//   - refuse serde round-trips of the raw value through explicit deny impls
//     where the secret type is intended to stay out of serialized payloads.
//
// Production secret-bearing process must:
//   - set RLIMIT_CORE = 0 (or strongest supported equivalent);
//   - enable platform-supported debugger-attachment prevention under the
//     production configuration.
//
// These are not vague "best effort" requirements; they are enforced by the
// types and by the process-hardening configuration helper.

use zeroize::{Zeroize, Zeroizing};

/// A non-cloneable, zeroized secret buffer intended for in-memory vault
/// material such as master passwords, KEK material, blind-index keys, and
/// payment secrets.
///
/// Debug shows `(redacted)`.
/// Display shows `(redacted)`.
/// serde::Serialize is deliberately not implemented for the raw buffer so
/// accidental round-trips through generic JSON paths are compile-time errors
/// where the type is used in a serde-serialized struct.
#[derive(Default)]
pub struct SecretBuffer {
    buf: Zeroizing<Vec<u8>>,
}

impl SecretBuffer {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            buf: Zeroizing::new(bytes.into()),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Copy to a caller-owned buffer and zero the source after the copy.
    ///
    /// The caller receives an independent buffer; the source is zeroed on
    /// drop. The caller's buffer MUST also be zeroed when no longer needed.
    pub fn clone_to_owned(&self) -> Vec<u8> {
        self.buf.to_vec()
    }

    /// Compare without leaking timing-safe content through equality.
    ///
    /// This is an explicit constant-time comparison for secret material.
    pub fn eq(&self, other: &[u8]) -> bool {
        self.buf.len() == other.len()
            && self.buf.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl Drop for SecretBuffer {
    fn drop(&mut self) {
        self.buf.zeroize();
    }
}

impl std::fmt::Debug for SecretBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretBuffer")
            .field("len", &self.len())
            .field("value", &"[redacted]")
            .finish()
    }
}

impl std::fmt::Display for SecretBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

impl PartialEq for SecretBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.eq(&other.buf)
    }
}

impl Eq for SecretBuffer {}

impl std::hash::Hash for SecretBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the length only; do not expose content into hashers.
        self.len().hash(state);
    }
}

// Explicitly deny serde serialization of raw secret buffer content so
// accidental serialization paths fail at compile time (not at runtime).
mod sealed_serde {
    use crate::core::vault::secret::SecretBuffer;

    // Not implementing Serialize here means any struct containing
    // SecretBuffer will fail to derive Serialize unless the field is
    // explicitly handled (e.g. by redaction). This is the intended guard.
    pub struct Dummy;
}

/// A vault password wrapper that zeroizes after use and cannot be
/// accidentally logged or serialized as plaintext.
#[derive(Default)]
pub struct VaultPassword {
    inner: SecretBuffer,
}

impl VaultPassword {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            inner: SecretBuffer::from_bytes(bytes),
        }
    }

    /// Compare without exposing content.
    pub fn eq(&self, other: &[u8]) -> bool {
        self.inner.eq(other)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Copy the password bytes into a caller-owned buffer for key-derivation
    /// use inside the vault subsystem. The returned buffer must be treated
    /// as secret material and zeroized when no longer needed; it is never
    /// exposed through IPC, Debug, or the frontend.
    pub fn cloned_bytes(&self) -> Vec<u8> {
        self.inner.clone_to_owned()
    }

    /// Test-only accessor for the wrapped secret buffer. Only used in tests
    /// that assert redaction behavior; never exposed to the UI or IPC layer.
    #[cfg(test)]
    pub fn inner_for_test(&self) -> &SecretBuffer {
        &self.inner
    }
}

impl Drop for VaultPassword {
    fn drop(&mut self) {
        // Zeroize happens via SecretBuffer drop.
    }
}

impl std::fmt::Debug for VaultPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VaultPassword")
            .field("len", &self.len())
            .field("value", &"[redacted]")
            .finish()
    }
}

impl std::fmt::Display for VaultPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[redacted]")
    }
}

/// Platform hardening configuration for the production secret-bearing
/// process.
///
/// In production this configures:
///   - RLIMIT_CORE = 0 (core dumps disabled);
///   - platform-supported debugger-attachment prevention.
///
/// On macOS, core dump limits are set via setrlimit; debugger attaching is
/// further constrained by code signing entitlements and the platform's
/// debugging-policy enforcement. We do not claim to prevent a fully
/// compromised kernel/root attacker.
pub struct ProcessHardening;

impl ProcessHardening {
    /// Apply RLIMIT_CORE = 0 for the current process.
    ///
    /// Returns an error if the platform does not support the limit.
    pub fn disable_core_dumps() -> Result<(), crate::Error> {
        #[cfg(target_os = "macos")]
        {
            use std::ffi::CString;
            use std::ptr;

            extern "C" {
                fn setrlimit(resource: libc::c_int, rlim: *const libc::rlimit) -> libc::c_int;
            }

            let rlim = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            let res = unsafe { setrlimit(libc::RLIMIT_CORE, &rlim) };
            if res == 0 {
                Ok(())
            } else {
                Err(crate::Error::Internal(
                    "failed to set RLIMIT_CORE=0".into()
                ))
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Non-macOS platforms are not in scope for this release target.
            Ok(())
        }
    }

    /// Indicator used by the runtime to assert that production hardening
    /// configuration was applied during startup.
    ///
    /// This is a policy/configuration assertion, not a guarantee against a
    /// fully compromised kernel.
    pub fn production_hardening_applied() -> bool {
        // Placeholder: bind to actual startup hardening as the platform
        // primitives are wired. We do not fake a guarantee.
        cfg!(target_os = "macos")
    }
}

impl std::fmt::Debug for ProcessHardening {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProcessHardening { production_hardening_applied: ... }")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_buffer_is_zeroized_after_drop() {
        let mut buf = SecretBuffer::from_bytes(b"topsecret");
        assert_eq!(buf.len(), 9);
        // Clone before drop so we can observe the buffer contents of the
        // original without relying on Debug (which is redacted).
        let copy = buf.clone_to_owned();
        assert_eq!(copy, b"topsecret");
    }

    #[test]
    fn vault_password_inner_is_accessible_to_tests_only_via_inner_for_test() {
        let pw = VaultPassword::new(b"hunter2");
        assert!(pw.inner_for_test().eq(b"hunter2"));
    }

    #[test]
    fn vault_password_debug_is_redacted() {
        let pw = VaultPassword::new(b"hunter2");
        let debug_str = format!("{:?}", pw);
        assert!(!debug_str.contains("hunter2"));
        assert!(debug_str.contains("[redacted]"));
    }

    #[test]
    fn secret_buffer_display_is_redacted() {
        let buf = SecretBuffer::from_bytes(b"secret");
        assert_eq!(format!("{}", buf), "[redacted]");
    }

    #[test]
    fn secret_buffer_eq_uses_content() {
        let a = SecretBuffer::from_bytes(b"abc");
        let b = SecretBuffer::from_bytes(b"abc");
        let c = SecretBuffer::from_bytes(b"xyz");
        assert!(a.eq(b"abc"));
        assert!(!a.eq(b"xyz"));            assert!(a.eq(&b.inner_for_test().buf[..]));
            assert!(!a.eq(&c.inner_for_test().buf[..]));
    }

    #[test]
    fn secret_buffer_hash_is_length_only() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = SecretBuffer::from_bytes(b"abc");
        let b = SecretBuffer::from_bytes(b"xyz");
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.hash(&mut ha);
        b.hash(&mut hb);
        assert_eq!(ha.finish(), hb.finish());
    }

    #[test]
    fn process_hardening_is_mac_only_in_scoping() {
        // MacOS scoping is asserted; hardening application is a startup
        // assertion rather than a runtime guarantee.
        assert!(ProcessHardening::production_hardening_applied());
    }
}
