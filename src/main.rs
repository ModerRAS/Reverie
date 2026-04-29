use anyhow::Result;
use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::{DatabaseConfig, DatabaseStorage, Storage, VfsConfig};
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

    let db_path = std::env::var("REVERIE_DB_PATH").unwrap_or_else(|_| "reverie.db".to_string());
    let music_dir = std::env::var("REVERIE_MUSIC_DIR").unwrap_or_else(|_| "./music".to_string());
    let auto_scan = std::env::var("REVERIE_AUTO_SCAN").unwrap_or_else(|_| "1".to_string()) != "0";

    tracing::info!(db_path = %db_path, music_dir = %music_dir, auto_scan, "存储配置");

    let storage = Arc::new(DatabaseStorage::new(DatabaseConfig::new(
        db_path,
        VfsConfig::local(music_dir),
    ))
    .await?);
    storage.initialize().await?;
    if auto_scan {
        if let Err(e) = storage.perform_scan("/").await {
            tracing::warn!(error = %e, "启动扫描失败（将继续启动服务）");
        }
    }

    let mut config = ServerRunConfig::default();
    // Serve the web UI (if present)
    config.ui_dir = default_ui_dir();

    run_with_storage(storage, config).await
}
