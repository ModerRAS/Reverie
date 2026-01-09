//! Reverie - 音乐流媒体服务器
//!
//! Reverie 是一个类似于 Navidrome 的音乐流媒体服务器，使用 Rust 编写，
//! 具有抽象的存储和网络层，以实现灵活性和可扩展性。

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use reverie_server::{run_with_storage, ServerRunConfig};
use reverie_storage::memory::MemoryStorage;

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

    let config = ServerRunConfig::default();
    tracing::info!("正在启动 HTTP 服务器 {}:{}", config.host, config.port);

    run_with_storage(storage.clone(), config).await
}
