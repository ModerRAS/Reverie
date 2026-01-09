//! 基于 OpenDAL 的 VFS 实现
//!
//! 使用 Apache OpenDAL 提供统一的存储后端支持

use async_trait::async_trait;
use bytes::Bytes;
use opendal::Operator;

use crate::error::{Result, StorageError};
use super::config::VfsConfig;
use super::vfs_trait::Vfs;
use super::types::{VfsEntry, VfsMetadata};

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
