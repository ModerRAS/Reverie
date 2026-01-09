//! 网络操作错误类型
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Server error: {0}")]
    ServerError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, NetworkError>;
