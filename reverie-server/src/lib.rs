//! Reverie 服务器应用连接

use anyhow::Result;
use reverie_network::{axum_server::AxumServer, HttpServer, NetworkConfig};
use reverie_storage::{Storage, SubsonicStorage};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ServerRunConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub max_body_size: usize,
    pub timeout_seconds: u64,
    pub ui_dir: Option<PathBuf>,
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

    server
        .start(addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start server: {}", e))?;

    Ok(())
}
