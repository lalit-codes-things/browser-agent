// High-risk queue.
//
// C-68: browser concurrency has one high-risk Chromium process maximum;
//        high-risk work is queued and memory-aware.

pub struct HighRiskQueue;

impl HighRiskQueue {
    pub fn enqueue(_task_id: &str) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "HighRiskQueue::enqueue is scheduled".into(),
        ))
    }
}
