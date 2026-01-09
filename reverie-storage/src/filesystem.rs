//! 基于文件系统的存储实现
//!
//! 此模块提供使用本地文件系统进行音频文件存储和元数据管理的存储实现。

use async_trait::async_trait;
use reverie_core::{Album, Artist, Playlist, PlaylistTrack, Track, User};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;

/// 文件系统存储配置
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    pub music_root: PathBuf,
    pub metadata_dir: PathBuf,
    pub cover_cache_dir: PathBuf,
    pub supported_extensions: Vec<&'static str>,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            music_root: PathBuf::from("./music"),
            metadata_dir: PathBuf::from("./metadata"),
            cover_cache_dir: PathBuf::from("./cache/covers"),
            supported_extensions: vec!["mp3", "flac", "m4a", "ogg", "opus", "wav", "aac"],
        }
    }
}

/// 媒体元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Option<f32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub comment: Option<String>,
}

/// 内部存储数据
#[derive(Default)]
struct StorageData {
    tracks: Vec<Track>,
    albums: Vec<Album>,
    artists: Vec<Artist>,
    users: Vec<User>,
    playlists: Vec<Playlist>,
}

/// 主要的文件系统存储实现
#[derive(Clone)]
pub struct FileSystemStorage {
    config: FileSystemConfig,
    data: Arc<Mutex<StorageData>>,
}

impl FileSystemStorage {
    pub async fn new() -> Result<Self> {
        Self::with_config(FileSystemConfig::default()).await
    }

