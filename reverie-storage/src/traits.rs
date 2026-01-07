//! Storage abstraction traits
//!
//! These traits define the interface for storage operations,
//! allowing different implementations to be swapped without
//! changing the core application logic.

use crate::error::Result;
use async_trait::async_trait;
use reverie_core::{Album, Artist, Playlist, PlaylistTrack, Track, User};
use uuid::Uuid;

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
pub trait Storage:
    TrackStorage + AlbumStorage + ArtistStorage + UserStorage + PlaylistStorage + FileStorage
{
    /// Initialize the storage backend
    async fn initialize(&self) -> Result<()>;

    /// Close the storage backend
    async fn close(&self) -> Result<()>;

    /// Check if the storage is healthy
    async fn health_check(&self) -> Result<bool>;
}

use reverie_core::{
    MediaFile, SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndexes,
    SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre,
    SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying,
    SubsonicOpenSubsonicExtension, SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs,
    SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare,
    SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser,
};

/// Complete Subsonic API storage trait
/// Implements all methods needed for navidrome-compatible Subsonic API
#[async_trait]
pub trait SubsonicStorage: Send + Sync {
    // === System ===
    /// Get server license info (always valid for self-hosted)
    async fn get_license(&self) -> Result<bool> {
        Ok(true)
    }

    // === Browsing ===
    /// Get all configured music folders
    async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>>;

    /// Get artist indexes (A-Z grouped artists)
    async fn get_indexes(
        &self,
        music_folder_id: Option<i32>,
        if_modified_since: Option<i64>,
    ) -> Result<SubsonicArtistIndexes>;

    /// Get all genres with song/album counts
    async fn get_genres(&self) -> Result<Vec<SubsonicGenre>>;

    /// Get directory contents (for folder-based browsing)
    async fn get_music_directory(&self, id: &str) -> Result<Option<SubsonicDirectory>>;

    /// Get artists (ID3 tag based)
    async fn get_artists(&self, music_folder_id: Option<i32>) -> Result<SubsonicArtistIndexes>;

    /// Get single artist by ID
    async fn get_artist(&self, id: &str) -> Result<Option<SubsonicArtist>>;

    /// Get single album by ID
    async fn get_album(&self, id: &str) -> Result<Option<SubsonicAlbum>>;

    /// Get single song by ID
    async fn get_song(&self, id: &str) -> Result<Option<MediaFile>>;

