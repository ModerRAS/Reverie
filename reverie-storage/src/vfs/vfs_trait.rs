//! 虚拟文件系统 trait 定义
//!
//! 定义不同存储后端必须实现的统一接口

use async_trait::async_trait;
use bytes::Bytes;

use crate::error::Result;
use super::types::{VfsEntry, VfsMetadata};

/// 虚拟文件系统 trait - 不同存储后端的抽象
#[async_trait]
pub trait Vfs: Send + Sync {
    /// 将文件完整读取到内存中
    async fn read(&self, path: &str) -> Result<Bytes>;

    /// 从文件中读取指定范围的字节
    async fn read_range(&self, path: &str, offset: u64, size: u64) -> Result<Bytes>;

    /// 向文件写入数据（创建或覆盖）
    async fn write(&self, path: &str, data: Bytes) -> Result<()>;

    /// 向文件追加数据
    async fn append(&self, path: &str, data: Bytes) -> Result<()>;

    /// 删除文件
    async fn delete(&self, path: &str) -> Result<()>;

    /// 检查路径是否存在
    async fn exists(&self, path: &str) -> Result<bool>;

    /// 获取路径的元数据
    async fn stat(&self, path: &str) -> Result<VfsMetadata>;

    /// 列出目录中的条目
    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>>;

    /// 递归列出条目
    async fn list_recursive(&self, path: &str) -> Result<Vec<VfsEntry>>;

    /// 创建目录（必要时创建父目录）
    async fn create_dir(&self, path: &str) -> Result<()>;

    /// 复制文件
    async fn copy(&self, from: &str, to: &str) -> Result<()>;

    /// 重命名/移动文件
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
}
