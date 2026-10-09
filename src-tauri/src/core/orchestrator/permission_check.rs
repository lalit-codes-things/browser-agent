pub struct PermissionCheck;

impl PermissionCheck {
    pub fn check(
        task: &crate::core::orchestrator::task_normalizer::NormalizedTask,
    ) -> Result<PermissionOutcome, crate::Error> {
        if task.original_text.trim().is_empty() {
            return Err(crate::Error::InvalidParameter(
                "normalized task is empty".into(),
            ));
        }
        if task.original_text.len() > 4_000 {
            return Err(crate::Error::PolicyBlocked(
                "task exceeds runtime limit".into(),
            ));
        }
        if task.projected_risk.level == crate::core::orchestrator::task_normalizer::RiskLevel::High
        {
            return Ok(PermissionOutcome {
                allowed: false,
                reason: "high-stakes task requires intervention".into(),
            });
        }
        Ok(PermissionOutcome {
            allowed: true,
            reason: "task is within the current shell scope".into(),
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PermissionOutcome {
    pub allowed: bool,
    pub reason: String,
}
