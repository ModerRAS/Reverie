//! 用于 API 请求和响应的数据传输对象 (DTOs)
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 曲目信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackResponse {
    pub id: Uuid,
    pub title: String,
    pub album_id: Option<Uuid>,
    pub artist_id: Option<Uuid>,
    pub duration: u32,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// 专辑信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumResponse {
    pub id: Uuid,
    pub name: String,
    pub artist_id: Option<Uuid>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// 艺术家信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistResponse {
    pub id: Uuid,
    pub name: String,
    pub bio: Option<String>,
}

/// 播放列表信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub is_public: bool,
}

/// 创建新播放列表的请求
#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

/// 向播放列表添加曲目的请求
#[derive(Debug, Deserialize)]
pub struct AddTrackToPlaylistRequest {
    pub track_id: Uuid,
}

/// 带分页的通用列表响应
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// 错误响应
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// 健康检查响应
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
