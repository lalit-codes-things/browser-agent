// CDP subsystem.
//
// C-03: browser control prefers chromiumoxide over CDP, with raw-CDP escape
//        hatch; playwright is test/reference only.
// C-04: chromium launched with --remote-debugging-pipe; no TCP debugging endpoint.
// C-70: Target.setAutoAttach({flatten:true}) validated; stale references invalidated.

pub mod connection;
pub mod contexts;
pub mod frames;
pub mod lifecycle;
pub mod loaders;
pub mod raw;
pub mod sessions;
pub mod targets;
