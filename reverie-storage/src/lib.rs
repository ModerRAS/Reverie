//! Reverie 存储 - 存储抽象层
//!
//! 该 crate 提供了基于 trait 的存储操作抽象，
//! 允许应用程序通过统一接口使用不同的存储后端
//!（文件系统、数据库、云存储等）。
//!
//! ## 架构
//!
//! 存储层分为两个主要组件：
//!
//! 1. **元数据存储** - 存储曲目、专辑、艺术家、用户和播放列表信息
//!    - `MemoryStorage` - 用于测试的内存存储
//!    - `DatabaseStorage` - 基于 SQLite 的持久化存储
//!
//! 2. **文件存储 (VFS)** - 使用 OpenDAL 存储实际的媒体文件
//!    - 本地文件系统
//!    - S3 兼容存储（AWS S3、MinIO 等）
//!    - Azure Blob 存储
//!    - Google Cloud Storage
//!    - WebDAV
//!    - SFTP
//!
//! ## 用法
//!
//! ```rust,ignore
//! use reverie_storage::{DatabaseStorage, DatabaseConfig, VfsConfig};
//!
//! // 使用本地文件系统后端创建存储
//! let config = DatabaseConfig::new("reverie.db", VfsConfig::local("./music"));
//! let storage = DatabaseStorage::new(config).await?;
//! storage.initialize().await?;
//!
//! // 或使用 S3 后端
//! let config = DatabaseConfig::new(
//!     "reverie.db",
//!     VfsConfig::s3("my-bucket", "us-east-1", None, None, None)
//! );
//! ```

pub mod error;
pub mod traits;
pub mod vfs;

#[cfg(feature = "filesystem")]
pub mod filesystem;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "database")]
pub mod database;

#[cfg(test)]
mod tests;

pub use error::*;
pub use traits::*;
pub use vfs::{create_vfs, OpendalVfs, SharedVfs, Vfs, VfsConfig, VfsEntry, VfsMetadata};

#[cfg(feature = "database")]
pub use database::{DatabaseConfig, DatabaseStorage};
