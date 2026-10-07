// Reasoning subsystem.
//
// C-54: LLM-every-step replaced by uncertainty-triggered reasoning once
//        Phase 6 validates the shift.
// C-55: uncertainty triggers include target-resolution failure, ambiguity,
//        unexpected state, failed invariant, unresolved parameters,
//        contradictory state, policy clarification, recovery-mode entry.
// C-57: deterministic decoding is the baseline; sampling is only a
//        separately evaluated experiment.
// C-64: Phase-1 slice invokes model deterministically at every step using
//        fixed model, prompt/schema, quantization, runtime, decoding
//        configuration, and seed.
// C-146: per-call wall-clock timeout with abort callbacks; task abort
//         cancels in-flight generation.
// C-147: model-residency policy handles external memory pressure.
// C-151: metrics separate schema-valid rate from semantically-correct-action
//         rate; grammar-constrained decoding overhead measured separately.
// C-22: model knows only REQUEST_CONFIRMATION; does not know or control
//        the biometric mechanism.

pub mod session;
pub mod schema;
pub mod grammar;
pub mod triggers;
pub mod timeout;
pub mod abort;
pub mod canary;
pub mod uncertainty;
