// Abort wiring.
//
// C-146: task abort cancels in-flight generation.
//
// This is a typed control surface. In Phase 1 we expose the abort signal;
// the inference backend wires it to llama.cpp Metal callbacks.

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

pub struct AbortSignal {
    flag: Arc<AtomicBool>,
}

impl AbortSignal {
    pub fn new() -> Self {
        Self { flag: Arc::new(AtomicBool::new(false)) }
    }

    pub fn check(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    pub fn request_abort(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }

    pub fn arc(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }
}

pub struct AbortController {
    signal: AbortSignal,
}

impl AbortController {
    pub fn new() -> Self {
        Self { signal: AbortSignal::new() }
    }

    pub fn signal(&self) -> &AbortSignal {
        &self.signal
    }

    pub fn abort(&self) {
        self.signal.request_abort();
    }
}
