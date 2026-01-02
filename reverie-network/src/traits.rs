//! Network abstraction traits
//!
//! These traits define the interface for network operations,
//! allowing different HTTP server implementations to be swapped
//! without changing the core application logic.

use async_trait::async_trait;
use crate::error::Result;
use std::net::SocketAddr;

/// Trait for HTTP server implementation
#[async_trait]
pub trait HttpServer: Send + Sync {
    /// Start the HTTP server
    async fn start(&self, addr: SocketAddr) -> Result<()>;

    /// Stop the HTTP server
    async fn stop(&self) -> Result<()>;

    /// Check if the server is running
    fn is_running(&self) -> bool;

    /// Get the address the server is listening on
    fn address(&self) -> Option<SocketAddr>;
}

/// Trait for handling HTTP requests
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Handle an incoming request
    async fn handle_request(&self, request: Request) -> Result<Response>;
}

/// Simplified HTTP request representation
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP methods
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    Head,
}

/// Simplified HTTP response representation
#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn ok(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            headers: vec![],
            body,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: vec![],
            body: b"Not Found".to_vec(),
        }
    }

    pub fn internal_error() -> Self {
        Self {
            status: 500,
            headers: vec![],
            body: b"Internal Server Error".to_vec(),
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
    }
}

/// Trait for streaming media files
#[async_trait]
pub trait MediaStreamer: Send + Sync {
    /// Stream a file to the client
    async fn stream_file(&self, path: &str) -> Result<Vec<u8>>;

    /// Check if transcoding is supported
    fn supports_transcoding(&self) -> bool;

    /// Transcode a file to a different format
    async fn transcode_file(&self, path: &str, format: &str, bitrate: Option<u32>) -> Result<Vec<u8>>;
}

/// Trait for external network connections (e.g., for federation, cloud sync)
#[async_trait]
pub trait ExternalConnection: Send + Sync {
    /// Connect to an external service
    async fn connect(&self, endpoint: &str) -> Result<()>;

    /// Disconnect from the external service
    async fn disconnect(&self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Send data to the external service
    async fn send_data(&self, data: &[u8]) -> Result<()>;

    /// Receive data from the external service
    async fn receive_data(&self) -> Result<Vec<u8>>;
}

/// Configuration for the network layer
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
    pub max_body_size: usize,
    pub timeout_seconds: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4533,
            enable_cors: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
            timeout_seconds: 30,
        }
    }
}
