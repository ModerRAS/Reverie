//! Core domain models for Reverie

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
