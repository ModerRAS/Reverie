//! Reverie - A Music Streaming Server
//!
//! Reverie is a music streaming server similar to Navidrome, written in Rust
//! with abstracted storage and network layers for flexibility and extensibility.

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use reverie_storage::memory::MemoryStorage;
use reverie_server::{run_with_storage, ServerRunConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reverie=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Reverie Music Server");

    // Initialize storage backend (using in-memory for this example)
    let storage = Arc::new(MemoryStorage::new());

    tracing::info!("Storage initialized successfully");

    let config = ServerRunConfig::default();
    tracing::info!("Starting HTTP server on {}:{}", config.host, config.port);

    run_with_storage(storage.clone(), config).await
}
