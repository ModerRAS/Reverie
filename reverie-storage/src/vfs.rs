//! 使用 OpenDAL 的虚拟文件系统抽象
//!
//! 此模块通过 Apache OpenDAL 为不同的存储后端（本地文件系统、S3、Azure Blob 等）
//! 提供统一的文件操作接口。

use async_trait::async_trait;
use bytes::Bytes;
use opendal::{Entry, EntryMode, Metadata, Operator};
use std::sync::Arc;

use crate::error::{Result, StorageError};

/// 虚拟文件系统配置
#[derive(Debug, Clone)]
pub struct VfsConfig {
    /// 存储后端的方案（例如："fs"、"s3"、"azblob"、"gcs"）
    pub scheme: String,
    /// 后端特定的配置选项
    pub options: std::collections::HashMap<String, String>,
}

impl VfsConfig {
    /// 创建本地文件系统 VFS 配置
    pub fn local(root: impl Into<String>) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("root".to_string(), root.into());
        Self {
            scheme: "fs".to_string(),
            options,
        }
    }

    /// 创建 S3 兼容存储配置
    pub fn s3(
        bucket: impl Into<String>,
        region: impl Into<String>,
        endpoint: Option<String>,
        access_key_id: Option<String>,
        secret_access_key: Option<String>,
    ) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("bucket".to_string(), bucket.into());
        options.insert("region".to_string(), region.into());
        if let Some(ep) = endpoint {
            options.insert("endpoint".to_string(), ep);
        }
        if let Some(ak) = access_key_id {
            options.insert("access_key_id".to_string(), ak);
        }
        if let Some(sk) = secret_access_key {
            options.insert("secret_access_key".to_string(), sk);
        }
        Self {
            scheme: "s3".to_string(),
            options,
        }
    }

    /// 创建 Azure Blob 存储配置
    pub fn azblob(
        container: impl Into<String>,
        account_name: impl Into<String>,
        account_key: Option<String>,
        endpoint: Option<String>,
    ) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("container".to_string(), container.into());
        options.insert("account_name".to_string(), account_name.into());
        if let Some(key) = account_key {
            options.insert("account_key".to_string(), key);
        }
        if let Some(ep) = endpoint {
            options.insert("endpoint".to_string(), ep);
        }
        Self {
            scheme: "azblob".to_string(),
            options,
        }
    }

    /// 创建 Google Cloud Storage 配置
    pub fn gcs(
        bucket: impl Into<String>,
        credential: Option<String>,
        endpoint: Option<String>,
    ) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("bucket".to_string(), bucket.into());
        if let Some(cred) = credential {
            options.insert("credential".to_string(), cred);
        }
        if let Some(ep) = endpoint {
            options.insert("endpoint".to_string(), ep);
        }
        Self {
            scheme: "gcs".to_string(),
            options,
        }
    }

    /// 创建内存存储配置（用于测试）
    pub fn memory() -> Self {
        Self {
            scheme: "memory".to_string(),
            options: std::collections::HashMap::new(),
        }
    }

    /// 创建 WebDAV 存储配置
    pub fn webdav(
        endpoint: impl Into<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("endpoint".to_string(), endpoint.into());
        if let Some(user) = username {
            options.insert("username".to_string(), user);
        }
        if let Some(pass) = password {
            options.insert("password".to_string(), pass);
        }
        Self {
            scheme: "webdav".to_string(),
            options,
        }
    }

    /// 创建 SFTP 存储配置
    pub fn sftp(
        endpoint: impl Into<String>,
        root: impl Into<String>,
        user: impl Into<String>,
        key: Option<String>,
    ) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("endpoint".to_string(), endpoint.into());
        options.insert("root".to_string(), root.into());
        options.insert("user".to_string(), user.into());
        if let Some(k) = key {
            options.insert("key".to_string(), k);
        }
        Self {
            scheme: "sftp".to_string(),
            options,
        }
    }
}

/// 来自 VFS 的文件元数据
#[derive(Debug, Clone)]
pub struct VfsMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

