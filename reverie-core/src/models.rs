//! Core domain models for Reverie

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

/// Represents a music track
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub title: String,
    pub album_id: Option<Uuid>,
    pub artist_id: Option<Uuid>,
    pub duration: u32, // in seconds
    pub file_path: String,
    pub file_size: u64,
    pub bitrate: u32,
    pub format: String, // e.g., "mp3", "flac"
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents an album
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub id: Uuid,
    pub name: String,
    pub artist_id: Option<Uuid>,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub cover_art_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents an artist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub id: Uuid,
    pub name: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a user in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a track in a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub playlist_id: Uuid,
    pub track_id: Uuid,
    pub position: u32,
    pub added_at: DateTime<Utc>,
}

/// Library scanning statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStats {
    pub total_tracks: u64,
    pub total_albums: u64,
    pub total_artists: u64,
    pub scanned_files: u64,
    pub new_tracks: u64,
    pub updated_tracks: u64,
    pub deleted_tracks: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub parent: Option<String>,
    pub is_dir: bool,
    pub title: String,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub size: i64,
    pub content_type: String,
    pub suffix: String,
    pub duration: f32,
    pub bit_rate: i32,
    pub sample_rate: i32,
    pub bit_depth: Option<i32>,
    pub channels: Option<i32>,
    pub path: String,
    pub play_count: Option<i64>,
    pub created: Option<DateTime<Utc>>,
    pub starred: Option<DateTime<Utc>>,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub r#type: String,
    pub user_rating: Option<i32>,
    pub library_id: i32,
    pub missing: bool,
}

impl Default for MediaFile {
    fn default() -> Self {
        MediaFile {
            id: String::new(),
            parent: None,
            is_dir: false,
            title: String::new(),
            album: None,
            artist: None,
            album_artist: None,
            track_number: None,
            disc_number: None,
            year: None,
            genre: None,
            cover_art: None,
            size: 0,
            content_type: String::new(),
            suffix: String::new(),
            duration: 0.0,
            bit_rate: 0,
            sample_rate: 0,
            bit_depth: None,
            channels: None,
            path: String::new(),
            play_count: None,
            created: None,
            starred: None,
            album_id: None,
            artist_id: None,
            r#type: String::new(),
            user_rating: None,
            library_id: 0,
            missing: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicAlbum {
    pub id: String,
    pub name: String,
    pub album_artist: Option<String>,
    pub album_artist_id: Option<String>,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub cover_art: Option<String>,
    pub song_count: i32,
    pub duration: f32,
    pub play_count: Option<i64>,
    pub created: Option<DateTime<Utc>>,
    pub starred: Option<DateTime<Utc>>,
    pub user_rating: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicArtist {
    pub id: String,
    pub name: String,
    pub cover_art: Option<String>,
    pub album_count: i32,
    pub starred: Option<DateTime<Utc>>,
    pub user_rating: Option<i32>,
}

pub type SubsonicArtistIndexes = Vec<SubsonicArtistIndex>;

#[derive(Debug, Clone)]
pub struct SubsonicArtistIndex {
    pub id: String,
    pub artists: Vec<SubsonicArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicGenre {
    pub name: String,
    pub song_count: i32,
    pub album_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicMusicFolder {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicScanStatus {
    pub scanning: bool,
    pub count: i64,
    pub folder_count: i64,
    pub last_scan: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub scan_type: Option<String>,
    pub elapsed_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicLyrics {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicPlayQueue {
    pub entries: Vec<MediaFile>,
    pub current: Option<String>,
    pub position: i64,
    pub username: String,
    pub changed: DateTime<Utc>,
    pub changed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsonicDirectory {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub artist: Option<String>,
    pub artist_id: Option<String>,
    pub cover_art: Option<String>,
    pub child_count: Option<i32>,
    pub album_count: Option<i32>,
    pub duration: Option<i32>,
    pub play_count: Option<i64>,
    pub starred: Option<DateTime<Utc>>,
    pub user_rating: Option<i32>,
    pub children: Vec<MediaFile>,
}

impl SubsonicDirectory {
    pub fn from_album(album: &SubsonicAlbum, children: Vec<MediaFile>) -> Self {
        SubsonicDirectory {
            id: album.id.clone(),
            parent: album.artist_id.clone(),
            name: album.name.clone(),
            artist: album.artist.clone(),
            artist_id: album.artist_id.clone(),
            cover_art: album.cover_art.clone(),
            child_count: Some(children.len() as i32),
            album_count: None,
            duration: Some(album.duration as i32),
            play_count: album.play_count,
            starred: album.starred,
            user_rating: album.user_rating,
            children,
        }
    }
}
