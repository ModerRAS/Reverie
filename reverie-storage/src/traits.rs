//! Storage abstraction traits
//!
//! These traits define the interface for storage operations,
//! allowing different implementations to be swapped without
//! changing the core application logic.

use async_trait::async_trait;
use reverie_core::{Track, Album, Artist, User, Playlist, PlaylistTrack};
use uuid::Uuid;
use crate::error::Result;

/// Trait for managing music tracks storage
#[async_trait]
pub trait TrackStorage: Send + Sync {
    /// Get a track by ID
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>>;

    /// Get all tracks
    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>>;

    /// Save a track
    async fn save_track(&self, track: &Track) -> Result<()>;

    /// Delete a track
    async fn delete_track(&self, id: Uuid) -> Result<()>;

    /// Search tracks by title
    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>>;

    /// Get tracks by album
    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>>;

    /// Get tracks by artist
    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>>;
}

/// Trait for managing album storage
#[async_trait]
pub trait AlbumStorage: Send + Sync {
    /// Get an album by ID
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>>;

    /// Get all albums
    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>>;

    /// Save an album
    async fn save_album(&self, album: &Album) -> Result<()>;

    /// Delete an album
    async fn delete_album(&self, id: Uuid) -> Result<()>;

    /// Get albums by artist
    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>>;
}

/// Trait for managing artist storage
#[async_trait]
pub trait ArtistStorage: Send + Sync {
    /// Get an artist by ID
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>>;

    /// Get all artists
    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>>;

    /// Save an artist
    async fn save_artist(&self, artist: &Artist) -> Result<()>;

    /// Delete an artist
    async fn delete_artist(&self, id: Uuid) -> Result<()>;
}

/// Trait for managing user storage
#[async_trait]
pub trait UserStorage: Send + Sync {
    /// Get a user by ID
    async fn get_user(&self, id: Uuid) -> Result<Option<User>>;

    /// Get a user by username
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Get all users
    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>>;

    /// Save a user
    async fn save_user(&self, user: &User) -> Result<()>;

    /// Delete a user
    async fn delete_user(&self, id: Uuid) -> Result<()>;
}

/// Trait for managing playlist storage
#[async_trait]
pub trait PlaylistStorage: Send + Sync {
    /// Get a playlist by ID
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>>;

    /// Get playlists by user
    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>>;

    /// Save a playlist
    async fn save_playlist(&self, playlist: &Playlist) -> Result<()>;

    /// Delete a playlist
    async fn delete_playlist(&self, id: Uuid) -> Result<()>;

    /// Add track to playlist
    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()>;

    /// Remove track from playlist
    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()>;

    /// Get tracks in a playlist
    async fn get_playlist_tracks(&self, playlist_id: Uuid) -> Result<Vec<PlaylistTrack>>;
}

/// Trait for file storage operations (audio files, cover art, etc.)
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Read a file by path
    async fn read_file(&self, path: &str) -> Result<Vec<u8>>;

    /// Write a file
    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()>;

    /// Check if a file exists
    async fn file_exists(&self, path: &str) -> Result<bool>;

    /// Delete a file
    async fn delete_file(&self, path: &str) -> Result<()>;

    /// List files in a directory
    async fn list_files(&self, path: &str) -> Result<Vec<String>>;

    /// Get file metadata (size, modified time, etc.)
    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata>;
}

/// File metadata information
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub is_file: bool,
    pub is_dir: bool,
}

/// Combined storage trait that includes all storage operations
#[async_trait]
pub trait Storage: TrackStorage + AlbumStorage + ArtistStorage + UserStorage + PlaylistStorage + FileStorage {
    /// Initialize the storage backend
    async fn initialize(&self) -> Result<()>;

    /// Close the storage backend
    async fn close(&self) -> Result<()>;

    /// Check if the storage is healthy
    async fn health_check(&self) -> Result<bool>;
}
