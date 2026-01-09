//! Subsonic API implementation
//!
//! Reverie aims to be compatible with Subsonic API 1.16.1.
//! This module provides handlers for all Subsonic API endpoints.

mod auth;
mod response;

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reverie_core::SUBSONIC_API_VERSION;
use reverie_storage::SubsonicStorage;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

use response::*;

// === State and Response Helpers ===

#[derive(Clone)]
pub(crate) struct SubsonicState<S: Clone> {
    pub(crate) storage: Arc<S>,
}

impl<S: Clone> SubsonicState<S> {
    pub(crate) fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
}

/// Return JSON or XML based on format parameter
fn format_response(params: &HashMap<String, String>, response: SubsonicResponse) -> Response {
    let format = params.get("f").map(|s| s.as_str()).unwrap_or("xml");
    
    if format == "json" {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&response).unwrap_or_default(),
        ).into_response()
    } else {
        // For now, return JSON even for XML requests (proper XML support TODO)
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&response).unwrap_or_default(),
        ).into_response()
    }
}

fn ok_response(params: &HashMap<String, String>) -> Response {
    format_response(params, SubsonicResponse::ok())
}

fn error_response(params: &HashMap<String, String>, code: i32, message: &str) -> Response {
    format_response(params, SubsonicResponse::error(code, message))
}

/// Create the Subsonic router.
///
/// Note: The returned router is *missing* `SubsonicState<S>` and is intended to be
/// nested into an outer router that provides state via `Router::with_state`.
#[cfg(feature = "axum-server")]
pub(crate) fn create_router<S: SubsonicStorage + Clone + 'static>() -> Router<SubsonicState<S>> {
    Router::new()
        // System endpoints
        .route("/ping", get(ping_handler::<S>))
        .route("/getLicense", get(get_license_handler::<S>))
        .route("/getMusicFolders", get(get_music_folders_handler::<S>))
        // Browsing endpoints
        .route("/getIndexes", get(stub_handler::<S>))
        .route("/getMusicDirectory", get(stub_handler::<S>))
        .route("/getGenres", get(stub_handler::<S>))
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
        .route("/getAlbumList", get(stub_handler::<S>))
        .route("/getAlbumList2", get(get_album_list2_handler::<S>))
        .route("/getRandomSongs", get(stub_handler::<S>))
        .route("/getSongsByGenre", get(stub_handler::<S>))
        .route("/getNowPlaying", get(stub_handler::<S>))
        .route("/getStarred", get(stub_handler::<S>))
        .route("/getStarred2", get(stub_handler::<S>))
        // Search endpoints
        .route("/search2", get(stub_handler::<S>))
        .route("/search3", get(search3_handler::<S>))
        // Playlist endpoints
        .route("/getPlaylists", get(stub_handler::<S>))
        .route("/getPlaylist", get(stub_handler::<S>))
        .route("/createPlaylist", get(stub_handler::<S>))
        .route("/updatePlaylist", get(stub_handler::<S>))
        .route("/deletePlaylist", get(stub_handler::<S>))
        // Media retrieval endpoints
        .route("/stream", get(stream_handler::<S>))
        .route("/download", get(stub_handler::<S>))
        .route("/getCoverArt", get(get_cover_art_handler::<S>))
        .route("/getLyrics", get(stub_handler::<S>))
        .route("/getLyricsBySongId", get(stub_handler::<S>))
        .route("/getAvatar", get(stub_handler::<S>))
        .route("/star", get(stub_handler::<S>))
        .route("/unstar", get(stub_handler::<S>))
        .route("/setRating", get(stub_handler::<S>))
        .route("/scrobble", get(stub_handler::<S>))
        .route("/getBookmarks", get(stub_handler::<S>))
        .route("/createBookmark", get(stub_handler::<S>))
        .route("/deleteBookmark", get(stub_handler::<S>))
        .route("/getPlayQueue", get(stub_handler::<S>))
        .route("/savePlayQueue", get(stub_handler::<S>))
        .route("/getShares", get(stub_handler::<S>))
        .route("/createShare", get(stub_handler::<S>))
        .route("/updateShare", get(stub_handler::<S>))
        .route("/deleteShare", get(stub_handler::<S>))
        .route("/getInternetRadioStations", get(stub_handler::<S>))
        .route("/createInternetRadioStation", get(stub_handler::<S>))
        .route("/updateInternetRadioStation", get(stub_handler::<S>))
        .route("/deleteInternetRadioStation", get(stub_handler::<S>))
        .route("/getUser", get(stub_handler::<S>))
        .route("/getUsers", get(stub_handler::<S>))
        .route("/getScanStatus", get(stub_handler::<S>))
        .route("/startScan", get(stub_handler::<S>))
        .route("/getOpenSubsonicExtensions", get(stub_handler::<S>))
}

// ===== System Handlers =====

/// GET /rest/ping - Test connectivity
async fn ping_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    ok_response(&params)
}

/// GET /rest/getLicense - Get server license info
async fn get_license_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let response = SubsonicResponse::ok_with(ResponseData::License(LicenseData {
        license: License { valid: true },
    }));
    format_response(&params, response)
}

/// GET /rest/getMusicFolders - Get configured music folders  
async fn get_music_folders_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match state.storage.get_music_folders().await {
        Ok(folders) => {
            let items: Vec<MusicFolderItem> = folders.iter().map(MusicFolderItem::from).collect();
            let response = SubsonicResponse::ok_with(ResponseData::MusicFolders(MusicFoldersData {
                music_folders: MusicFoldersList {
                    music_folder: items,
                },
            }));
            format_response(&params, response)
        }
        Err(e) => error_response(&params, 0, &e.to_string()),
    }
}

// ===== Stub Handler for unimplemented endpoints =====

/// Stub handler for unimplemented endpoints - returns empty OK response
async fn stub_handler<S: SubsonicStorage + Clone>(
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    ok_response(&params)
}

// ===== Browsing Handlers =====

/// GET /rest/getArtists - Get all artists
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

/// GET /rest/getArtist - Get artist details
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

/// GET /rest/getAlbum - Get album details
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

/// GET /rest/getSong - Get song details
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

// ===== Album List Handlers =====

/// GET /rest/getAlbumList2 - Get album list by type (ID3 tag based)
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

    match state.storage.get_album_list2(
        list_type, size, offset, from_year, to_year, genre, music_folder_id
    ).await {
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

// ===== Search Handlers =====

/// GET /rest/search3 - Search using ID3 tags
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

    match state.storage.search3(
        query, artist_count, artist_offset, album_count, album_offset, song_count, song_offset
    ).await {
        Ok(result) => {
            let artists: Vec<ArtistID3Item> = result.artists.iter().map(ArtistID3Item::from).collect();
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

// ===== Media Retrieval Handlers =====

/// GET /rest/getCoverArt - Get cover art image
async fn get_cover_art_handler<S: SubsonicStorage + Clone>(
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
            // For now, return a placeholder response
            // In production, this would stream the actual image file
            // The storage layer would use VFS to read the file
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/jpeg")
                .body(axum::body::Body::empty())
                .unwrap()
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

/// GET /rest/stream - Stream media file
async fn stream_handler<S: SubsonicStorage + Clone>(
    State(state): State<SubsonicState<S>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let id = match params.get("id") {
        Some(id) => id,
        None => return error_response(&params, 10, "Missing required parameter: id"),
    };

    match state.storage.get_stream_path(id).await {
        Ok(Some(_path)) => {
            // For now, return a placeholder response
            // In production, this would stream the actual media file
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "audio/mpeg")
                .body(axum::body::Body::empty())
                .unwrap()
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
