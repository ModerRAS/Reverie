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
use chrono::Utc;
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
        .route("/getArtistInfo", get(get_artist_info_handler::<S>))
        .route("/getArtistInfo2", get(get_artist_info2_handler::<S>))
        .route("/getAlbumInfo", get(get_album_info_handler::<S>))
        .route("/getAlbumInfo2", get(get_album_info2_handler::<S>))
        .route("/getSimilarSongs", get(get_similar_songs_handler::<S>))
        .route("/getSimilarSongs2", get(get_similar_songs2_handler::<S>))
        .route("/getTopSongs", get(get_top_songs_handler::<S>))
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
        .route("/getLyrics", get(get_lyrics_handler::<S>))
        .route("/getLyricsBySongId", get(get_lyrics_by_song_id_handler::<S>))
        .route("/getAvatar", get(get_avatar_handler::<S>))
        // Annotation endpoints
        .route("/star", get(star_handler::<S>))
        .route("/unstar", get(unstar_handler::<S>))
        .route("/setRating", get(set_rating_handler::<S>))
        .route("/scrobble", get(scrobble_handler::<S>))
        // Bookmark endpoints
        .route("/getBookmarks", get(get_bookmarks_handler::<S>))
        .route("/createBookmark", get(create_bookmark_handler::<S>))
        .route("/deleteBookmark", get(delete_bookmark_handler::<S>))
        .route("/getPlayQueue", get(get_play_queue_handler::<S>))
        .route("/savePlayQueue", get(save_play_queue_handler::<S>))
        // Share endpoints
        .route("/getShares", get(get_shares_handler::<S>))
        .route("/createShare", get(create_share_handler::<S>))
        .route("/updateShare", get(update_share_handler::<S>))
        .route("/deleteShare", get(delete_share_handler::<S>))
        // Internet radio endpoints
        .route("/getInternetRadioStations", get(get_internet_radio_stations_handler::<S>))
        .route("/createInternetRadioStation", get(create_internet_radio_station_handler::<S>))
        .route("/updateInternetRadioStation", get(update_internet_radio_station_handler::<S>))
        .route("/deleteInternetRadioStation", get(delete_internet_radio_station_handler::<S>))
        // User management endpoints
        .route("/getUser", get(get_user_handler::<S>))
        .route("/getUsers", get(get_users_handler::<S>))
        .route("/createUser", get(create_user_handler::<S>))
        .route("/updateUser", get(update_user_handler::<S>))
        // Scanning endpoints
        .route("/getScanStatus", get(get_scan_status_handler::<S>))
        .route("/startScan", get(start_scan_handler::<S>))
        // OpenSubsonic extensions
        .route("/getOpenSubsonicExtensions", get(get_open_subsonic_extensions_handler::<S>))
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

// ===== 艺术家/专辑信息处理器 =====