    /// Get videos (not implemented, returns empty)
    async fn get_videos(&self) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    /// Get artist info (biography, images, similar artists)
    async fn get_artist_info(
        &self,
        id: &str,
        count: Option<i32>,
        include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo>;

    /// Get artist info (ID3 version)
    async fn get_artist_info2(
        &self,
        id: &str,
        count: Option<i32>,
        include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo>;

    /// Get album info (notes, images)
    async fn get_album_info(&self, id: &str) -> Result<SubsonicAlbumInfo>;

    /// Get album info (ID3 version)
    async fn get_album_info2(&self, id: &str) -> Result<SubsonicAlbumInfo>;

    /// Get similar songs
    async fn get_similar_songs(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>>;

    /// Get similar songs (ID3 version)
    async fn get_similar_songs2(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>>;

    /// Get top songs for an artist
    async fn get_top_songs(&self, artist: &str, count: Option<i32>) -> Result<SubsonicTopSongs>;

    // === Album/Song Lists ===
    /// Get album list (various sort types)
    async fn get_album_list(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>>;

    /// Get album list (ID3 version)
    async fn get_album_list2(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>>;

    /// Get random songs
    async fn get_random_songs(
        &self,
        size: Option<i32>,
        genre: Option<&str>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>>;

    /// Get songs by genre
    async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: Option<i32>,
        offset: Option<i32>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>>;

    /// Get now playing entries
    async fn get_now_playing(&self) -> Result<Vec<SubsonicNowPlaying>>;

    /// Get starred items
    async fn get_starred(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred>;

    /// Get starred items (ID3 version)
    async fn get_starred2(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred>;

    // === Searching ===
    /// Search (deprecated, use search2/search3)
    async fn search(
        &self,
        artist: Option<&str>,
        album: Option<&str>,
        title: Option<&str>,
        any: Option<&str>,
        _count: Option<i32>,
        _offset: Option<i32>,
        _newer_than: Option<i64>,
    ) -> Result<SubsonicSearchResult2> {
        // Default implementation using search2
        let query = any.or(title).or(album).or(artist).unwrap_or("");
        self.search2(query, None, None, None, None, None, None)
            .await
    }

    /// Search2 (folder-based)
    async fn search2(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult2>;

    /// Search3 (ID3-based)
    async fn search3(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult3>;

    // === Playlists ===
    /// Get all playlists
    async fn get_playlists(&self, username: Option<&str>) -> Result<Vec<SubsonicPlaylist>>;

    /// Get single playlist with songs
    async fn get_playlist(&self, id: &str) -> Result<Option<SubsonicPlaylistWithSongs>>;

    /// Create playlist
    async fn create_playlist(
        &self,
        name: Option<&str>,
        playlist_id: Option<&str>,
        song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs>;

    /// Update playlist
    async fn update_playlist(
        &self,
        playlist_id: &str,
        name: Option<&str>,
        comment: Option<&str>,
        public: Option<bool>,
        song_ids_to_add: &[&str],
        song_indexes_to_remove: &[i32],
    ) -> Result<()>;

    /// Delete playlist
    async fn delete_playlist(&self, id: &str) -> Result<()>;

    // === Media Retrieval (paths only, actual streaming handled by network layer) ===
    /// Get file path for streaming
    async fn get_stream_path(&self, id: &str) -> Result<Option<String>>;

    /// Get cover art path
    async fn get_cover_art_path(&self, id: &str) -> Result<Option<String>>;

    /// Get lyrics
    async fn get_lyrics(
        &self,
        artist: Option<&str>,
        title: Option<&str>,
    ) -> Result<Option<SubsonicLyrics>>;

    /// Get lyrics by song ID (OpenSubsonic)
    async fn get_lyrics_by_song_id(&self, id: &str) -> Result<Vec<SubsonicStructuredLyrics>>;

    /// Get avatar path for user
    async fn get_avatar_path(&self, username: &str) -> Result<Option<String>>;

    // === Media Annotation ===
    /// Star items (add to favorites)
    async fn star(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()>;

    /// Unstar items (remove from favorites)
    async fn unstar(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()>;

    /// Set rating (0-5)
    async fn set_rating(&self, id: &str, rating: i32) -> Result<()>;

    /// Scrobble (record play)
    async fn scrobble(&self, id: &str, time: Option<i64>, submission: bool) -> Result<()>;

    // === Bookmarks ===
    /// Get all bookmarks for user
    async fn get_bookmarks(&self) -> Result<Vec<SubsonicBookmark>>;

    /// Create/update bookmark
    async fn create_bookmark(&self, id: &str, position: i64, comment: Option<&str>) -> Result<()>;

    /// Delete bookmark
    async fn delete_bookmark(&self, id: &str) -> Result<()>;

    /// Get play queue
    async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>>;

    /// Save play queue
    async fn save_play_queue(
        &self,
        ids: &[&str],
        current: Option<&str>,
        position: Option<i64>,
    ) -> Result<()>;

    // === Sharing ===
    /// Get all shares
    async fn get_shares(&self) -> Result<Vec<SubsonicShare>>;

    /// Create share
    async fn create_share(
        &self,
        ids: &[&str],
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<SubsonicShare>;

    /// Update share
    async fn update_share(
        &self,
        id: &str,
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<()>;

    /// Delete share
    async fn delete_share(&self, id: &str) -> Result<()>;

    // === Internet Radio ===
    /// Get all internet radio stations
    async fn get_internet_radio_stations(&self) -> Result<Vec<SubsonicInternetRadioStation>>;

    /// Create internet radio station
    async fn create_internet_radio_station(
        &self,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()>;

    /// Update internet radio station
    async fn update_internet_radio_station(
        &self,
        id: &str,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()>;

    /// Delete internet radio station
    async fn delete_internet_radio_station(&self, id: &str) -> Result<()>;

    // === User Management ===
    /// Get user by username
    async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>>;

    /// Get all users
    async fn get_users(&self) -> Result<Vec<SubsonicUser>>;

    /// Create user
    async fn create_user(
        &self,
        username: &str,
        password: &str,
        email: Option<&str>,
        admin_role: bool,
        settings_role: bool,
        stream_role: bool,
        jukebox_role: bool,
        download_role: bool,
        upload_role: bool,
        playlist_role: bool,
        cover_art_role: bool,
        comment_role: bool,
        podcast_role: bool,
        share_role: bool,
        video_conversion_role: bool,
        music_folder_ids: &[i32],
    ) -> Result<()>;

    /// Update user
    async fn update_user(
        &self,
        username: &str,
        password: Option<&str>,
        email: Option<&str>,
        admin_role: Option<bool>,
        settings_role: Option<bool>,
        stream_role: Option<bool>,
        jukebox_role: Option<bool>,
        download_role: Option<bool>,
        upload_role: Option<bool>,
        playlist_role: Option<bool>,
        cover_art_role: Option<bool>,
        comment_role: Option<bool>,
        podcast_role: Option<bool>,
        share_role: Option<bool>,
        video_conversion_role: Option<bool>,
        music_folder_ids: Option<&[i32]>,
        max_bit_rate: Option<i32>,
    ) -> Result<()>;

    /// Delete user
    async fn delete_user(&self, username: &str) -> Result<()>;

    /// Change password
    async fn change_password(&self, username: &str, password: &str) -> Result<()>;

    // === Library Scanning ===
    /// Get scan status
    async fn get_scan_status(&self) -> Result<SubsonicScanStatus>;

    /// Start library scan
    async fn start_scan(&self) -> Result<SubsonicScanStatus>;

    // === OpenSubsonic Extensions ===
    /// Get supported OpenSubsonic extensions
    async fn get_open_subsonic_extensions(&self) -> Result<Vec<SubsonicOpenSubsonicExtension>> {
        Ok(vec![
            SubsonicOpenSubsonicExtension {
                name: "transcodeOffset".to_string(),
                versions: vec![1],
            },
            SubsonicOpenSubsonicExtension {
                name: "formPost".to_string(),
                versions: vec![1],
            },
            SubsonicOpenSubsonicExtension {
                name: "songLyrics".to_string(),
                versions: vec![1],
            },
        ])
    }
}
