//! 用户和系统相关端点处理器

use axum::{
    extract::{Query, State},
    response::Response,
};
use reverie_storage::{FileStorage, SubsonicStorage};
use std::collections::HashMap;

use super::{error_response, format_response, ok_response, SubsonicState};
use super::response::*;

/// GET /rest/getUser - 获取用户信息
pub async fn get_user_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let username = match params.get("username") {
        Some(u) => u.as_str(),
        None => return error_response(&params, 10, "Missing required parameter: username"),
    };

    match state.storage.get_user(username).await {
        Ok(Some(user)) => {
            let data = UserData {
                user: UserItem::from(&user),
            };
            let response = SubsonicResponse::ok_with(ResponseData::User(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "User not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getUsers - 获取所有用户
pub async fn get_users_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_users().await {
        Ok(users) => {
            let items: Vec<UserItem> = users.iter().map(UserItem::from).collect();
            let data = UsersData {
                users: UsersInner { user: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Users(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getScanStatus - 获取扫描状态
pub async fn get_scan_status_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_scan_status().await {
        Ok(status) => {
            let data = ScanStatusData {
                scan_status: ScanStatusItem::from(&status),
            };
            let response = SubsonicResponse::ok_with(ResponseData::ScanStatus(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/startScan - 开始媒体库扫描
pub async fn start_scan_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.start_scan().await {
        Ok(status) => {
            let data = ScanStatusData {
                scan_status: ScanStatusItem::from(&status),
            };
            let response = SubsonicResponse::ok_with(ResponseData::ScanStatus(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/search2 - 搜索（基于文件夹）
pub async fn search2_handler<S: SubsonicStorage + Clone>(
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

    match state
        .storage
        .search2(
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
            // search2 使用 ArtistItem（非 ID3 版本）
            let artists: Vec<ArtistItem> = result.artists.iter().map(|a| ArtistItem {
                id: a.id.clone(),
                name: a.name.clone(),
                cover_art: a.cover_art.clone(),
                artist_image_url: None,
                starred: a.starred.map(|d| d.to_rfc3339()),
                user_rating: a.user_rating,
            }).collect();
            
            // albums 转换为 Child
            let albums: Vec<Child> = result.albums.iter().map(|a| Child {
                id: a.id.clone(),
                parent: a.artist_id.clone(),
                is_dir: true,
                title: a.name.clone(),
                album: Some(a.name.clone()),
                artist: a.artist.clone(),
                track: None,
                year: a.year,
                genre: a.genre.clone(),
                cover_art: a.cover_art.clone(),
                size: None,
                content_type: None,
                suffix: None,
                duration: Some(a.duration as i32),
                bit_rate: None,
                path: None,
                play_count: a.play_count,
                disc_number: None,
                created: a.created.map(|d| d.to_rfc3339()),
                album_id: Some(a.id.clone()),
                artist_id: a.artist_id.clone(),
                starred: a.starred.map(|d| d.to_rfc3339()),
                user_rating: a.user_rating,
                media_type: Some("album".to_string()),
                is_video: false,
            }).collect();
            
            let songs: Vec<Child> = result.songs.iter().map(Child::from).collect();

            let data = SearchResult2Data {
                search_result2: SearchResult2Inner {
                    artist: artists,
                    album: albums,
                    song: songs,
                },
            };
            let response = SubsonicResponse::ok_with(ResponseData::SearchResult2(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/star - 收藏
pub async fn star_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "id")
        .map(|(_, v)| v.as_str())
        .collect();
    let album_ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "albumId")
        .map(|(_, v)| v.as_str())
        .collect();
    let artist_ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "artistId")
        .map(|(_, v)| v.as_str())
        .collect();

    match state.storage.star(&ids, &album_ids, &artist_ids).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/unstar - 取消收藏
pub async fn unstar_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "id")
        .map(|(_, v)| v.as_str())
        .collect();
    let album_ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "albumId")
        .map(|(_, v)| v.as_str())
        .collect();
    let artist_ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| *k == "artistId")
        .map(|(_, v)| v.as_str())
        .collect();

    match state.storage.unstar(&ids, &album_ids, &artist_ids).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/setRating - 设置评分
pub async fn set_rating_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };
    let rating = match params.get("rating").and_then(|s| s.parse().ok()) {
        Some(r) => r,
        None => return error_response(&params, 10, "Missing required parameter: rating"),
    };

    match state.storage.set_rating(id, rating).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/scrobble - Scrobble
pub async fn scrobble_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };
    let time = params.get("time").and_then(|s| s.parse().ok());
    let submission = params.get("submission").and_then(|s| s.parse().ok()).unwrap_or(true);

    match state.storage.scrobble(id, time, submission).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/download - 下载媒体文件
pub async fn download_handler<S: SubsonicStorage + FileStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    use axum::http::{header, StatusCode};

    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_stream_path(id).await {
        Ok(Some(path)) => {
            match state.storage.read_file(&path).await {
                Ok(data) => {
                    // 获取文件名
                    let filename = std::path::Path::new(&path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("download");

                    Response::builder()
                        .status(StatusCode::OK)
                        .header(header::CONTENT_TYPE, "application/octet-stream")
                        .header(
                            header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", filename),
                        )
                        .header(header::CONTENT_LENGTH, data.len())
                        .body(axum::body::Body::from(data))
                        .unwrap()
                }
                Err(e) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(axum::body::Body::from(format!("Failed to read file: {}", e)))
                    .unwrap(),
            }
        }
        Ok(None) => Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(axum::body::Body::from("File not found"))
            .unwrap(),
        Err(e) => Response::builder()
            .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
            .body(axum::body::Body::from(e.to_string()))
            .unwrap(),
    }
}
