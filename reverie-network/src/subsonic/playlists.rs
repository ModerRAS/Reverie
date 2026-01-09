//! 播放列表相关端点处理器

use axum::{
    extract::{Query, State},
    response::Response,
};
use reverie_storage::SubsonicStorage;
use std::collections::HashMap;

use super::{error_response, format_response, ok_response, SubsonicState};
use super::response::*;

/// GET /rest/getPlaylists - 获取播放列表
pub async fn get_playlists_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let username = params.get("username").map(|s| s.as_str());

    match state.storage.get_playlists(username).await {
        Ok(playlists) => {
            let items: Vec<PlaylistItem> = playlists.iter().map(PlaylistItem::from).collect();
            let data = PlaylistsData {
                playlists: PlaylistsInner { playlist: items },
            };
            let response = SubsonicResponse::ok_with(ResponseData::Playlists(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/getPlaylist - 获取播放列表详情
pub async fn get_playlist_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_playlist(id).await {
        Ok(Some(playlist)) => {
            let data = PlaylistData {
                playlist: PlaylistWithEntries::from(&playlist),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Playlist(data));
            format_response(&params, response)
        }
        Ok(None) => error_response(&params, 70, "Playlist not found"),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/createPlaylist - 创建或更新播放列表
pub async fn create_playlist_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let playlist_id = params.get("playlistId").map(|s| s.as_str());
    let name = params.get("name").map(|s| s.as_str());
    
    // 收集所有 songId 参数
    let song_ids: Vec<&str> = params
        .iter()
        .filter(|(k, _)| k.starts_with("songId"))
        .map(|(_, v)| v.as_str())
        .collect();

    if playlist_id.is_none() && name.is_none() {
        return error_response(&params, 10, "Either playlistId or name must be provided");
    }

    match state.storage.create_playlist(name, playlist_id, &song_ids).await {
        Ok(playlist) => {
            let data = PlaylistData {
                playlist: PlaylistWithEntries::from(&playlist),
            };
            let response = SubsonicResponse::ok_with(ResponseData::Playlist(data));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/updatePlaylist - 更新播放列表
pub async fn update_playlist_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let playlist_id = match params.get("playlistId") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: playlistId"),
    };

    let name = params.get("name").map(|s| s.as_str());
    let comment = params.get("comment").map(|s| s.as_str());
    let public = params.get("public").and_then(|s| s.parse().ok());

    // 收集要添加的歌曲
    let song_ids_to_add: Vec<&str> = params
        .iter()
        .filter(|(k, _)| k.starts_with("songIdToAdd"))
        .map(|(_, v)| v.as_str())
        .collect();

    // 收集要删除的索引
    let indexes_to_remove: Vec<i32> = params
        .iter()
        .filter(|(k, _)| k.starts_with("songIndexToRemove"))
        .filter_map(|(_, v)| v.parse().ok())
        .collect();

    match state
        .storage
        .update_playlist(
            playlist_id,
            name,
            comment,
            public,
            &song_ids_to_add,
            &indexes_to_remove,
        )
        .await
    {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

/// GET /rest/deletePlaylist - 删除播放列表
pub async fn delete_playlist_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.delete_playlist(id).await {
        Ok(()) => ok_response(&params),
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}
