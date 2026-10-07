// Policy subsystem.
//
// C-06: action classes exactly READ / REVERSIBLE_WRITE / IRREVERSIBLE /
//        UNKNOWN -> IRREVERSIBLE.
// C-07: authorization tier is a separate dimension: NONE / POLICY_ONLY /
//        NATIVE_CONFIRM / BIOMETRIC_CONFIRM.
// C-08: tier derived by Policy, never by model, using side-effect class +
//        taint, destination, data flow, amount thresholds, task authority.
// C-09: UNKNOWN consequential actions treated as IRREVERSIBLE and tiered
//        conservatively/upward on uncertainty.
// C-14: confirmation is tiered; fresh Touch ID not required for every
//        irreversible action.
// C-15: fresh Touch ID only for HIGH_STAKES payments/financial actions and
//        important sensitive decisions.
// C-16: ordinary login/browsing/typing/credential fill/reversible/routine
//        execution do not use biometrics.
// C-17: other consequential actions use native confirmation and policy
//        controls without biometrics.
// C-18: high-stakes biometric prompts rate-limited to 3 rejected attempts
//        per 60 seconds; native-confirmation + master-password fallback
//        symmetrically rate-limited and audited.
// C-19: screen lock/sleep invalidates pending biometric approvals.
// C-20: no silent written-policy override that disables HIGH_STAKES auth.
// C-21: CI/headless/replay cannot perform real HIGH_STAKES actions outside
//        isolated synthetic policy.
// C-22: model knows only REQUEST_CONFIRMATION.
// C-24: approval commitments include authorization_tier; execution-time
//        tier mismatch is a commitment mismatch.
// C-25: on mismatch invalidate -> re-stabilize -> re-perceive -> at most
//        one fresh native re-approval -> recover.
// C-26: no page may relabel an approved ordinary submission into/out of a
//        biometric-gated action without re-approval.
// C-27: exact-origin credentials default; multi-subdomain requires explicit
//        scope object; wildcards restrictive.
// C-28: credentials modeled as (origin, account); disambiguation driven by
//        task authority, not page-visible username field.
// C-29: same-origin account conflicts fail to clarification.
// C-30: session-surviving taint preserved until explicit declassification.
// C-31: sensitive-to-public data flow maps to HIGH_STAKES; credential-to-public
//        is blocked.
// C-32: navigation URLs treated as potential exfiltration channels.
// C-33: task-authority parameters must reconcile with action parameters;
//        divergence is STOP.
// C-34: every skill/macro step is independently policy checked.
// C-35: recovery mode uses tighter budgets and is protected against
//        attacker-triggered uncertainty with rate limiting.
// C-36: LLM/evidence logic is never the authority for HIGH_STAKES auth.
// C-139: payments/transfers/sensitive publication/transmission/account-security
//         changes remain behind fresh biometrics; ordinary irreversible actions
//         use native confirmation only.
// C-140: tier under-classification is the measured, gated, adversarially
//         tested safety property.
// C-144: page-derived content is untrusted; no page-controlled data can
//         redefine task authority, policy, or authorization tier.
// C-148: vault key hierarchy: master password -> Argon2id KEK -> Keychain
//         master key wrapping distinct vault-item and audit keys; password
//         change re-wraps keys without breaking HMAC chain verification.

pub mod classes;
pub mod tiers;
pub mod scopes;
pub mod taint;
pub mod provenance;
pub mod declassify;
pub mod reconcile;
pub mod locale_amount;
pub mod recovery_mode;
pub mod rate_limit;
pub mod confirmation;
pub mod engine;
