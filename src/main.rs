use anyhow::Result;
use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::memory::MemoryStorage;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn default_ui_dir() -> Option<PathBuf> {
    // Allow overriding for packaged deployments
    if let Ok(dir) = std::env::var("REVERIE_UI_DIR") {
        if !dir.trim().is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    // During `cargo run` in the workspace, the build script copies UI into target/<profile>/ui.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let candidate = manifest_dir.join("target").join(profile).join("ui");
    if candidate.join("index.html").exists() {
        Some(candidate)
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reverie=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let storage = Arc::new(MemoryStorage::new());

    let mut config = ServerRunConfig::default();
    // Serve the web UI (if present)
    config.ui_dir = default_ui_dir();

    run_with_storage(storage, config).await
}
