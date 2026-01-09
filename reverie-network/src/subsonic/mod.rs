//! Subsonic API 实现
//!
//! Reverie 旨在兼容 Subsonic API 1.16.1。
//! 该模块提供了所有 Subsonic API 端点的处理程序。

mod auth;
mod browsing;
mod playlists;
mod users;
pub mod response;

#[cfg(test)]
mod tests;

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reverie_storage::{FileStorage, SubsonicStorage};
use std::{collections::HashMap, sync::Arc};

use response::*;

// 导入子模块处理器
use browsing::*;
use playlists::*;
use users::*;

// === State and Response Helpers ===

#[derive(Clone)]
pub struct SubsonicState<S: Clone> {
    pub storage: Arc<S>,
}

impl<S: Clone> SubsonicState<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
}

/// 根据格式参数返回 JSON 或 XML
fn format_response(params: &HashMap<String, String>, response: SubsonicResponse) -> Response {
    let format = params.get("f").map(|s| s.as_str()).unwrap_or("xml");

    if format == "json" {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&response).unwrap_or_default(),
        )
            .into_response()
    } else {
        // 目前，即使对于 XML 请求也返回 JSON（完整的 XML 支持待办）
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&response).unwrap_or_default(),
        )
            .into_response()
    }
}

fn ok_response(params: &HashMap<String, String>) -> Response {
    format_response(params, SubsonicResponse::ok())
}

fn error_response(params: &HashMap<String, String>, code: i32, message: &str) -> Response {
    format_response(params, SubsonicResponse::error(code, message))
}

/// 创建 Subsonic 路由器。
///
/// 注意：返回的路由器缺少 `SubsonicState<S>`，它旨在嵌套到提供状态的外部路由器中，
/// 通过 `Router::with_state` 实现。
#[cfg(feature = "axum-server")]
pub(crate) fn create_router<S: SubsonicStorage + FileStorage + Clone + 'static>() -> Router<SubsonicState<S>> {
    Router::new()
        // System endpoints
        .route("/ping", get(ping_handler::<S>))
        .route("/getLicense", get(get_license_handler::<S>))
        .route("/getMusicFolders", get(get_music_folders_handler::<S>))
        // Browsing endpoints
        .route("/getIndexes", get(get_indexes_handler::<S>))
        .route("/getMusicDirectory", get(get_music_directory_handler::<S>))
        .route("/getGenres", get(get_genres_handler::<S>))
        .route("/getArtists", get(get_artists_handler::<S>))
        .route("/getArtist", get(get_artist_handler::<S>))
        .route("/getAlbum", get(get_album_handler::<S>))
        .route("/getSong", get(get_song_handler::<S>))
        .route("/getArtistInfo", get(stub_handler::<S>))
        .route("/getArtistInfo2", get(stub_handler::<S>))
        .route("/getAlbumInfo", get(stub_handler::<S>))
        .route("/getAlbumInfo2", get(stub_handler::<S>))
        .route("/getSimilarSongs", get(stub_handler::<S>))
        .route("/getSimilarSongs2", get(stub_handler::<S>))
        .route("/getTopSongs", get(stub_handler::<S>))
        // Album list endpoints
        .route("/getAlbumList", get(get_album_list_handler::<S>))
        .route("/getAlbumList2", get(get_album_list2_handler::<S>))
        .route("/getRandomSongs", get(get_random_songs_handler::<S>))
        .route("/getSongsByGenre", get(get_songs_by_genre_handler::<S>))
        .route("/getNowPlaying", get(get_now_playing_handler::<S>))
        .route("/getStarred", get(get_starred_handler::<S>))
        .route("/getStarred2", get(get_starred2_handler::<S>))
        // Search endpoints
        .route("/search2", get(search2_handler::<S>))
        .route("/search3", get(search3_handler::<S>))
        // Playlist endpoints
        .route("/getPlaylists", get(get_playlists_handler::<S>))
        .route("/getPlaylist", get(get_playlist_handler::<S>))
        .route("/createPlaylist", get(create_playlist_handler::<S>))
        .route("/updatePlaylist", get(update_playlist_handler::<S>))
        .route("/deletePlaylist", get(delete_playlist_handler::<S>))
        // Media retrieval endpoints
        .route("/stream", get(stream_handler::<S>))
        .route("/download", get(download_handler::<S>))
        .route("/getCoverArt", get(get_cover_art_handler::<S>))
        .route("/getLyrics", get(stub_handler::<S>))
        .route("/getLyricsBySongId", get(stub_handler::<S>))
        .route("/getAvatar", get(stub_handler::<S>))
        // Annotation endpoints
        .route("/star", get(star_handler::<S>))
        .route("/unstar", get(unstar_handler::<S>))
        .route("/setRating", get(set_rating_handler::<S>))
        .route("/scrobble", get(scrobble_handler::<S>))
        // Bookmark endpoints
        .route("/getBookmarks", get(stub_handler::<S>))
        .route("/createBookmark", get(stub_handler::<S>))
        .route("/deleteBookmark", get(stub_handler::<S>))
        .route("/getPlayQueue", get(stub_handler::<S>))
        .route("/savePlayQueue", get(stub_handler::<S>))
        // Share endpoints
        .route("/getShares", get(stub_handler::<S>))
        .route("/createShare", get(stub_handler::<S>))
        .route("/updateShare", get(stub_handler::<S>))
        .route("/deleteShare", get(stub_handler::<S>))
        // Internet radio endpoints
        .route("/getInternetRadioStations", get(stub_handler::<S>))
        .route("/createInternetRadioStation", get(stub_handler::<S>))
        .route("/updateInternetRadioStation", get(stub_handler::<S>))
        .route("/deleteInternetRadioStation", get(stub_handler::<S>))
        // User management endpoints
        .route("/getUser", get(get_user_handler::<S>))
        .route("/getUsers", get(get_users_handler::<S>))
        // Scanning endpoints
        .route("/getScanStatus", get(get_scan_status_handler::<S>))
        .route("/startScan", get(start_scan_handler::<S>))
        // OpenSubsonic extensions
        .route("/getOpenSubsonicExtensions", get(stub_handler::<S>))
}

