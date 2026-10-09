// Drift detection.
//
// C-102: material skill drift automatically suspends a skill.

#[derive(Debug, Clone)]
pub enum DriftSeverity {
    Material,
    Minor,
}

pub struct DriftDetector;

impl DriftDetector {
    pub fn evaluate(
        _commitment: &crate::core::skills::commitment::SkillCommitment,
        _observed: &[String],
    ) -> Option<DriftSeverity> {
        None
    }
}
