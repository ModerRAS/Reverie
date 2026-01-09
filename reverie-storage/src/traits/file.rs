//! 文件存储 traits
//!
//! 定义了文件存储和元数据相关的接口。

use crate::error::Result;
use async_trait::async_trait;

/// 用于文件存储操作的 trait（音频文件、封面图片等）
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// 按路径读取文件
    async fn read_file(&self, path: &str) -> Result<Vec<u8>>;

    /// 写入文件
    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()>;

    /// 检查文件是否存在
    async fn file_exists(&self, path: &str) -> Result<bool>;

    /// 删除文件
    async fn delete_file(&self, path: &str) -> Result<()>;

    /// 列出目录中的文件
    async fn list_files(&self, path: &str) -> Result<Vec<String>>;

    /// 获取文件元数据（大小、修改时间等）
    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata>;
}

/// 文件元数据信息
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub is_file: bool,
    pub is_dir: bool,
}
