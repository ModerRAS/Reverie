//! Reverie Network - Network Abstraction Layer
//!
//! This crate provides a trait-based abstraction for network operations,
//! allowing the application to work with different HTTP server implementations
//! and external connection systems through a unified interface.

pub mod dto;
pub mod error;
pub mod subsonic;
pub mod traits;

#[cfg(feature = "axum-server")]
pub mod axum_server;

#[cfg(test)]
mod tests;

pub use dto::*;
pub use error::*;
pub use subsonic::*;
pub use traits::*;
