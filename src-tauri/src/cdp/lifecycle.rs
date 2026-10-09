// Lifecycle.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum LifecycleEvent {
    TargetCreated,
    TargetDestroyed,
    FrameAttached,
    FrameDetached,
    LoaderStarted,
    LoaderFinished,
}

pub struct LifecycleWatcher;

impl LifecycleWatcher {
    pub fn subscribe() -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented(
            "LifecycleWatcher::subscribe is scheduled".into(),
        ))
    }
}
