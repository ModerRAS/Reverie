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

    /// 从文件中读取指定范围的字节
    ///
    /// 默认实现读取整个文件然后切片。后端可以重写以进行更高效的范围读取。
    async fn read_file_range(&self, path: &str, offset: u64, size: u64) -> Result<Vec<u8>> {
        let data = self.read_file(path).await?;
        let start = offset as usize;
        let end = std::cmp::min(start + size as usize, data.len());
        if start >= data.len() {
            return Err(crate::error::StorageError::NotFound(format!(
                "Range start {} exceeds file size {}: {}",
                offset,
                data.len(),
                path
            )));
        }
        Ok(data[start..end].to_vec())
    }

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
