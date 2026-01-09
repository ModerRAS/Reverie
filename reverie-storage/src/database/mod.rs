//! 数据库存储模块
//!
//! 此模块提供基于 SQLite 的元数据存储实现，而媒体文件存储在 VFS 后端（本地文件系统、S3 等）

pub mod config;
pub mod core;
pub mod track;
pub mod album;
pub mod user_playlist;
pub mod subsonic;

// 重新导出主要类型
pub use config::DatabaseConfig;
pub use core::DatabaseStorage;
