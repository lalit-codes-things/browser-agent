// Shadow evaluation.
//
// C-104: skill shadow evaluation is replay-based; live shadow inference is
//        not used because it would double inference on the 8 GB envelope.

pub struct ShadowEvaluator;

impl ShadowEvaluator {
    pub fn evaluate_replay(_commitment: &crate::core::skills::commitment::SkillCommitment, _trace: &[String]) -> Result<bool, crate::Error> {
        Err(crate::Error::NotImplemented("ShadowEvaluator::evaluate_replay is scheduled".into()))
    }
}
