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
use reverie_storage::{
    AlbumStorage, ArtistStorage, PlaylistStorage, SubsonicStorage, TrackStorage,
};

pub mod health;
pub mod tracks;
pub mod albums;
pub mod artists;
pub mod playlists;

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
            .route("/health", get(health::health_handler))
            // Subsonic API
            .nest("/rest", subsonic::create_router::<S>())
            // 曲目路由
            .route("/api/tracks", get(tracks::list_tracks_handler::<S>))
            .route("/api/tracks/:id", get(tracks::get_track_handler::<S>))
            .route("/api/tracks/search", get(tracks::search_tracks_handler::<S>))
            // 专辑路由
            .route("/api/albums", get(albums::list_albums_handler::<S>))
            .route("/api/albums/:id", get(albums::get_album_handler::<S>))
            .route("/api/albums/:id/tracks", get(albums::get_album_tracks_handler::<S>))
            // 艺术家路由
            .route("/api/artists", get(artists::list_artists_handler::<S>))
            .route("/api/artists/:id", get(artists::get_artist_handler::<S>))
            .route(
                "/api/artists/:id/albums",
                get(artists::get_artist_albums_handler::<S>),
            )
            // 播放列表路由
            .route("/api/playlists/:id", get(playlists::get_playlist_handler::<S>));

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
