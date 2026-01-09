//! 播放列表处理器
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use uuid::Uuid;

use crate::{
    dto::{ErrorResponse, PlaylistResponse},
    subsonic,
};
use reverie_storage::PlaylistStorage;

/// 获取播放列表处理程序
pub async fn get_playlist_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: PlaylistStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_playlist(id).await {
        Ok(Some(playlist)) => (
            StatusCode::OK,
            Json(PlaylistResponse {
                id: playlist.id,
                name: playlist.name,
                owner_id: playlist.owner_id,
                is_public: playlist.is_public,
                track_ids: playlist.track_ids,
                created_at: playlist.created_at,
                updated_at: playlist.updated_at,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Playlist {} not found", id),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "storage_error".to_string(),
                message: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// 创建播放列表路由
pub fn create_router<S>() -> Router
where
    S: PlaylistStorage + Clone + Send + Sync + 'static,
{
    Router::new().route("/api/playlists/:id", get(get_playlist_handler::<S>))
}
