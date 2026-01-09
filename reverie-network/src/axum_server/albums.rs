//! 专辑处理器
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    dto::{AlbumResponse, ErrorResponse, ListResponse, TrackResponse},
    subsonic,
};
use reverie_storage::{AlbumStorage, TrackStorage};

/// 用于分页的查询参数
#[derive(Deserialize)]
struct PaginationQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    50
}

/// 列出专辑处理程序
pub async fn list_albums_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse
where
    S: AlbumStorage + Clone + Send + Sync + 'static,
{
    match state
        .storage
        .list_albums(pagination.limit, pagination.offset)
        .await
    {
        Ok(albums) => {
            let responses: Vec<AlbumResponse> = albums
                .into_iter()
                .map(|a| AlbumResponse {
                    id: a.id,
                    name: a.name,
                    artist_id: a.artist_id,
                    year: a.year,
                    genre: a.genre,
                })
                .collect();

            let total = responses.len();
            (
                StatusCode::OK,
                Json(ListResponse {
                    items: responses,
                    total,
                    limit: pagination.limit,
                    offset: pagination.offset,
                }),
            )
                .into_response()
        }
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

/// 获取单个专辑处理程序
pub async fn get_album_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: AlbumStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_album(id).await {
        Ok(Some(album)) => (
            StatusCode::OK,
            Json(AlbumResponse {
                id: album.id,
                name: album.name,
                artist_id: album.artist_id,
                year: album.year,
                genre: album.genre,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Album {} not found", id),
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

/// 获取专辑曲目处理程序
pub async fn get_album_tracks_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_tracks_by_album(id).await {
        Ok(tracks) => {
            let responses: Vec<TrackResponse> = tracks
                .into_iter()
                .map(|t| TrackResponse {
                    id: t.id,
                    title: t.title,
                    album_id: t.album_id,
                    artist_id: t.artist_id,
                    duration: t.duration,
                    track_number: t.track_number,
                    disc_number: t.disc_number,
                    year: t.year,
                    genre: t.genre,
                })
                .collect();

            (StatusCode::OK, Json(responses)).into_response()
        }
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

/// 创建专辑路由
pub fn create_router<S>() -> Router
where
    S: AlbumStorage + TrackStorage + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/albums", get(list_albums_handler::<S>))
        .route("/api/albums/:id", get(get_album_handler::<S>))
        .route("/api/albums/:id/tracks", get(get_album_tracks_handler::<S>))
}
