//! 基于 Axum 的 HTTP 服务器实现
use async_trait::async_trait;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, get_service},
    Router,
};
use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::{
    dto::*,
    error::{NetworkError, Result},
    subsonic,
    traits::{HttpServer, NetworkConfig},
};
use reverie_storage::{AlbumStorage, ArtistStorage, PlaylistStorage, SubsonicStorage, TrackStorage};

/// 基于 Axum 的 HTTP 服务器。
pub struct AxumServer<S> {
    storage: Arc<S>,
    config: NetworkConfig,
    ui_dir: Option<PathBuf>,
    addr: Arc<RwLock<Option<SocketAddr>>>,
    is_running: Arc<RwLock<bool>>,
}

impl<S> AxumServer<S>
where
    S: TrackStorage
        + AlbumStorage
        + ArtistStorage
        + PlaylistStorage
        + SubsonicStorage
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub fn new(storage: Arc<S>, config: NetworkConfig) -> Self {
        Self {
            storage,
            config,
            ui_dir: None,
            addr: Arc::new(RwLock::new(None)),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// 从给定目录提供构建好的 Web UI (dx 构建输出)。
    ///
    /// 预期结构：
    /// - index.html
    /// - assets/
    /// - wasm/
    pub fn with_ui_dir(mut self, ui_dir: impl Into<PathBuf>) -> Self {
        self.ui_dir = Some(ui_dir.into());
        self
    }

    fn create_ui_router(&self) -> Option<Router<subsonic::SubsonicState<S>>> {
        let ui_dir = self.ui_dir.clone()?;

        let index = ui_dir.join("index.html");
        let assets_dir = ui_dir.join("assets");
        let wasm_dir = ui_dir.join("wasm");

        Some(
            Router::<subsonic::SubsonicState<S>>::new()
                .route("/", get_service(ServeFile::new(index.clone())))
                .route(
                    "/favicon.ico",
                    get_service(ServeFile::new(ui_dir.join("favicon.ico"))),
                )
                .nest_service("/assets", ServeDir::new(assets_dir))
                .nest_service("/wasm", ServeDir::new(wasm_dir))
                // SPA 回退：其他所有内容都指向 index.html
                .route("/*path", get_service(ServeFile::new(index))),
        )
    }

    fn create_router(&self) -> Router {
        let mut router = Router::<subsonic::SubsonicState<S>>::new()
            // 健康检查
            .route("/health", get(health_handler))
            // Subsonic API
            .nest("/rest", subsonic::create_router::<S>())
            // 曲目路由
            .route("/api/tracks", get(list_tracks_handler::<S>))
            .route("/api/tracks/:id", get(get_track_handler::<S>))
            .route("/api/tracks/search", get(search_tracks_handler::<S>))
            // 专辑路由
            .route("/api/albums", get(list_albums_handler::<S>))
            .route("/api/albums/:id", get(get_album_handler::<S>))
            .route("/api/albums/:id/tracks", get(get_album_tracks_handler::<S>))
            // 艺术家路由
            .route("/api/artists", get(list_artists_handler::<S>))
            .route("/api/artists/:id", get(get_artist_handler::<S>))
            .route(
                "/api/artists/:id/albums",
                get(get_artist_albums_handler::<S>),
            )
            // 播放列表路由
            .route("/api/playlists/:id", get(get_playlist_handler::<S>));

        if let Some(ui_router) = self.create_ui_router() {
            router = router.merge(ui_router);
        }

        router
            .with_state(subsonic::SubsonicState::new(Arc::clone(&self.storage)))
            .layer(if self.config.enable_cors {
                CorsLayer::permissive()
            } else {
                CorsLayer::new()
            })
            .layer(TraceLayer::new_for_http())
    }
}

#[async_trait]
impl<S> HttpServer for AxumServer<S>
where
    S: TrackStorage
        + AlbumStorage
        + ArtistStorage
        + PlaylistStorage
        + SubsonicStorage
        + Clone
        + Send
        + Sync
        + 'static,
{
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        let router = self.create_router();

        *self.is_running.write().await = true;
        *self.addr.write().await = Some(addr);

        tracing::info!("正在启动 Axum 服务器 {}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| NetworkError::ServerError(e.to_string()))?;

        axum::serve(listener, router)
            .await
            .map_err(|e| NetworkError::ServerError(e.to_string()))?;

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        *self.is_running.write().await = false;
        *self.addr.write().await = None;
        Ok(())
    }

    fn is_running(&self) -> bool {
        // 注意：这是一个简化检查。
        // 在实际实现中，我们会跟踪服务器任务。
        false
    }

    fn address(&self) -> Option<SocketAddr> {
        // 注意：在实际实现中需要适当的异步上下文。
        None
    }
}

// 用于分页的查询参数
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

// 用于搜索的查询参数
#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

/// 健康检查处理程序
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 曲目处理程序
async fn list_tracks_handler<S>(
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

async fn get_track_handler<S>(
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

async fn search_tracks_handler<S>(
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

/// 专辑处理程序
async fn list_albums_handler<S>(
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

async fn get_album_handler<S>(
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

async fn get_album_tracks_handler<S>(
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

/// 艺术家处理程序
async fn list_artists_handler<S>(
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

async fn get_artist_handler<S>(
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

async fn get_artist_albums_handler<S>(
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

/// 播放列表处理程序
async fn get_playlist_handler<S>(
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
                description: playlist.description,
                user_id: playlist.user_id,
                is_public: playlist.is_public,
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
