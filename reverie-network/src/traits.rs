//! 网络抽象 traits
//!
//! 这些 traits 定义了网络操作的接口，
//! 允许在不改变核心应用程序逻辑的情况下切换
//! 不同的 HTTP 服务器实现。
use crate::error::Result;
use async_trait::async_trait;
use std::net::SocketAddr;

/// HTTP 服务器实现的 trait
#[async_trait]
pub trait HttpServer: Send + Sync {
    /// 启动 HTTP 服务器
    async fn start(&self, addr: SocketAddr) -> Result<()>;

    /// 停止 HTTP 服务器
    async fn stop(&self) -> Result<()>;

    /// 检查服务器是否正在运行
    fn is_running(&self) -> bool;

    /// 获取服务器正在监听的地址
    fn address(&self) -> Option<SocketAddr>;
}

/// 处理 HTTP 请求的 trait
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// 处理传入的请求
    async fn handle_request(&self, request: Request) -> Result<Response>;
}

/// 简化的 HTTP 请求表示
#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP 方法
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

/// 简化的 HTTP 响应表示
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

/// 流媒体文件的 trait
#[async_trait]
pub trait MediaStreamer: Send + Sync {
    /// 将文件流式传输到客户端
    async fn stream_file(&self, path: &str) -> Result<Vec<u8>>;

    /// 检查是否支持转码
    fn supports_transcoding(&self) -> bool;

    /// 将文件转码为不同格式
    async fn transcode_file(
        &self,
        path: &str,
        format: &str,
        bitrate: Option<u32>,
    ) -> Result<Vec<u8>>;
}

/// 外部网络连接的 trait（例如，用于联合、云同步）
#[async_trait]
pub trait ExternalConnection: Send + Sync {
    /// 连接到外部服务
    async fn connect(&self, endpoint: &str) -> Result<()>;

    /// 断开与外部服务的连接
    async fn disconnect(&self) -> Result<()>;

    /// 检查是否已连接
    fn is_connected(&self) -> bool;

    /// 向外部服务发送数据
    async fn send_data(&self, data: &[u8]) -> Result<()>;

    /// 从外部服务接收数据
    async fn receive_data(&self) -> Result<Vec<u8>>;
}

/// 网络层配置
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
