//! 使用 OpenDAL 的虚拟文件系统抽象
//!
//! 此模块通过 Apache OpenDAL 为不同的存储后端（本地文件系统、S3、Azure Blob 等）
//! 提供统一的文件操作接口。

pub mod config;
pub mod opendal;
pub mod vfs_trait;
pub mod types;

// 重新导出主要类型
pub use config::VfsConfig;
pub use opendal::OpendalVfs;
pub use vfs_trait::Vfs;
pub use types::{VfsEntry, VfsMetadata};

use std::sync::Arc;

/// 提供 Arc 共享的 VFS 包装器
pub type SharedVfs = Arc<dyn Vfs>;

/// 从配置创建共享 VFS
pub fn create_vfs(config: VfsConfig) -> Result<SharedVfs, crate::error::StorageError> {
    Ok(Arc::new(OpendalVfs::new(config)?))
}
