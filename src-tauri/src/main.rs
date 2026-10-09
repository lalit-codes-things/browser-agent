// Browser Agent — Tauri application entry point
//
// This app is a local operations console, not a SaaS product.
// We keep the dependency surface small and the trust boundary in Rust.
// (C-02, C-156)

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    if let Err(error) = browser_agent_lib::run() {
        eprintln!("Browser Agent failed to start: {error}");
        std::process::exit(1);
    }
}
