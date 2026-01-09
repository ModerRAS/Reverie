//! 数据库存储配置

use crate::vfs::VfsConfig;

/// 数据库存储配置
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// SQLite 数据库路径（例如："reverie.db" 或 ":memory:"）
    pub database_url: String,
    /// 连接池中的最大连接数
    pub max_connections: u32,
    /// 媒体文件存储的 VFS 配置
    pub vfs_config: VfsConfig,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "reverie.db".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::local("./music"),
        }
    }
}

impl DatabaseConfig {
    /// 创建新的数据库配置
    pub fn new(database_url: impl Into<String>, vfs_config: VfsConfig) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 5,
            vfs_config,
        }
    }

    /// 创建内存数据库配置（用于测试）
    pub fn memory() -> Self {
        Self {
            database_url: ":memory:".to_string(),
            max_connections: 1,
            vfs_config: VfsConfig::memory(),
        }
    }
}
