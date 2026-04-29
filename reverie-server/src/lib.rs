//! Reverie 服务器应用连接

use anyhow::Result;
use reverie_network::{axum_server::AxumServer, HttpServer, NetworkConfig};
use reverie_storage::{Storage, SubsonicStorage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct ServerRunConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub max_body_size: usize,
    pub timeout_seconds: u64,
    pub ui_dir: Option<PathBuf>,
    /// Optional shutdown receiver. When received, the server stops.
    /// If None, the server runs forever (until process termination).
    pub shutdown_rx: Option<oneshot::Receiver<()>>,
}

impl Default for ServerRunConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4533,
            enable_cors: true,
            max_body_size: 10 * 1024 * 1024,
            timeout_seconds: 30,
            ui_dir: None,
            shutdown_rx: None,
        }
    }
}

pub async fn run_with_storage<S>(storage: Arc<S>, config: ServerRunConfig) -> Result<()>
where
    S: Storage + SubsonicStorage + Clone + 'static,
{
    storage
        .initialize()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize storage: {}", e))?;

    let network_config = NetworkConfig {
        host: config.host.clone(),
        port: config.port,
        enable_cors: config.enable_cors,
        max_body_size: config.max_body_size,
        timeout_seconds: config.timeout_seconds,
    };

    let mut server = AxumServer::new(storage.clone(), network_config.clone());
    if let Some(ui_dir) = config.ui_dir.clone() {
        server = server.with_ui_dir(ui_dir);
    }

    let addr: SocketAddr = format!("{}:{}", network_config.host, network_config.port)
        .parse()
        .expect("Invalid server address");

    // Keep running until shutdown signal is received, or forever if None.
    //
    // When shutdown_rx is provided, we start the server first (it blocks until
    // the server naturally stops, e.g. via axum::serve error), and only then
    // wait for the shutdown signal. This keeps the future alive indefinitely,
    // preventing the test harness's tokio::spawn from completing prematurely.
    if let Some(shutdown_rx) = config.shutdown_rx {
        server
            .start(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
        // Server has stopped — wait for shutdown signal.
        // This keeps the future alive so the caller's tokio::spawn does not
        // complete and kill the server between tests.
        let _ = shutdown_rx.await;
        tracing::info!("Server shutdown complete");
        Ok(())
    } else {
        server
            .start(addr)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
        Ok(())
    }
}
