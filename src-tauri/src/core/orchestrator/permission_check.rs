// Permission Check.
//
// Orchestration-layer gate before execution. It validates task authority,
// required capabilities, and whether the task is within allowed scope.
//
// This is separate from Policy (which evaluates side-effect class and
// authorization tier). Permission check is about whether the task may
// proceed at all.

pub struct PermissionCheck;

impl PermissionCheck {
    pub fn check(_task: &crate::core::orchestrator::task_normalizer::NormalizedTask) -> Result<PermissionOutcome, crate::Error> {
        Err(crate::Error::NotImplemented(
            "PermissionCheck::check is scheduled".into(),
        ))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionOutcome {
    pub allowed: bool,
    pub reason: String,
}
