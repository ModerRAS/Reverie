//! Error types for Reverie core

use thiserror::Error;

/// Core error types for Reverie
#[derive(Error, Debug)]
pub enum ReverieError {
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, ReverieError>;
