// Skill memory sanitization allowlist.
//
// C-103: skill memory is allowlist-defined structural/semantic metadata only;
//        must not retain field values, text excerpts, token-bearing URLs, or
//        page-derived content beyond structure.

pub struct SkillMemoryAllowlist;

impl SkillMemoryAllowlist {
    pub fn allowed_key(key: &str) -> bool {
        matches!(
            key,
            "role"
                | "relationship"
                | "origin"
                | "field_type"
                | "selection_criteria"
                | "precondition"
                | "postcondition"
                | "invariant"
        )
    }
}
