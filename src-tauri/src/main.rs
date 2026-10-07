// Browser Agent — Tauri application entry point
//
// This app is a local operations console, not a SaaS product.
// We keep the dependency surface small and the trust boundary in Rust.
// (C-02, C-156)

fn main() {
    // Initialize logging/tracing in release builds when enabled.
    // For Phase 1 we keep this minimal and explicit.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    browser_agent_lib::init();

    // Real Tauri bootstrap happens here once tauri.conf.json is wired.
    // We do not start the app behind a fake success path.
    tracing::info!("browser-agent runtime placeholder initialized");
}
