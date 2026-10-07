// Verification evidence tiers.
//
// C-93: VERIFIED_SUCCESS requires sufficiently independent evidence.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EvidenceTier {
    Insufficient,
    Independent,
    StronglyIndependent,
}
