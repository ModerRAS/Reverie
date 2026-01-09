//! 核心存储 traits
//!
//! 定义了音乐曲目、专辑和艺术家等核心实体的存储接口。

use crate::error::Result;
use async_trait::async_trait;
use reverie_core::{Album, Artist, Track};
use uuid::Uuid;

/// 用于管理音乐曲目存储的 trait
#[async_trait]
pub trait TrackStorage: Send + Sync {
    /// 通过 ID 获取曲目
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>>;

    /// 获取所有曲目
    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>>;

    /// 保存曲目
    async fn save_track(&self, track: &Track) -> Result<()>;

    /// 删除曲目
    async fn delete_track(&self, id: Uuid) -> Result<()>;

    /// 按标题搜索曲目
    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>>;

    /// 按专辑获取曲目
    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>>;

    /// 按艺术家获取曲目
    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>>;
}

/// 用于管理专辑存储的 trait
#[async_trait]
pub trait AlbumStorage: Send + Sync {
    /// 通过 ID 获取专辑
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>>;

    /// 获取所有专辑
    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>>;

    /// 保存专辑
    async fn save_album(&self, album: &Album) -> Result<()>;

    /// 删除专辑
    async fn delete_album(&self, id: Uuid) -> Result<()>;

    /// 按艺术家获取专辑
    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>>;
}

/// 用于管理艺术家存储的 trait
#[async_trait]
pub trait ArtistStorage: Send + Sync {
    /// 通过 ID 获取艺术家
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>>;

    /// 获取所有艺术家
    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>>;

    /// 保存艺术家
    async fn save_artist(&self, artist: &Artist) -> Result<()>;

    /// 删除艺术家
    async fn delete_artist(&self, id: Uuid) -> Result<()>;
}
