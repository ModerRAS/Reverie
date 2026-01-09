//! 用户和播放列表存储 traits
//!
//! 定义了用户和播放列表相关的存储接口。

use crate::error::Result;
use async_trait::async_trait;
use reverie_core::{Playlist, PlaylistTrack, User};
use uuid::Uuid;

/// 用于管理用户存储的 trait
#[async_trait]
pub trait UserStorage: Send + Sync {
    /// 通过 ID 获取用户
    async fn get_user(&self, id: Uuid) -> Result<Option<User>>;

    /// 通过用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// 获取所有用户
    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>>;

    /// 保存用户
    async fn save_user(&self, user: &User) -> Result<()>;

    /// 删除用户
    async fn delete_user(&self, id: Uuid) -> Result<()>;
}

/// 用于管理播放列表存储的 trait
#[async_trait]
pub trait PlaylistStorage: Send + Sync {
    /// 通过 ID 获取播放列表
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>>;

    /// 按用户获取播放列表
    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>>;

    /// 保存播放列表
    async fn save_playlist(&self, playlist: &Playlist) -> Result<()>;

    /// 删除播放列表
    async fn delete_playlist(&self, id: Uuid) -> Result<()>;

    /// 向播放列表添加曲目
    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()>;

    /// 从播放列表移除曲目
    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()>;

    /// 获取播放列表中的曲目
    async fn get_playlist_tracks(&self, playlist_id: Uuid) -> Result<Vec<PlaylistTrack>>;
}
