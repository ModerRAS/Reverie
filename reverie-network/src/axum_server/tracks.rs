//! 曲目处理器
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
    dto::{ErrorResponse, ListResponse, TrackResponse},
    subsonic,
};
use reverie_storage::TrackStorage;

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

/// 用于搜索的查询参数
#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

/// 列出曲目处理程序
pub async fn list_tracks_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    match state
        .storage
        .list_tracks(pagination.limit, pagination.offset)
        .await
    {
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

/// 获取单个曲目处理程序
pub async fn get_track_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_track(id).await {
        Ok(Some(track)) => (
            StatusCode::OK,
            Json(TrackResponse {
                id: track.id,
                title: track.title,
                album_id: track.album_id,
                artist_id: track.artist_id,
                duration: track.duration,
                track_number: track.track_number,
                disc_number: track.disc_number,
                year: track.year,
                genre: track.genre,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Track {} not found", id),
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

/// 搜索曲目处理程序
pub async fn search_tracks_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Query(search): Query<SearchQuery>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    match state.storage.search_tracks(&search.q).await {
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

/// 创建曲目路由
pub fn create_router<S>() -> Router
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/tracks", get(list_tracks_handler::<S>))
        .route("/api/tracks/:id", get(get_track_handler::<S>))
        .route("/api/tracks/search", get(search_tracks_handler::<S>))
}