impl From<Metadata> for VfsMetadata {
    fn from(meta: Metadata) -> Self {
        Self {
            size: meta.content_length(),
            is_file: meta.mode() == EntryMode::FILE,
            is_dir: meta.mode() == EntryMode::DIR,
            // OpenDAL's last_modified returns jiff::Timestamp
            // Convert by parsing its RFC3339 string representation
            last_modified: meta.last_modified().and_then(|t| {
                chrono::DateTime::parse_from_rfc3339(&t.to_string())
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            content_type: meta.content_type().map(|s| s.to_string()),
            etag: meta.etag().map(|s| s.to_string()),
        }
    }
}

/// 来自 VFS 的目录条目
#[derive(Debug, Clone)]
pub struct VfsEntry {
    pub path: String,
    pub metadata: VfsMetadata,
}

impl From<Entry> for VfsEntry {
    fn from(entry: Entry) -> Self {
        Self {
            path: entry.path().to_string(),
            metadata: entry.metadata().clone().into(),
        }
    }
}

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

/// 基于 OpenDAL 的 VFS 实现
#[derive(Clone)]
pub struct OpendalVfs {
    operator: Operator,
    config: VfsConfig,
}

impl OpendalVfs {
    /// 从配置创建新的 OpenDAL VFS
    pub fn new(config: VfsConfig) -> Result<Self> {
        let operator = Self::build_operator(&config)?;
        Ok(Self { operator, config })
    }

    /// 从配置构建 OpenDAL 操作符
    fn build_operator(config: &VfsConfig) -> Result<Operator> {
        use opendal::services::*;

        let op = match config.scheme.as_str() {
            "fs" => {
                let mut builder = Fs::default();
                if let Some(root) = config.options.get("root") {
                    builder = builder.root(root);
                }
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            "memory" => {
                let builder = Memory::default();
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            #[cfg(feature = "vfs-s3")]
            "s3" => {
                let mut builder = S3::default();
                if let Some(bucket) = config.options.get("bucket") {
                    builder = builder.bucket(bucket);
                }
                if let Some(region) = config.options.get("region") {
                    builder = builder.region(region);
                }
                if let Some(endpoint) = config.options.get("endpoint") {
                    builder = builder.endpoint(endpoint);
                }
                if let Some(ak) = config.options.get("access_key_id") {
                    builder = builder.access_key_id(ak);
                }
                if let Some(sk) = config.options.get("secret_access_key") {
                    builder = builder.secret_access_key(sk);
                }
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            #[cfg(feature = "vfs-azblob")]
            "azblob" => {
                let mut builder = Azblob::default();
                if let Some(container) = config.options.get("container") {
                    builder = builder.container(container);
                }
                if let Some(account) = config.options.get("account_name") {
                    builder = builder.account_name(account);
                }
                if let Some(key) = config.options.get("account_key") {
                    builder = builder.account_key(key);
                }
                if let Some(endpoint) = config.options.get("endpoint") {
                    builder = builder.endpoint(endpoint);
                }
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            #[cfg(feature = "vfs-gcs")]
            "gcs" => {
                let mut builder = Gcs::default();
                if let Some(bucket) = config.options.get("bucket") {
                    builder = builder.bucket(bucket);
                }
                if let Some(cred) = config.options.get("credential") {
                    builder = builder.credential(cred);
                }
                if let Some(endpoint) = config.options.get("endpoint") {
                    builder = builder.endpoint(endpoint);
                }
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            #[cfg(feature = "vfs-webdav")]
            "webdav" => {
                let mut builder = Webdav::default();
                if let Some(endpoint) = config.options.get("endpoint") {
                    builder = builder.endpoint(endpoint);
                }
                if let Some(user) = config.options.get("username") {
                    builder = builder.username(user);
                }
                if let Some(pass) = config.options.get("password") {
                    builder = builder.password(pass);
                }
                Operator::new(builder)
                    .map_err(|e| StorageError::Unavailable(e.to_string()))?
                    .finish()
            }
            _ => {
                return Err(StorageError::Unavailable(format!(
                    "Unsupported VFS scheme: {}",
                    config.scheme
                )));
            }
        };

        Ok(op)
    }

    /// 获取底层 OpenDAL 操作符
    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    /// 获取 VFS 配置
    pub fn config(&self) -> &VfsConfig {
        &self.config
    }
}

#[async_trait]
impl Vfs for OpendalVfs {
    async fn read(&self, path: &str) -> Result<Bytes> {
        self.operator
            .read(path)
            .await
            .map(|buf| buf.to_bytes())
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn read_range(&self, path: &str, offset: u64, size: u64) -> Result<Bytes> {
        self.operator
            .read_with(path)
            .range(offset..offset + size)
            .await
            .map(|buf| buf.to_bytes())
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn write(&self, path: &str, data: Bytes) -> Result<()> {
        let _ = self
            .operator
            .write(path, data)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))?;
        Ok(())
    }

    async fn append(&self, path: &str, data: Bytes) -> Result<()> {
        // OpenDAL doesn't have native append, so we read + write
        let existing = match self.read(path).await {
            Ok(existing) => existing,
            Err(_) => Bytes::new(),
        };
        let mut combined = existing.to_vec();
        combined.extend_from_slice(&data);
        self.write(path, Bytes::from(combined)).await
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.operator
            .delete(path)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        self.operator
            .exists(path)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn stat(&self, path: &str) -> Result<VfsMetadata> {
        self.operator
            .stat(path)
            .await
            .map(VfsMetadata::from)
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>> {
        let entries = self
            .operator
            .list(path)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(entries.into_iter().map(VfsEntry::from).collect())
    }

    async fn list_recursive(&self, path: &str) -> Result<Vec<VfsEntry>> {
        let entries = self
            .operator
            .list_with(path)
            .recursive(true)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))?;

        Ok(entries.into_iter().map(VfsEntry::from).collect())
    }

    async fn create_dir(&self, path: &str) -> Result<()> {
        // Ensure path ends with /
        let dir_path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{}/", path)
        };
        self.operator
            .create_dir(&dir_path)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn copy(&self, from: &str, to: &str) -> Result<()> {
        self.operator
            .copy(from, to)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }

    async fn rename(&self, from: &str, to: &str) -> Result<()> {
        self.operator
            .rename(from, to)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
    }
}

/// 提供 Arc 共享的 VFS 包装器
pub type SharedVfs = Arc<dyn Vfs>;

/// 从配置创建共享 VFS
pub fn create_vfs(config: VfsConfig) -> Result<SharedVfs> {
    Ok(Arc::new(OpendalVfs::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_vfs() {
        let vfs = OpendalVfs::new(VfsConfig::memory()).unwrap();

        // 写入
        vfs.write("test.txt", Bytes::from("hello world"))
            .await
            .unwrap();

        // 读取
        let data = vfs.read("test.txt").await.unwrap();
        assert_eq!(&data[..], b"hello world");

        // 存在性检查
        assert!(vfs.exists("test.txt").await.unwrap());
        assert!(!vfs.exists("nonexistent.txt").await.unwrap());

        // 删除
        vfs.delete("test.txt").await.unwrap();
        assert!(!vfs.exists("test.txt").await.unwrap());
    }
}