// ===== 系统处理器 =====

/// GET /rest/ping - 测试连接
async fn ping_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    ok_response(&params)
}

/// GET /rest/getLicense - 获取服务器许可证信息
async fn get_license_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let response = SubsonicResponse::ok_with(ResponseData::License(LicenseData {
        license: License { valid: true },
    }));
    format_response(&params, response)
}

/// GET /rest/getMusicFolders - 获取已配置的音乐文件夹
async fn get_music_folders_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_music_folders().await {
        Ok(folders) => {
            let items: Vec<MusicFolderItem> = folders.iter().map(MusicFolderItem::from).collect();
            let response =
                SubsonicResponse::ok_with(ResponseData::MusicFolders(MusicFoldersData {
                    music_folders: MusicFoldersList {
                        music_folder: items,
                    },
                }));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== 未实现端点的存根处理器 =====

/// 未实现端点的存根处理器 - 返回空的 OK 响应
async fn stub_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    ok_response(&params)
}

// ===== 浏览处理器 =====

/// GET /rest/getArtists - 获取所有艺术家
async fn get_artists_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state.storage.get_artists(music_folder_id).await {
        Ok(indexes) => {
            let data = build_artists(&indexes, 0);
            let response = SubsonicResponse::ok_with(ResponseData::Artists(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getArtist - 获取艺术家详情
async fn get_artist_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_artist(id).await {
        Ok(Some(artist)) => {
            let data = ArtistData {
                artist: ArtistWithAlbums::from(&artist),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Artist(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "Artist not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getAlbum - 获取专辑详情
async fn get_album_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_album(id).await {
        Ok(Some(album)) => {
            let data = AlbumData {
                album: AlbumWithSongs::from(&album),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Album(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "Album not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getSong - 获取歌曲详情
async fn get_song_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_song(id).await {
        Ok(Some(song)) => {
            let data = SongData {
                song: Child::from(&song),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Song(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "Song not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== 专辑列表处理器 =====

/// GET /rest/getAlbumList2 - 按类型获取专辑列表（基于 ID3 标签）
async fn get_album_list2_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let list_type = match params.get("type") {
        Some(t) => t.as_str(),
        None => return error_response(&params, 10, "Missing required parameter: type"),
    };

    let size = params.get("size").and_then(|s| s.parse().ok());
    let offset = params.get("offset").and_then(|s| s.parse().ok());
    let from_year = params.get("fromYear").and_then(|s| s.parse().ok());
    let to_year = params.get("toYear").and_then(|s| s.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());
    let music_folder_id = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state
        .storage
        .get_album_list2(
            list_type,
            size,
            offset,
            from_year,
            to_year,
            genre,
            music_folder_id,
        )
        .await
    {
        Ok(albums) => {
            let items: Vec<AlbumID3Item> = albums.iter().map(AlbumID3Item::from).collect();
            let data = AlbumList2Data {
                album_list2: AlbumList2Inner { album: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::AlbumList2(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== 搜索处理器 =====

/// GET /rest/search3 - 使用 ID3 标签搜索
async fn search3_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let query = match params.get("query") {
        Some(q) => q.as_str(),
        None => return error_response(&params, 10, "Missing required parameter: query"),
    };

    let artist_count = params.get("artistCount").and_then(|s| s.parse().ok());
    let artist_offset = params.get("artistOffset").and_then(|s| s.parse().ok());
    let album_count = params.get("albumCount").and_then(|s| s.parse().ok());
    let album_offset = params.get("albumOffset").and_then(|s| s.parse().ok());
    let song_count = params.get("songCount").and_then(|s| s.parse().ok());
    let song_offset = params.get("songOffset").and_then(|s| s.parse().ok());
    let _music_folder_id: Option<i32> = params.get("musicFolderId").and_then(|s| s.parse().ok());

    match state
        .storage
        .search3(
            query,
            artist_count,
            artist_offset,
            album_count,
            album_offset,
            song_count,
            song_offset,
        )
        .await
    {
        Ok(result) => {
            let artists: Vec<ArtistID3Item> =
                result.artists.iter().map(ArtistID3Item::from).collect();
            let albums: Vec<AlbumID3Item> = result.albums.iter().map(AlbumID3Item::from).collect();
            let songs: Vec<Child> = result.songs.iter().map(Child::from).collect();

            let data = SearchResult3Data {
                search_result3: SearchResult3Inner {
                    artist: artists,
                    album: albums,
                    song: songs,
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::SearchResult3(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== 媒体检索处理器 =====

/// GET /rest/getCoverArt - 获取封面图片
async fn get_cover_art_handler<S: SubsonicStorage + FileStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };
    let _size: Option<i32> = params.get("size").and_then(|s| s.parse().ok());

    match state.storage.get_cover_art_path(id).await {
        Ok(Some(path)) => {
            // 读取封面图片文件
            match state.storage.read_file(&path).await {
                Ok(data) => {
                    // 根据文件扩展名确定 MIME 类型
                    let mime_type = if path.ends_with(".png") {
                        "image/png"
                    } else if path.ends_with(".gif") {
                        "image/gif"
                    } else if path.ends_with(".webp") {
                        "image/webp"
                    } else {
                        "image/jpeg"
                    };

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, mime_type)
                        .header(header::CONTENT_LENGTH, data.len())
                        .header(header::CACHE_CONTROL, "public, max-age=86400")
                        .body(axum::body::Body::from(data))
                        .unwrap()
                }
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(format!("Failed to read cover art: {}", e)))
                    .unwrap(),
            }
        }
        Ok(None) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Cover art not found"))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(e.to_string()))
            .unwrap(),
    }
}

/// GET /rest/stream - 流式传输媒体文件
async fn stream_handler<S: SubsonicStorage + FileStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    // 可选参数
    let _max_bit_rate: Option<i32> = params.get("maxBitRate").and_then(|s| s.parse().ok());
    let _format = params.get("format").map(|s| s.as_str());
    let _time_offset: Option<i32> = params.get("timeOffset").and_then(|s| s.parse().ok());
    let _estimated_content_length: Option<bool> = params.get("estimateContentLength").and_then(|s| s.parse().ok());

    match state.storage.get_stream_path(id).await {
        Ok(Some(path)) => {
            // 读取媒体文件
            match state.storage.read_file(&path).await {
                Ok(data) => {
                    // 根据文件扩展名确定 MIME 类型
                    let mime_type = if path.ends_with(".flac") {
                        "audio/flac"
                    } else if path.ends_with(".ogg") || path.ends_with(".opus") {
                        "audio/ogg"
                    } else if path.ends_with(".m4a") || path.ends_with(".aac") {
                        "audio/mp4"
                    } else if path.ends_with(".wav") {
                        "audio/wav"
                    } else if path.ends_with(".wma") {
                        "audio/x-ms-wma"
                    } else {
                        "audio/mpeg"
                    };

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, mime_type)
                        .header(header::CONTENT_LENGTH, data.len())
                        .header(header::ACCEPT_RANGES, "bytes")
                        .body(axum::body::Body::from(data))
                        .unwrap()
                }
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(format!("Failed to read media file: {}", e)))
                    .unwrap(),
            }
        }
        Ok(None) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("Media file not found"))
            .unwrap(),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(e.to_string()))
            .unwrap(),
    }
}
