//! API client for communicating with Reverie server
//!
//! Implements the Subsonic API client for fetching music data.

#![allow(unused)]

use serde::{Deserialize, Serialize};

/// Base URL for the Subsonic API
const API_BASE: &str = "/rest";

/// API version
const API_VERSION: &str = "1.16.1";

/// Client name
const CLIENT_NAME: &str = "reverie-ui";

/// Subsonic API response wrapper
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubsonicResponse<T> {
    pub subsonic_response: SubsonicResponseInner<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsonicResponseInner<T> {
    pub status: String,
    pub version: String,
    #[serde(flatten)]
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

/// Music folder
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MusicFolder {
    pub id: i32,
    pub name: String,
}

/// Artist index
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArtistIndex {
    pub name: String,
    pub artist: Vec<Artist>,
}

/// Artist
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub album_count: i32,
    pub cover_art: Option<String>,
    pub artist_image_url: Option<String>,
    #[serde(default)]
    pub starred: Option<String>,
}

/// Album
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    pub song_count: Option<i32>,
    pub duration: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub created: Option<String>,
    #[serde(default)]
    pub starred: Option<String>,
    #[serde(default)]
    pub play_count: i32,
}

/// Song/Track
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub id: String,
    pub title: String,
    pub album: Option<String>,
    pub album_id: Option<String>,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub track: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub duration: Option<i32>,
    pub bit_rate: Option<i32>,
    pub suffix: Option<String>,
    pub content_type: Option<String>,
    pub path: Option<String>,
    #[serde(default)]
    pub starred: Option<String>,
    #[serde(default)]
    pub play_count: i32,
}

/// Playlist
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub song_count: i32,
    pub duration: i32,
    pub owner: Option<String>,
    pub public: Option<bool>,
    pub created: Option<String>,
    pub changed: Option<String>,
    pub cover_art: Option<String>,
    #[serde(default)]
    pub entry: Vec<Song>,
}

/// Search result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult3 {
    #[serde(default)]
    pub artist: Vec<Artist>,
    #[serde(default)]
    pub album: Vec<Album>,
    #[serde(default)]
    pub song: Vec<Song>,
}

/// Genre
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Genre {
    pub value: String,
    pub song_count: i32,
    pub album_count: i32,
}

/// Now playing entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingEntry {
    pub username: String,
    pub minutes_ago: i32,
    pub player_id: Option<i32>,
    pub player_name: Option<String>,
    #[serde(flatten)]
    pub song: Song,
}

/// API Client for Subsonic-compatible server
pub struct ApiClient {
    base_url: String,
    username: String,
    password: String,
}

impl ApiClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// Build API URL with authentication parameters
    fn build_url(&self, endpoint: &str) -> String {
        format!(
            "{}{}.view?u={}&p={}&v={}&c={}&f=json",
            self.base_url, endpoint, self.username, self.password, API_VERSION, CLIENT_NAME
        )
    }

    /// Get cover art URL
    pub fn cover_art_url(&self, id: &str, size: Option<i32>) -> String {
        let size_param = size.map(|s| format!("&size={}", s)).unwrap_or_default();
        format!(
            "{}/getCoverArt.view?u={}&p={}&v={}&c={}&id={}{}",
            self.base_url, self.username, self.password, API_VERSION, CLIENT_NAME, id, size_param
        )
    }

    /// Get stream URL for a song
    pub fn stream_url(&self, id: &str) -> String {
        format!(
            "{}/stream.view?u={}&p={}&v={}&c={}&id={}",
            self.base_url, self.username, self.password, API_VERSION, CLIENT_NAME, id
        )
    }
}

// Response data structures for API calls
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFoldersData {
    pub music_folders: MusicFoldersList,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFoldersList {
    pub music_folder: Vec<MusicFolder>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexesData {
    pub indexes: IndexesList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexesList {
    #[serde(default)]
    pub index: Vec<ArtistIndex>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtistsData {
    pub artists: ArtistsList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtistsList {
    #[serde(default)]
    pub index: Vec<ArtistIndex>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumListData {
    pub album_list2: AlbumList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumList {
    #[serde(default)]
    pub album: Vec<Album>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlbumData {
    pub album: AlbumWithSongs,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumWithSongs {
    #[serde(flatten)]
    pub album: Album,
    #[serde(default)]
    pub song: Vec<Song>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArtistData {
    pub artist: ArtistWithAlbums,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistWithAlbums {
    #[serde(flatten)]
    pub artist: Artist,
    #[serde(default)]
    pub album: Vec<Album>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistsData {
    pub playlists: PlaylistsList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistsList {
    #[serde(default)]
    pub playlist: Vec<Playlist>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistData {
    pub playlist: Playlist,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult3Data {
    pub search_result3: SearchResult3,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenresData {
    pub genres: GenresList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenresList {
    #[serde(default)]
    pub genre: Vec<Genre>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarredData {
    pub starred2: StarredItems,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StarredItems {
    #[serde(default)]
    pub artist: Vec<Artist>,
    #[serde(default)]
    pub album: Vec<Album>,
    #[serde(default)]
    pub song: Vec<Song>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingData {
    pub now_playing: NowPlayingList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NowPlayingList {
    #[serde(default)]
    pub entry: Vec<NowPlayingEntry>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomSongsData {
    pub random_songs: RandomSongsList,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RandomSongsList {
    #[serde(default)]
    pub song: Vec<Song>,
}