/// GET /rest/getArtistInfo - 获取艺术家信息（简介、图片、相似艺术家）
async fn get_artist_info_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());
    let include_not_present = params.get("includeNotPresent").and_then(|s| s.parse().ok());

    match state.storage.get_artist_info(id, count, include_not_present).await {
        Ok(info) => {
            let data = ArtistInfoData {
                artist_info: ArtistInfo::from(&info),
            };
            let response = SubsonicResponse::ok_with(ResponseData::ArtistInfo(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getArtistInfo2 - 获取艺术家信息（ID3 版本）
async fn get_artist_info2_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());
    let include_not_present = params.get("includeNotPresent").and_then(|s| s.parse().ok());

    match state.storage.get_artist_info2(id, count, include_not_present).await {
        Ok(info) => {
            let data = ArtistInfo2Data {
                artist_info2: ArtistInfo2::from(&info),
            };
            let response = SubsonicResponse::ok_with(ResponseData::ArtistInfo2(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getAlbumInfo - 获取专辑信息（备注、图片）
async fn get_album_info_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_album_info(id).await {
        Ok(info) => {
            let data = AlbumInfoData {
                album_info: AlbumInfo::from(&info),
            };
            let response = SubsonicResponse::ok_with(ResponseData::AlbumInfo(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getAlbumInfo2 - 获取专辑信息（ID3 版本）
async fn get_album_info2_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_album_info2(id).await {
        Ok(info) => {
            // AlbumInfo2 使用相同的响应格式
            let data = AlbumInfoData {
                album_info: AlbumInfo::from(&info),
            };
            let response = SubsonicResponse::ok_with(ResponseData::AlbumInfo(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getSimilarSongs - 获取相似歌曲
async fn get_similar_songs_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());

    match state.storage.get_similar_songs(id, count).await {
        Ok(songs) => {
            let items: Vec<Child> = songs.iter().map(Child::from).collect();
            let data = SimilarSongsData {
                similar_songs: SimilarSongsInner { song: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::SimilarSongs(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getSimilarSongs2 - 获取相似歌曲（ID3 版本）
async fn get_similar_songs2_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());

    match state.storage.get_similar_songs2(id, count).await {
        Ok(songs) => {
            let items: Vec<Child> = songs.iter().map(Child::from).collect();
            let data = SimilarSongs2Data {
                similar_songs2: SimilarSongs2Inner { song: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::SimilarSongs2(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getTopSongs - 获取艺术家的热门歌曲
async fn get_top_songs_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let artist = match params.get("artist") {
        Some(a) => a,
        None => return error_response(&params, 10, "Missing required parameter: artist"),
    };

    let count = params.get("count").and_then(|s| s.parse().ok());

    match state.storage.get_top_songs(artist, count).await {
        Ok(top_songs) => {
            let items: Vec<Child> = top_songs.songs.iter().map(Child::from).collect();
            let data = TopSongsData {
                top_songs: TopSongsInner { song: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::TopSongs(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getLyrics - 获取歌词
async fn get_lyrics_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let artist = params.get("artist").map(|s| s.as_str());
    let title = params.get("title").map(|s| s.as_str());

    match state.storage.get_lyrics(artist, title).await {
        Ok(Some(lyrics)) => {
            let data = LyricsData {
                lyrics: LyricsItem::from(&lyrics),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Lyrics(data));
            format_response(&params, response)
        }
        Ok(None) => {
            // 没有找到歌词，返回空的歌词对象
            let data = LyricsData {
                lyrics: LyricsItem {
                    artist: artist.map(|s| s.to_string()),
                    title: title.map(|s| s.to_string()),
                    value: String::new(),
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Lyrics(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== Bookmark 处理器 =====

/// GET /rest/getBookmarks - 获取用户的所有书签
async fn get_bookmarks_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_bookmarks().await {
        Ok(bookmarks) => {
            let items: Vec<BookmarkItem> = bookmarks.iter().map(BookmarkItem::from).collect();
            let data = BookmarksData {
                bookmarks: BookmarksList { bookmark: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Bookmarks(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/createBookmark - 创建/更新书签
async fn create_bookmark_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let position = match params.get("position").and_then(|s| s.parse().ok()) {
        Some(p) => p,
        None => return error_response(&params, 10, "Missing required parameter: position"),
    };

    let comment = params.get("comment").map(|s| s.as_str());

    match state.storage.create_bookmark(id, position, comment).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/deleteBookmark - 删除书签
async fn delete_bookmark_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.delete_bookmark(id).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getPlayQueue - 获取播放队列
async fn get_play_queue_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_play_queue().await {
        Ok(Some(queue)) => {
            let entries: Vec<Child> = queue.entries.iter().map(Child::from).collect();
            let data = PlayQueueData {
                play_queue: PlayQueueInner {
                    entry: entries,
                    current: queue.current,
                    position: queue.position,
                    username: queue.username,
                    changed: queue.changed.to_rfc3339(),
                    changed_by: queue.changed_by,
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::PlayQueue(data));
            format_response(&params, response)
        }
        Ok(None) => {
            // 没有播放队列，返回空队列
            let data = PlayQueueData {
                play_queue: PlayQueueInner {
                    entry: vec![],
                    current: None,
                    position: 0,
                    username: String::new(),
                    changed: Utc::now().to_rfc3339(),
                    changed_by: String::new(),
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::PlayQueue(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/savePlayQueue - 保存播放队列
async fn save_play_queue_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // 获取多个 id 参数
    let ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| k.as_str() == "id")
        .map(|(_, v)| v.as_str())
        .collect();

    let current = params.get("current").map(|s| s.as_str());
    let position = params.get("position").and_then(|s| s.parse().ok());

    match state.storage.save_play_queue(&ids, current, position).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== Share 处理器 =====

/// GET /rest/getShares - 获取所有分享
async fn get_shares_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_shares().await {
        Ok(shares) => {
            let items: Vec<ShareItem> = shares.iter().map(ShareItem::from).collect();
            let data = SharesData {
                shares: SharesList { share: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Shares(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/createShare - 创建分享
async fn create_share_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // 获取多个 id 参数
    let ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| k.as_str() == "id")
        .map(|(_, v)| v.as_str())
        .collect();

    if ids.is_empty() {
        return error_response(&params, 10, "Missing required parameter: id");
    }

    let description = params.get("description").map(|s| s.as_str());
    let expires = params.get("expires").and_then(|s| s.parse().ok());

    match state.storage.create_share(&ids, description, expires).await {
        Ok(share) => {
            let items = vec![ShareItem::from(&share)];
            let data = SharesData {
                shares: SharesList { share: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Shares(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/updateShare - 更新分享
async fn update_share_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let description = params.get("description").map(|s| s.as_str());
    let expires = params.get("expires").and_then(|s| s.parse().ok());

    match state.storage.update_share(id, description, expires).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/deleteShare - 删除分享
async fn delete_share_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.delete_share(id).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== Internet Radio 处理器 =====

/// GET /rest/getInternetRadioStations - 获取所有网络电台
async fn get_internet_radio_stations_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_internet_radio_stations().await {
        Ok(stations) => {
            let items: Vec<InternetRadioStationItem> = stations.iter().map(InternetRadioStationItem::from).collect();
            let data = InternetRadioStationsData {
                internet_radio_stations: InternetRadioStationsList { internet_radio_station: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::InternetRadioStations(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/createInternetRadioStation - 创建网络电台
async fn create_internet_radio_station_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let stream_url = match params.get("streamUrl") {
        Some(url) => url,
        None => return error_response(&params, 10, "Missing required parameter: streamUrl"),
    };

    let name = match params.get("name") {
        Some(n) => n,
        None => return error_response(&params, 10, "Missing required parameter: name"),
    };

    let homepage_url = params.get("homepageUrl").map(|s| s.as_str());

    match state.storage.create_internet_radio_station(stream_url, name, homepage_url).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/updateInternetRadioStation - 更新网络电台
async fn update_internet_radio_station_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    let stream_url = match params.get("streamUrl") {
        Some(url) => url,
        None => return error_response(&params, 10, "Missing required parameter: streamUrl"),
    };

    let name = match params.get("name") {
        Some(n) => n,
        None => return error_response(&params, 10, "Missing required parameter: name"),
    };

    let homepage_url = params.get("homepageUrl").map(|s| s.as_str());

    match state.storage.update_internet_radio_station(id, stream_url, name, homepage_url).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/deleteInternetRadioStation - 删除网络电台
async fn delete_internet_radio_station_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.delete_internet_radio_station(id).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getOpenSubsonicExtensions - 获取 OpenSubsonic 扩展列表
async fn get_open_subsonic_extensions_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // 目前返回空的扩展列表
    let data = OpenSubsonicExtensionsData {
        open_subsonic_extensions: OpenSubsonicExtensionsList {
            extension: vec![],
        },
    };
    let response = SubsonicResponse::ok_with(ResponseData::OpenSubsonicExtensions(data));
    format_response(&params, response)
}

/// GET /rest/getLyricsBySongId - 通过歌曲 ID 获取歌词（OpenSubsonic 扩展）
async fn get_lyrics_by_song_id_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_lyrics_by_song_id(id).await {
        Ok(lyrics_list) => {
            let items: Vec<StructuredLyricsItem> = lyrics_list.iter().map(StructuredLyricsItem::from).collect();
            let data = LyricsListData {
                lyrics_list: LyricsListInner { lyrics: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::LyricsList(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getAvatar - 获取用户头像
async fn get_avatar_handler<S: SubsonicStorage + FileStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let username = match params.get("username") {
        Some(u) => u,
        None => return error_response(&params, 10, "Missing required parameter: username"),
    };

    match state.storage.get_avatar_path(username).await {
        Ok(Some(path)) => {
            // 读取头像文件并返回
            match state.storage.read_file(&path).await {
                Ok(data) => {
                    // 根据文件扩展名确定 MIME 类型
                    let content_type = if path.ends_with(".png") {
                        "image/png"
                    } else if path.ends_with(".gif") {
                        "image/gif"
                    } else {
                        "image/jpeg"
                    };
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, content_type)],
                        data,
                    )
                        .into_response()
                }
                Err(_) => {
                    // 文件读取失败，返回默认头像或 404
                    (StatusCode::NOT_FOUND, "Avatar not found").into_response()
                }
            }
        }
        Ok(None) => (StatusCode::NOT_FOUND, "Avatar not found").into_response(),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
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
