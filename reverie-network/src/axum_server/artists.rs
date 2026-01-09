//! 艺术家处理器
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
    dto::{AlbumResponse, ArtistResponse, ErrorResponse, ListResponse},
    subsonic,
};
use reverie_storage::{AlbumStorage, ArtistStorage};

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

/// 列出艺术家处理程序
pub async fn list_artists_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Query(pagination): Query<PaginationQuery>,
) -> impl IntoResponse
where
    S: ArtistStorage + Clone + Send + Sync + 'static,
{
    match state
        .storage
        .list_artists(pagination.limit, pagination.offset)
        .await
    {
        Ok(artists) => {
            let responses: Vec<ArtistResponse> = artists
                .into_iter()
                .map(|a| ArtistResponse {
                    id: a.id,
                    name: a.name,
                    bio: a.bio,
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

/// 获取单个艺术家处理程序
pub async fn get_artist_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: ArtistStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_artist(id).await {
        Ok(Some(artist)) => (
            StatusCode::OK,
            Json(ArtistResponse {
                id: artist.id,
                name: artist.name,
                bio: artist.bio,
            }),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Artist {} not found", id),
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

/// 获取艺术家专辑处理程序
pub async fn get_artist_albums_handler<S>(
    State(state): State<subsonic::SubsonicState<S>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse
where
    S: AlbumStorage + Clone + Send + Sync + 'static,
{
    match state.storage.get_albums_by_artist(id).await {
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

/// 创建艺术家路由
pub fn create_router<S>() -> Router
where
    S: ArtistStorage + AlbumStorage + Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/api/artists", get(list_artists_handler::<S>))
        .route("/api/artists/:id", get(get_artist_handler::<S>))
        .route("/api/artists/:id/albums", get(get_artist_albums_handler::<S>))
}
