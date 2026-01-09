//! Reverie 核心模块 - 领域模型和业务逻辑
//!
//! 此 crate 包含 Reverie 的核心领域模型和业务逻辑，
//! 与存储和网络实现细节无关。

pub mod error;
pub mod models;

#[cfg(test)]
mod tests;

pub use error::*;
pub use models::*;
