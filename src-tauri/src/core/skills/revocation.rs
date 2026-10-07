// Revocation.
//
// C-102: critical security bugs trigger immediate revocation; in-flight tasks
//        abort safely.

pub struct Revocation;

impl Revocation {
    pub fn revoke(_skill_id: &str, _reason: &str) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented("Revocation::revoke is scheduled".into()))
    }
}
