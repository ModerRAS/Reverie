//! Reverie - 音乐流媒体服务器
//!
//! Reverie 是一个类似于 Navidrome 的音乐流媒体服务器，使用 Rust 编写，
//! 具有抽象的存储和网络层，以实现灵活性和可扩展性。

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::{DatabaseConfig, DatabaseStorage, Storage, VfsConfig};

fn default_ui_dir() -> Option<PathBuf> {
    // Allow overriding for packaged deployments
    if let Ok(dir) = std::env::var("REVERIE_UI_DIR") {
        if !dir.trim().is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    // When running this crate, `CARGO_MANIFEST_DIR` points to `reverie-server/`.
    // UI is bundled into workspace `target/<profile>/ui`.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent()?.to_path_buf();
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let candidate = workspace_dir.join("target").join(profile).join("ui");
    if candidate.join("index.html").exists() {
        Some(candidate)
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化追踪
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reverie=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("正在启动 Reverie 音乐服务器");

    // 初始化存储后端（默认使用 SQLite + 本地文件系统 VFS）
    let db_path = std::env::var("REVERIE_DB_PATH").unwrap_or_else(|_| "reverie.db".to_string());
    let music_dir = std::env::var("REVERIE_MUSIC_DIR").unwrap_or_else(|_| "./music".to_string());
    let auto_scan = std::env::var("REVERIE_AUTO_SCAN").unwrap_or_else(|_| "1".to_string()) != "0";

    tracing::info!(db_path = %db_path, music_dir = %music_dir, auto_scan, "存储配置");

    let config = DatabaseConfig::new(db_path, VfsConfig::local(music_dir.clone()));
    let storage = Arc::new(DatabaseStorage::new(config).await?);
    storage.initialize().await?;

    if auto_scan {
        // VFS root 已指向 music_dir，因此扫描根目录即可
        if let Err(e) = storage.perform_scan("/").await {
            tracing::warn!(error = %e, "启动扫描失败（将继续启动服务）");
        }
    }

    tracing::info!("存储初始化成功");

    let mut config = ServerRunConfig::default();
    // Serve the web UI (if present)
    config.ui_dir = default_ui_dir();
    tracing::info!(
        env_REVERIE_UI_DIR = std::env::var("REVERIE_UI_DIR")
            .ok()
            .as_deref()
            .unwrap_or(""),
        selected_ui_dir = config
            .ui_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<none>".to_string()),
        "UI 目录选择"
    );
    tracing::info!("正在启动 HTTP 服务器 {}:{}", config.host, config.port);

    run_with_storage(storage.clone(), config).await
}
