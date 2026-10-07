// Skills subsystem.
//
// C-100: skill records and skill versions are immutable; macro execution is
//        represented through canonical semantic SkillCommitments.
// C-101: skill commitments are origin-bound by default and require unique,
//        safe resolution of required roles/relationships; fuzzy DOM fingerprint
//        similarity is prohibited.
// C-102: material skill drift automatically suspends a skill; critical security
//        bugs trigger immediate revocation; in-flight tasks abort safely; no
//        automatic re-promotion from resemblance.
// C-103: skill memory is allowlist-defined structural/semantic metadata only;
//        must not retain field values, text excerpts, token-bearing URLs, or
//        page-derived content beyond structure.
// C-104: skill shadow evaluation is replay-based; live shadow inference not used.
// C-105: side-effectful skill candidates evaluated only in mock/replay/dry-run/
//        sandbox.
// C-106: generalization must measure >= 20 variations; if > 80% are site-specific,
//        characterize honestly as procedural-cache/acceleration with measured
//        break-even.
// C-107: role-to-field residual risk measured by form-field-swap and invariant-
//        failure rates and feeds root-cause taxonomy.

pub mod store;
pub mod versions;
pub mod commitment;
pub mod pipeline;
pub mod sanitize_allowlist;
pub mod shadow;
pub mod promotion;
pub mod revocation;
pub mod drift;
