//! Reverie Network - Network Abstraction Layer
//!
//! This crate provides a trait-based abstraction for network operations,
//! allowing the application to work with different HTTP server implementations
//! and external connection systems through a unified interface.

pub mod traits;
pub mod error;
pub mod dto;

#[cfg(feature = "axum-server")]
pub mod axum_server;

pub use traits::*;
pub use error::*;
pub use dto::*;