    pub async fn with_config(config: FileSystemConfig) -> Result<Self> {
        tokio::fs::create_dir_all(&config.music_root).await?;
        tokio::fs::create_dir_all(&config.metadata_dir).await?;
        tokio::fs::create_dir_all(&config.cover_cache_dir).await?;

        Ok(Self {
            config,
            data: Arc::new(Mutex::new(StorageData::default())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        self.load_metadata().await?;
        self.ensure_default_user().await?;
        Ok(())
    }

    async fn load_metadata(&self) -> Result<()> {
        let metadata_dir = self.config.metadata_dir.clone();

        let tracks_path = metadata_dir.join("tracks.json");
        let albums_path = metadata_dir.join("albums.json");
        let artists_path = metadata_dir.join("artists.json");
        let users_path = metadata_dir.join("users.json");
        let playlists_path = metadata_dir.join("playlists.json");

        let mut tracks = Vec::new();
        let mut albums = Vec::new();
        let mut artists = Vec::new();
        let mut users = Vec::new();
        let mut playlists = Vec::new();

        if let Ok(data_str) = tokio::fs::read_to_string(&tracks_path).await {
            if let Ok(loaded) = serde_json::from_str(&data_str) {
                tracks = loaded;
            }
        }

        if let Ok(data_str) = tokio::fs::read_to_string(&albums_path).await {
            if let Ok(loaded) = serde_json::from_str(&data_str) {
                albums = loaded;
            }
        }

        if let Ok(data_str) = tokio::fs::read_to_string(&artists_path).await {
            if let Ok(loaded) = serde_json::from_str(&data_str) {
                artists = loaded;
            }
        }

        if let Ok(data_str) = tokio::fs::read_to_string(&users_path).await {
            if let Ok(loaded) = serde_json::from_str(&data_str) {
                users = loaded;
            }
        }

        if let Ok(data_str) = tokio::fs::read_to_string(&playlists_path).await {
            if let Ok(loaded) = serde_json::from_str(&data_str) {
                playlists = loaded;
            }
        }

        let mut data = self.data.lock().unwrap();
        data.tracks = tracks;
        data.albums = albums;
        data.artists = artists;
        data.users = users;
        data.playlists = playlists;

        Ok(())
    }

    async fn save_metadata(&self) -> Result<()> {
        let (tracks, albums, artists, users, playlists) = {
            let data = self.data.lock().unwrap();
            (
                data.tracks.clone(),
                data.albums.clone(),
                data.artists.clone(),
                data.users.clone(),
                data.playlists.clone(),
            )
        };

        let metadata_dir = self.config.metadata_dir.clone();

        let tracks_path = metadata_dir.join("tracks.json");
        let data_str = serde_json::to_string(&tracks).map_err(StorageError::SerializationError)?;
        tokio::fs::write(&tracks_path, data_str).await?;

        let albums_path = metadata_dir.join("albums.json");
        let data_str = serde_json::to_string(&albums).map_err(StorageError::SerializationError)?;
        tokio::fs::write(&albums_path, data_str).await?;

        let artists_path = metadata_dir.join("artists.json");
        let data_str = serde_json::to_string(&artists).map_err(StorageError::SerializationError)?;
        tokio::fs::write(&artists_path, data_str).await?;

        let users_path = metadata_dir.join("users.json");
        let data_str = serde_json::to_string(&users).map_err(StorageError::SerializationError)?;
        tokio::fs::write(&users_path, data_str).await?;

        let playlists_path = metadata_dir.join("playlists.json");
        let data_str =
            serde_json::to_string(&playlists).map_err(StorageError::SerializationError)?;
        tokio::fs::write(&playlists_path, data_str).await?;

        Ok(())
    }

    async fn ensure_default_user(&self) -> Result<()> {
        let user_count = {
            let data = self.data.lock().unwrap();
            data.users.len()
        };

        if user_count == 0 {
            let mut data = self.data.lock().unwrap();
            data.users.push(User {
                id: Uuid::new_v4(),
                username: "admin".to_string(),
                password_hash: "admin".to_string(),
                email: Some("admin@reverie.local".to_string()),
                is_admin: true,
                created_at: chrono::DateTime::from(std::time::SystemTime::now()),
                updated_at: chrono::DateTime::from(std::time::SystemTime::now()),
            });
        }
        Ok(())
    }
}

#[async_trait]
impl Storage for FileSystemStorage {
    async fn initialize(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.config.music_root).await?;
        tokio::fs::create_dir_all(&self.config.metadata_dir).await?;
        tokio::fs::create_dir_all(&self.config.cover_cache_dir).await?;
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.save_metadata().await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        tokio::fs::metadata(&self.config.music_root)
            .await
            .map_err(StorageError::IoError)?;
        Ok(true)
    }
}

#[async_trait]
impl TrackStorage for FileSystemStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let tracks = self.data.lock().unwrap();
        Ok(tracks.tracks.iter().find(|t| t.id == id).cloned())
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let mut tracks = self.data.lock().unwrap();
        tracks.tracks.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(tracks
            .tracks
            .clone()
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        {
            let mut tracks = self.data.lock().unwrap();
            if let Some(pos) = tracks.tracks.iter().position(|t| t.id == track.id) {
                tracks.tracks[pos] = track.clone();
            } else {
                tracks.tracks.push(track.clone());
            }
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn delete_track(&self, id: Uuid) -> Result<()> {
        {
            let mut tracks = self.data.lock().unwrap();
            tracks.tracks.retain(|t| t.id != id);
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let query_lower = query.to_lowercase();
        let tracks = self.data.lock().unwrap();
        let results: Vec<Track> = tracks
            .tracks
            .iter()
            .filter(|t| t.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.data.lock().unwrap();
        let results: Vec<Track> = tracks
            .tracks
            .iter()
            .filter(|t| t.album_id == Some(album_id))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.data.lock().unwrap();
        let results: Vec<Track> = tracks
            .tracks
            .iter()
            .filter(|t| t.artist_id == Some(artist_id))
            .cloned()
            .collect();
        Ok(results)
    }
}

#[async_trait]
impl AlbumStorage for FileSystemStorage {
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>> {
        let albums = self.data.lock().unwrap();
        Ok(albums.albums.iter().find(|a| a.id == id).cloned())
    }

    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        let mut albums = self.data.lock().unwrap();
        albums.albums.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(albums
            .albums
            .clone()
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    async fn save_album(&self, album: &Album) -> Result<()> {
        {
            let mut albums = self.data.lock().unwrap();
            if let Some(pos) = albums.albums.iter().position(|a| a.id == album.id) {
                albums.albums[pos] = album.clone();
            } else {
                albums.albums.push(album.clone());
            }
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn delete_album(&self, id: Uuid) -> Result<()> {
        {
            let mut albums = self.data.lock().unwrap();
            albums.albums.retain(|a| a.id != id);
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>> {
        let albums = self.data.lock().unwrap();
        let results: Vec<Album> = albums
            .albums
            .iter()
            .filter(|a| a.artist_id == Some(artist_id))
            .cloned()
            .collect();
        Ok(results)
    }
}

#[async_trait]
impl ArtistStorage for FileSystemStorage {
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>> {
        let artists = self.data.lock().unwrap();
        Ok(artists.artists.iter().find(|a| a.id == id).cloned())
    }

    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>> {
        let mut artists = self.data.lock().unwrap();
        artists.artists.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(artists
            .artists
            .clone()
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    async fn save_artist(&self, artist: &Artist) -> Result<()> {
        {
            let mut artists = self.data.lock().unwrap();
            if let Some(pos) = artists.artists.iter().position(|a| a.id == artist.id) {
                artists.artists[pos] = artist.clone();
            } else {
                artists.artists.push(artist.clone());
            }
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn delete_artist(&self, id: Uuid) -> Result<()> {
        {
            let mut artists = self.data.lock().unwrap();
            artists.artists.retain(|a| a.id != id);
        }
        self.save_metadata().await?;
        Ok(())
    }
}

#[async_trait]
impl UserStorage for FileSystemStorage {
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let users = self.data.lock().unwrap();
        Ok(users.users.iter().find(|u| u.id == id).cloned())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.data.lock().unwrap();
        Ok(users.users.iter().find(|u| u.username == username).cloned())
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>> {
        let users = self.data.lock().unwrap();
        Ok(users
            .users
            .clone()
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect())
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        {
            let mut users = self.data.lock().unwrap();
            if let Some(pos) = users.users.iter().position(|u| u.id == user.id) {
                users.users[pos] = user.clone();
            } else {
                users.users.push(user.clone());
            }
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        {
            let mut users = self.data.lock().unwrap();
            users.users.retain(|u| u.id != id);
        }
        self.save_metadata().await?;
        Ok(())
    }
}

#[async_trait]
impl PlaylistStorage for FileSystemStorage {
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>> {
        let playlists = self.data.lock().unwrap();
        Ok(playlists.playlists.iter().find(|p| p.id == id).cloned())
    }

    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>> {
        let playlists = self.data.lock().unwrap();
        let results: Vec<Playlist> = playlists
            .playlists
            .iter()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        {
            let mut playlists = self.data.lock().unwrap();
            if let Some(pos) = playlists.playlists.iter().position(|p| p.id == playlist.id) {
                playlists.playlists[pos] = playlist.clone();
            } else {
                playlists.playlists.push(playlist.clone());
            }
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<()> {
        {
            let mut playlists = self.data.lock().unwrap();
            playlists.playlists.retain(|p| p.id != id);
        }
        self.save_metadata().await?;
        Ok(())
    }

    async fn add_track_to_playlist(&self, _playlist_track: &PlaylistTrack) -> Result<()> {
        self.save_metadata().await?;
        Ok(())
    }

    async fn remove_track_from_playlist(&self, _playlist_id: Uuid, _track_id: Uuid) -> Result<()> {
        self.save_metadata().await?;
        Ok(())
    }

    async fn get_playlist_tracks(&self, _playlist_id: Uuid) -> Result<Vec<PlaylistTrack>> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl FileStorage for FileSystemStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let path = Path::new(path);
        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        Ok(tokio::fs::try_exists(path).await?)
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;
        while let Some(entry) = dir.next_entry().await? {
            entries.push(entry.path().to_string_lossy().to_string());
        }
        Ok(entries)
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(FileMetadata {
            size: metadata.len(),
            modified: metadata.modified().map_err(StorageError::IoError)?,
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
        })
    }
}
