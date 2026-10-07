// Orchestrator subsystem.
//
// C-01: Task Normalizer and Permission Check act as orchestration.
// C-154: orchestrator-owned typed task-progress object updated deterministically
//        after each verified action and supplied to the model at every step.
// C-63: hard stop = preserve safe state -> audit -> notify -> require intervention;
//        resumption never reuses stale state.
// C-149: budgets; exceeding any triggers C-63 stop semantics.
// C-60: no unwrap in policy-critical paths.

pub mod task_normalizer;
pub mod permission_check;
pub mod task_authority;
pub mod progress;
pub mod state_machine;
pub mod resume;
pub mod human_checkpoint;
pub mod budgets;
pub mod intervention_epoch;

pub use task_normalizer::TaskNormalizer;
pub use permission_check::{PermissionCheck, PermissionOutcome};
pub use progress::{TaskProgress, TaskState, ProgressMode};
pub use state_machine::{StateMachine, TransitionError};
