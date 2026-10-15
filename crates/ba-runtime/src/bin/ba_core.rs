//! `ba-core` — the Browser-Agent application core process.
//!
//! Two modes:
//!
//! * `ba-core --stdio` (default) reads one JSON command per line from stdin and
//!   writes one JSON event per line to stdout. The managed host adapter and the
//!   first-party UI both attach to this process; the UI's frames are treated as
//!   untrusted input.
//! * `ba-core export-copy` prints the canonical copy contract that the UI
//!   imports, so a UI change cannot silently invent its own status wording.
//!
//! There is no HTTP listener, no CDP endpoint and no privileged bridge in this
//! process (Section 14.2).

// The stdio loop is this process's entire purpose, so printing is the intended
// behaviour. The allowance is scoped to `main` rather than the crate.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::io::{BufRead, Write};

use ba_domain::UnixMillis;
use ba_ipc::SequenceTracker;
use ba_runtime::{AppCore, encode_outbound};

#[allow(clippy::print_stdout, clippy::print_stderr)]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("export-copy") => {
            print!("{}", AppCore::copy_contract());
        }
        Some("--help") | Some("-h") => {
            println!("ba-core — Browser-Agent application core");
            println!();
            println!("  ba-core --stdio       read commands on stdin, write events on stdout");
            println!("  ba-core export-copy   print the canonical copy contract JSON");
            println!("  ba-core --help        show this message");
        }
        Some("--stdio") | None => run_stdio(),
        Some(other) => {
            eprintln!("unknown argument `{other}`; try --help");
            std::process::exit(2);
        }
    }
}

#[allow(clippy::print_stdout, clippy::print_stderr)]
fn run_stdio() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut core = AppCore::new();
    let mut tracker = SequenceTracker::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let monotonic_ms = monotonic_millis();
        let now = match UnixMillis::now() {
            Ok(now) => now,
            Err(_) => {
                // A clock that cannot be read is not a reason to fabricate a
                // timestamp; the frame is refused instead.
                write_rejection(
                    &mut core,
                    &mut stdout,
                    "unknown",
                    "the system clock is not usable, so nothing was processed",
                    now_for_rejection(),
                    monotonic_ms,
                );
                continue;
            }
        };

        let command = match ba_runtime::decode_or_reject(trimmed.as_bytes(), &mut tracker) {
            Ok(command) => command,
            Err(error) => {
                write_rejection(
                    &mut core,
                    &mut stdout,
                    "unparsed",
                    &error.to_string(),
                    now,
                    monotonic_ms,
                );
                continue;
            }
        };

        core.tick(monotonic_ms);
        for event in core.handle(&command, now) {
            match encode_outbound(&event, now, monotonic_ms) {
                Ok(line) => {
                    let _ = stdout.write_all(line.as_bytes());
                }
                // If an event cannot be serialized, a four-part error cannot be
                // serialized either, so the honest report is a refusal: the
                // event was not delivered and the interface must not assume it
                // was.
                Err(_) => write_rejection(
                    &mut core,
                    &mut stdout,
                    "event encoding",
                    "the event could not be serialized, so it was not delivered",
                    now,
                    monotonic_ms,
                ),
            }
        }
        let _ = stdout.flush();
    }
}

/// Writes one refusal as a normal enveloped event.
#[allow(clippy::print_stdout, clippy::print_stderr)]
fn write_rejection<W: Write>(
    core: &mut AppCore,
    stdout: &mut W,
    command: &str,
    reason: &str,
    now: UnixMillis,
    monotonic_ms: u64,
) {
    let event = core.reject_frame(command, reason, now);
    if let Ok(line) = encode_outbound(&event, now, monotonic_ms) {
        let _ = stdout.write_all(line.as_bytes());
    }
    let _ = stdout.flush();
}

/// Timestamp used when the wall clock itself is unusable.
///
/// The value is a fixed, clearly-labelled placeholder rather than a fabricated
/// current time; the accompanying reason states that the clock could not be
/// read, and no command is processed with it.
fn now_for_rejection() -> UnixMillis {
    UnixMillis::from_millis(0)
}

fn monotonic_millis() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;

    static START: OnceLock<Instant> = OnceLock::new();
    let start = START.get_or_init(Instant::now);
    u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX)
}
