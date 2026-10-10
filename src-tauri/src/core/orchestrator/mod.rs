// Orchestrator subsystem.
//
// C-01: Task Normalizer and Permission Check act as orchestration.
// C-154: orchestrator-owned typed task-progress object updated deterministically
//        after each verified action and supplied to the model at every step.
// C-63: hard stop = preserve safe state -> audit -> notify -> require intervention;
//        resumption never reuses stale state.
// C-149: budgets; exceeding any triggers C-63 stop semantics.
// C-60: no unwrap in policy-critical paths.

pub mod budgets;
pub mod human_checkpoint;
pub mod intervention_epoch;
pub mod permission_check;
pub mod progress;
pub mod resume;
pub mod runtime;
pub mod state_machine;
pub mod task_authority;
pub mod task_normalizer;

pub use permission_check::{PermissionCheck, PermissionOutcome};
pub use progress::{ProgressMode, TaskProgress, TaskState};
pub use runtime::{TaskRegistry, TaskRuntime, TaskSlot};
pub use state_machine::{StateMachine, TransitionError};
pub use task_normalizer::TaskNormalizer;
