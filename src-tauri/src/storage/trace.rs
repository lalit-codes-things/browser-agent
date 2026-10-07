// Trace ingestion surface.
//
// C-126 (bench trace corpus): benchmark traces flow through a typed
//        ingestion boundary; analysis never assumes raw runtime memory.
// C-127: storage schema is release-gated; migrations live under
//        storage/migrations.
//
// StoredTrace (storage/models/trace.rs) is the persisted record; this
// module owns validation and conversion of runtime event streams into
// that record.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Monotonic timestamp string from the backend (canonical time source).
    pub at: String,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TraceError {
    /// Empty event streams are refused: an empty trace is not evidence.
    Empty,
}

/// Validate a raw event stream and produce a persisted record.
pub fn ingest(task_id: String, events: Vec<TraceEvent>, created_at: Option<String>) -> Result<crate::storage::models::trace::StoredTrace, TraceError> {
    if events.is_empty() {
        return Err(TraceError::Empty);
    }
    Ok(crate::storage::models::trace::StoredTrace {
        id: format!("trace-{}", task_id),
        task_id,
        events: events.into_iter().map(|e| format!("{} {}", e.at, e.event)).collect(),
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(at: &str, event: &str) -> TraceEvent {
        TraceEvent { at: at.into(), event: event.into() }
    }

    #[test]
    fn ingests_nonempty_stream() {
        let trace = ingest(
            "T1".into(),
            vec![event("t0", "TASK_STATE RUNNING"), event("t1", "POLICY ALLOW")],
            None,
        )
        .unwrap();
        assert_eq!(trace.task_id, "T1");
        assert_eq!(trace.events.len(), 2);
        assert!(trace.id.starts_with("trace-T1"));
    }

    #[test]
    fn empty_stream_is_refused() {
        assert_eq!(ingest("T1".into(), vec![], None), Err(TraceError::Empty));
    }
}
