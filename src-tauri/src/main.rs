// Browser Agent — Tauri application entry point
//
// This app is a local operations console, not a SaaS product.
// We keep the dependency surface small and the trust boundary in Rust.
// (C-02, C-156)

use std::io::Write;
use std::path::Path;

/// Local-only diagnostic log for startup panics.
///
/// Startup failures that end in abort() (for example a panic inside the
/// macOS event-loop callback, which cannot unwind across the Objective-C
/// boundary) would otherwise leave no actionable trace. This path receives
/// panic payload, location, and backtrace only — never secrets, page
/// contents, or other user data.
fn panic_log_path() -> std::path::PathBuf {
    std::env::temp_dir().join("browser-agent-panic.log")
}

/// Record a panic to a local diagnostic log.
///
/// Kept separate from hook installation so the writer is unit-testable.
fn write_panic_diagnostics(
    path: &Path,
    payload: &str,
    location: &str,
    backtrace: &str,
) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let unix_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    writeln!(file, "=== browser-agent panic ===")?;
    writeln!(file, "time_unix_ms: {unix_ms}")?;
    writeln!(file, "payload: {payload}")?;
    writeln!(file, "location: {location}")?;
    writeln!(file, "backtrace:\n{backtrace}")?;
    file.sync_data()?;
    Ok(())
}

/// Install an early panic hook that records the ORIGINAL panic payload,
/// location, and backtrace.
///
/// Panics raised inside non-unwinding FFI/event-loop callbacks abort the
/// process after hooks run, so installing this before any startup work is
/// what makes those failures diagnosable. The default hook still prints
/// to stderr.
fn install_panic_diagnostics(log_path: std::path::PathBuf) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        default_hook(info);

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "non-string panic payload".to_string()
        };
        let location = info
            .location()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "unknown location".to_string());
        let backtrace = std::backtrace::Backtrace::force_capture().to_string();

        let _ = write_panic_diagnostics(&log_path, &payload, &location, &backtrace);
    }));
}

fn main() {
    // First: make every subsequent startup failure diagnosable, including
    // panics inside the event loop that end in abort().
    install_panic_diagnostics(panic_log_path());

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    tracing::info!(stage = "process_entry", "browser-agent starting");

    if let Err(error) = browser_agent_lib::run() {
        tracing::error!(stage = "run", "browser-agent failed to start: {error}");
        eprintln!("Browser Agent failed to start: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_log(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "browser-agent-panic-test-{}-{name}.log",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        path
    }

    #[test]
    fn writer_records_payload_location_and_backtrace() {
        let path = scratch_log("writer");
        write_panic_diagnostics(&path, "test payload", "src/main.rs:1:1", "frame-zero")
            .expect("diagnostics write must succeed");
        let content = std::fs::read_to_string(&path).expect("read back");
        assert!(content.contains("payload: test payload"));
        assert!(content.contains("location: src/main.rs:1:1"));
        assert!(content.contains("frame-zero"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn writer_appends_across_crashes() {
        let path = scratch_log("append");
        write_panic_diagnostics(&path, "first", "a", "b").expect("first write");
        write_panic_diagnostics(&path, "second", "c", "d").expect("second write");
        let content = std::fs::read_to_string(&path).expect("read back");
        assert!(content.contains("payload: first"));
        assert!(content.contains("payload: second"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn installed_hook_captures_panicking_thread_payload() {
        let path = scratch_log("hook");
        install_panic_diagnostics(path.clone());

        let handle = std::thread::spawn(|| {
            panic!("hook-capture-payload");
        });
        // The test harness catches panics per thread; the hook must have
        // run with the original payload before join returns.
        let _ = handle.join();

        let content = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            content.contains("hook-capture-payload"),
            "hook must record the original panic payload; got: {content}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
