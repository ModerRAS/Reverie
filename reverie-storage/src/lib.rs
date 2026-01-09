//! Reverie Storage - Storage Abstraction Layer
//!
//! This crate provides a trait-based abstraction for storage operations,
//! allowing the application to work with different storage backends
//! (filesystem, database, cloud storage, etc.) through a unified interface.
//!
//! ## Architecture
//!
//! The storage layer is split into two main components:
//!
//! 1. **Metadata Storage** - Stores track, album, artist, user, and playlist information
//!    - `MemoryStorage` - In-memory storage for testing
//!    - `DatabaseStorage` - SQLite-based persistent storage
//!
//! 2. **File Storage (VFS)** - Stores actual media files using OpenDAL
//!    - Local filesystem
//!    - S3-compatible storage (AWS S3, MinIO, etc.)
//!    - Azure Blob Storage
//!    - Google Cloud Storage
//!    - WebDAV
//!    - SFTP
//!
//! ## Usage
//!
//! ```rust,ignore
//! use reverie_storage::{DatabaseStorage, DatabaseConfig, VfsConfig};
//!
//! // Create storage with local filesystem backend
//! let config = DatabaseConfig::new("reverie.db", VfsConfig::local("./music"));
//! let storage = DatabaseStorage::new(config).await?;
//! storage.initialize().await?;
//!
//! // Or with S3 backend
//! let config = DatabaseConfig::new(
//!     "reverie.db",
//!     VfsConfig::s3("my-bucket", "us-east-1", None, None, None)
//! );
//! ```

pub mod error;
pub mod traits;
pub mod vfs;

#[cfg(feature = "filesystem")]
pub mod filesystem;

#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "database")]
pub mod database;

#[cfg(test)]
mod tests;

pub use error::*;
pub use traits::*;
pub use vfs::{create_vfs, OpendalVfs, SharedVfs, Vfs, VfsConfig, VfsEntry, VfsMetadata};

#[cfg(feature = "database")]
pub use database::{DatabaseConfig, DatabaseStorage};
