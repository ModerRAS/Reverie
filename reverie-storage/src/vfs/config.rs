//! 虚拟文件系统配置
//!
//! 提供不同存储后端的配置结构

use std::collections::HashMap;

/// 虚拟文件系统配置
#[derive(Debug, Clone)]
pub struct VfsConfig {
    /// 存储后端的方案（例如："fs"、"s3"、"azblob"、"gcs"）
    pub scheme: String,
    /// 后端特定的配置选项
    pub options: HashMap<String, String>,
}

impl VfsConfig {
    /// 创建本地文件系统 VFS 配置
    pub fn local(root: impl Into<String>) -> Self {
        let mut options = HashMap::new();
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
        let mut options = HashMap::new();
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
        let mut options = HashMap::new();
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
        let mut options = HashMap::new();
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
            options: HashMap::new(),
        }
    }

    /// 创建 WebDAV 存储配置
    pub fn webdav(
        endpoint: impl Into<String>,
        username: Option<String>,
        password: Option<String>,
    ) -> Self {
        let mut options = HashMap::new();
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
        let mut options = HashMap::new();
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
