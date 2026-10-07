// Verification subsystem.
//
// C-91: exactly five outcome categories: VERIFIED_SUCCESS, LIKELY_SUCCESS,
//        UNKNOWN, LIKELY_FAILURE, VERIFIED_FAILURE.
// C-92: irreversible never auto-retried after LIKELY_SUCCESS or UNKNOWN;
//        VERIFIED_FAILURE retries only when Policy proves repeatability.
// C-93: VERIFIED_SUCCESS requires sufficiently independent evidence; page-
//        controlled banners are not independent truth.
// C-94: secure-fill verification relies on input-state mutation plus form
//        transition and never reads the secret value back.
// C-95: download verification checks size and type bounds before VERIFIED_SUCCESS.
// C-158: verification is independent of the model; model self-report of
//         success is never verification evidence.

pub mod outcomes;
pub mod evidence;
pub mod tiers;
pub mod postconditions;
pub mod network_evidence;
pub mod download_check;
pub mod redaction;
