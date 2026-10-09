// Browser subsystem.
//
// C-04: --remote-debugging-pipe; no TCP debugging endpoint.
// C-05: sandbox and site isolation remain enabled; forbidden flags not used.
// C-12: high-risk profiles ephemeral by default.
// C-13: persistent per-site high-risk profiles require opt-in, expiry,
//        storage caps, purge.
// C-68: one high-risk Chromium process maximum; high-risk work queued and
//        memory-aware.
// C-69: extensions off; sync/component-update/pings disabled where compatible;
//        unnecessary permissions denied; downloads quarantined and never
//        automatically opened.
// C-78: jetsam-aware safe-state preservation; watchdogs, caps, safe termination.
// C-88: hybrid typing primitive lives in execution/typing.rs; this module owns
//        browser-side typing hooks.
//
// Runtime attach/detach lifecycle is owned here. The orchestrator consumes a
// runtime handle via the browser runtime controller. The frontend projects the
// attached state through the typed event union.

pub mod controller;
pub mod action;
pub mod downloads;
pub mod highrisk_queue;
pub mod permissions;
pub mod process;
pub mod profiles;
pub mod watchdog;
