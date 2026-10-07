// Audit subsystem.
//
// C-62: audit is HMAC-chained from the first commit.
// C-96: audit is encrypted, append-only, HMAC-chained with rotated-segment
//        linkage, redacted, retained under policy, user-exportable, locally
//        verifiable; signed export provided.
// C-148: password change re-wraps keys without breaking HMAC chain verification
//        across rotated segments.

pub mod chain;
pub mod encrypt;
pub mod access;
pub mod redact;
