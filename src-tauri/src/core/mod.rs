// Core engine modules.
//
// Five engines with orchestrator/task-normalizer/permission-check
// performing orchestration and the Skill Store as the data layer
// (C-01).
//
// Module boundaries are intentional. Each subsystem keeps its own
// types, tests, and C-ID references.

pub mod orchestrator;
pub mod perception;
pub mod reasoning;
pub mod policy;
pub mod execution;
pub mod verification;
pub mod skills;
pub mod vault;
