// Resume.
//
// C-63: resumption never reuses stale state. Stale commitments and
// epochs must be shown as invalidated when the backend says so.
// No silent continuation.
//
// This module is the runtime side of safe-stop -> preserve -> resume
// semantics. In Phase 1 we implement preserve/resume boundaries, with
// Phase 8 completing explicit resumption semantics.

pub struct ResumeAnchor {
    pub task_id: String,
    pub preserved_state_summary: String,
    pub valid: bool,
}

pub struct ResumeRequest {
    pub task_id: String,
    pub explicit: bool,
}

pub struct ResumeOutcome {
    pub resumed: bool,
    pub reason: String,
    pub new_state: crate::core::orchestrator::progress::TaskState,
}

pub struct Resume;

impl Resume {
    pub fn attempt(_anchor: &ResumeAnchor, _request: &ResumeRequest) -> ResumeOutcome {
        // Phase 1 placeholder: explicit resume boundary only.
        ResumeOutcome {
            resumed: false,
            reason: "Resume is scheduled; explicit resumption path not implemented yet".into(),
            new_state: crate::core::orchestrator::progress::TaskState::Aborted,
        }
    }
}
