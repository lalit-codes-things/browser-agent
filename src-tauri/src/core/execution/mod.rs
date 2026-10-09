// Execution engine.
//
// C-142: exposes only typed operations; arbitrary model-controlled
//        JavaScript prohibited in all phases.
// C-66: every action log records epoch, target/frame/loader identity,
//        semantic reference, policy result including tier, action type,
//        execution result, verification result.
// C-155: Phase-1 slice performs basic click hit-test validation before
//        execution; Phase 3 completes recursive OOPIF descent per C-87.
// C-141: actions bound to epoch from which they were derived.

pub mod authorization;
pub mod hittest;
pub mod idempotency;
pub mod primitives;


pub mod retry;
pub mod secure_fill;
pub mod serialization;
pub mod typing;
