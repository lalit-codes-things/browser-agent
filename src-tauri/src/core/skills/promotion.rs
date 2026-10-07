// Promotion.
//
// C-102: no automatic re-promotion from resemblance; promotion creates a new
//        immutable version.

pub struct Promotion;

impl Promotion {
    pub fn request(_skill_id: &str, _approval_id: &str) -> Result<crate::core::skills::versions::SkillVersion, crate::Error> {
        Err(crate::Error::NotImplemented("Promotion::request is scheduled".into()))
    }
}
