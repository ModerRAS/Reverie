//! Reverie Core - Domain Models and Business Logic
//!
//! This crate contains the core domain models and business logic for Reverie,
//! independent of storage and network implementation details.

pub mod error;
pub mod models;

pub use error::*;
pub use models::*;
