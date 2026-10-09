// Application shell modules.
//
// Anchors runtime-side lifecycle, settings, and diagnostics concerns.
// Subsystem code lives under core/, cdp/, browser/, net/, etc.

pub mod diagnostics;
pub mod lifecycle;
pub mod settings;

pub use diagnostics::*;
pub use lifecycle::*;
pub use settings::*;
