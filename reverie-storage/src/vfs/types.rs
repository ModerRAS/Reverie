//! 虚拟文件系统类型定义
//!
//! 定义 VFS 使用的核心数据结构

use chrono::{DateTime, Utc};
use opendal::{Entry, Metadata};

/// 来自 VFS 的文件元数据
#[derive(Debug, Clone)]
pub struct VfsMetadata {
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub last_modified: Option<DateTime<Utc>>,
    pub content_type: Option<String>,
    pub etag: Option<String>,
}

impl From<Metadata> for VfsMetadata {
    fn from(meta: Metadata) -> Self {
        Self {
            size: meta.content_length(),
            is_file: meta.mode() == opendal::EntryMode::FILE,
            is_dir: meta.mode() == opendal::EntryMode::DIR,
            // OpenDAL's last_modified returns jiff::Timestamp
            // Convert by parsing its RFC3339 string representation
            last_modified: meta.last_modified().and_then(|t| {
                DateTime::parse_from_rfc3339(&t.to_string())
                    .ok()
                    .map(|dt| dt.with_timezone(&Utc))
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
