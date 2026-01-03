//! Reverie - A Music Streaming Server
//!
//! Reverie is a music streaming server similar to Navidrome, written in Rust
//! with abstracted storage and network layers for flexibility and extensibility.

use anyhow::Result;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use reverie_storage::memory::MemoryStorage;
use reverie_storage::Storage;
use reverie_network::{axum_server::AxumServer, NetworkConfig, HttpServer};

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
    
    // Initialize storage
    storage.initialize().await
        .map_err(|e| anyhow::anyhow!("Failed to initialize storage: {}", e))?;

    tracing::info!("Storage initialized successfully");

    // Create network configuration
    let network_config = NetworkConfig {
        host: "127.0.0.1".to_string(),
        port: 4533,
        enable_cors: true,
        max_body_size: 10 * 1024 * 1024,
        timeout_seconds: 30,
    };

    // Create HTTP server
    let server = AxumServer::new(storage.clone(), network_config.clone());
    
    let addr = format!("{}:{}", network_config.host, network_config.port)
        .parse()
        .expect("Invalid server address");

    tracing::info!("Starting HTTP server on {}", addr);

    // Start the server
    server.start(addr).await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;

    Ok(())
}
