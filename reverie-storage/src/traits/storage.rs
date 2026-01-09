//! 组合存储 trait
//!
//! 定义了包含所有存储操作的组合 trait。

use crate::error::Result;
use async_trait::async_trait;
use super::{TrackStorage, AlbumStorage, ArtistStorage, UserStorage, PlaylistStorage, FileStorage};

/// 组合存储 trait，包含所有存储操作
#[async_trait]
pub trait Storage:
    TrackStorage + AlbumStorage + ArtistStorage + UserStorage + PlaylistStorage + FileStorage
{
    /// 初始化存储后端
    async fn initialize(&self) -> Result<()>;

    /// 关闭存储后端
    async fn close(&self) -> Result<()>;

    /// 检查存储是否健康
    async fn health_check(&self) -> Result<bool>;
}
