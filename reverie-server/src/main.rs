//! Reverie - 音乐流媒体服务器
//!
//! Reverie 是一个类似于 Navidrome 的音乐流媒体服务器，使用 Rust 编写，
//! 具有抽象的存储和网络层，以实现灵活性和可扩展性。

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::memory::MemoryStorage;

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

    // 初始化存储后端（此示例使用内存存储）
    let storage = Arc::new(MemoryStorage::new());

    tracing::info!("存储初始化成功");

    let mut config = ServerRunConfig::default();
    // Serve the web UI (if present)
    config.ui_dir = default_ui_dir();
    tracing::info!("正在启动 HTTP 服务器 {}:{}", config.host, config.port);

    run_with_storage(storage.clone(), config).await
}
