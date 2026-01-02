//! Reverie Storage - Storage Abstraction Layer
//!
//! This crate provides a trait-based abstraction for storage operations,
//! allowing the application to work with different storage backends
//! (filesystem, database, cloud storage, etc.) through a unified interface.

pub mod traits;
pub mod error;

#[cfg(feature = "filesystem")]
pub mod filesystem;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "database")]
pub mod database;

pub use traits::*;
pub use error::*;
