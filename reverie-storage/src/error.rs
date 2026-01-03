//! Error types for storage operations

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Storage unavailable: {0}")]
    Unavailable(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;
