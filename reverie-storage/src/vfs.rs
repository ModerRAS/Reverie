//! Virtual File System abstraction using OpenDAL
//!
//! This module provides a unified interface for file operations across different
//! storage backends (local filesystem, S3, Azure Blob, etc.) using Apache OpenDAL.

use async_trait::async_trait;
use bytes::Bytes;
use opendal::{Entry, EntryMode, Metadata, Operator};
use std::sync::Arc;

use crate::error::{Result, StorageError};

/// Virtual File System configuration
#[derive(Debug, Clone)]
pub struct VfsConfig {
    /// The scheme of the storage backend (e.g., "fs", "s3", "azblob", "gcs")
    pub scheme: String,
    /// Backend-specific configuration options
    pub options: std::collections::HashMap<String, String>,
}

impl VfsConfig {
    /// Create a local filesystem VFS configuration
    pub fn local(root: impl Into<String>) -> Self {
        let mut options = std::collections::HashMap::new();
        options.insert("root".to_string(), root.into());
        Self {
            scheme: "fs".to_string(),
            options,
        }
    }

    /// Create an S3-compatible storage configuration
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

    /// Create an Azure Blob Storage configuration
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

    /// Create a Google Cloud Storage configuration
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

    /// Create an in-memory storage configuration (useful for testing)
    pub fn memory() -> Self {
        Self {
            scheme: "memory".to_string(),
            options: std::collections::HashMap::new(),
        }
    }

    /// Create a WebDAV storage configuration
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

    /// Create an SFTP storage configuration
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

/// File metadata from VFS
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

/// Directory entry from VFS
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

/// Virtual File System trait - abstraction over different storage backends
#[async_trait]
pub trait Vfs: Send + Sync {
    /// Read a file completely into memory
    async fn read(&self, path: &str) -> Result<Bytes>;

    /// Read a range of bytes from a file
    async fn read_range(&self, path: &str, offset: u64, size: u64) -> Result<Bytes>;

    /// Write data to a file (creates or overwrites)
    async fn write(&self, path: &str, data: Bytes) -> Result<()>;

    /// Append data to a file
    async fn append(&self, path: &str, data: Bytes) -> Result<()>;

    /// Delete a file
    async fn delete(&self, path: &str) -> Result<()>;

    /// Check if a path exists
    async fn exists(&self, path: &str) -> Result<bool>;

    /// Get metadata for a path
    async fn stat(&self, path: &str) -> Result<VfsMetadata>;

    /// List entries in a directory
    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>>;

    /// List entries recursively
    async fn list_recursive(&self, path: &str) -> Result<Vec<VfsEntry>>;

    /// Create a directory (and parent directories if needed)
    async fn create_dir(&self, path: &str) -> Result<()>;

    /// Copy a file
    async fn copy(&self, from: &str, to: &str) -> Result<()>;

    /// Rename/move a file
    async fn rename(&self, from: &str, to: &str) -> Result<()>;
}

/// OpenDAL-based VFS implementation
#[derive(Clone)]
pub struct OpendalVfs {
    operator: Operator,
    config: VfsConfig,
}

impl OpendalVfs {
    /// Create a new OpenDAL VFS from configuration
    pub fn new(config: VfsConfig) -> Result<Self> {
        let operator = Self::build_operator(&config)?;
        Ok(Self { operator, config })
    }

    /// Build an OpenDAL operator from configuration
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

    /// Get the underlying OpenDAL operator
    pub fn operator(&self) -> &Operator {
        &self.operator
    }

    /// Get the VFS configuration
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
        self.operator
            .write(path, data)
            .await
            .map_err(|e| StorageError::IoError(std::io::Error::other(e.to_string())))
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

/// A wrapper that provides Arc-based sharing of VFS
pub type SharedVfs = Arc<dyn Vfs>;

/// Create a shared VFS from configuration
pub fn create_vfs(config: VfsConfig) -> Result<SharedVfs> {
    Ok(Arc::new(OpendalVfs::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_vfs() {
        let vfs = OpendalVfs::new(VfsConfig::memory()).unwrap();

        // Write
        vfs.write("test.txt", Bytes::from("hello world"))
            .await
            .unwrap();

        // Read
        let data = vfs.read("test.txt").await.unwrap();
        assert_eq!(&data[..], b"hello world");

        // Exists
        assert!(vfs.exists("test.txt").await.unwrap());
        assert!(!vfs.exists("nonexistent.txt").await.unwrap());

        // Delete
        vfs.delete("test.txt").await.unwrap();
        assert!(!vfs.exists("test.txt").await.unwrap());
    }
}
