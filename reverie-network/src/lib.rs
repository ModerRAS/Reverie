//! Reverie 网络层 - 网络抽象层
//!
//! 此 crate 提供了基于 trait 的网络操作抽象，
//! 允许应用程序通过统一接口与不同的 HTTP 服务器实现
//! 和外部连接系统一起工作。
pub mod dto;
pub mod error;
pub mod subsonic;
pub mod traits;

#[cfg(feature = "axum-server")]
pub mod axum_server;

#[cfg(test)]
mod tests;

pub use dto::*;
pub use error::*;
pub use traits::*;

// 注意：subsonic 模块是 pub(crate) - 不重新导出
// 使用 reverie_server 访问 Subsonic API 端点
