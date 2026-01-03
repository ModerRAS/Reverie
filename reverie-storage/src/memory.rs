//! In-memory storage implementation for testing and development

use async_trait::async_trait;
use reverie_core::{Track, Album, Artist, User, Playlist, PlaylistTrack};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::{Result, StorageError};
use crate::traits::*;

/// In-memory storage implementation using HashMaps
#[derive(Clone)]
pub struct MemoryStorage {
    tracks: Arc<RwLock<HashMap<Uuid, Track>>>,
    albums: Arc<RwLock<HashMap<Uuid, Album>>>,
    artists: Arc<RwLock<HashMap<Uuid, Artist>>>,
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    playlists: Arc<RwLock<HashMap<Uuid, Playlist>>>,
    playlist_tracks: Arc<RwLock<HashMap<Uuid, Vec<PlaylistTrack>>>>,
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(HashMap::new())),
            albums: Arc::new(RwLock::new(HashMap::new())),
            artists: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            playlists: Arc::new(RwLock::new(HashMap::new())),
            playlist_tracks: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TrackStorage for MemoryStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.get(&id).cloned())
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        let mut tracks = self.tracks.write().await;
        tracks.insert(track.id, track.clone());
        Ok(())
    }

    async fn delete_track(&self, id: Uuid) -> Result<()> {
        let mut tracks = self.tracks.write().await;
        tracks.remove(&id);
        Ok(())
    }

    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        let query_lower = query.to_lowercase();
        Ok(tracks.values()
            .filter(|t| t.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.values()
            .filter(|t| t.album_id == Some(album_id))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.values()
            .filter(|t| t.artist_id == Some(artist_id))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl AlbumStorage for MemoryStorage {
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.get(&id).cloned())
    }

    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn save_album(&self, album: &Album) -> Result<()> {
        let mut albums = self.albums.write().await;
        albums.insert(album.id, album.clone());
        Ok(())
    }

    async fn delete_album(&self, id: Uuid) -> Result<()> {
        let mut albums = self.albums.write().await;
        albums.remove(&id);
        Ok(())
    }

    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.values()
            .filter(|a| a.artist_id == Some(artist_id))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl ArtistStorage for MemoryStorage {
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>> {
        let artists = self.artists.read().await;
        Ok(artists.get(&id).cloned())
    }

    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>> {
        let artists = self.artists.read().await;
        Ok(artists.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn save_artist(&self, artist: &Artist) -> Result<()> {
        let mut artists = self.artists.write().await;
        artists.insert(artist.id, artist.clone());
        Ok(())
    }

    async fn delete_artist(&self, id: Uuid) -> Result<()> {
        let mut artists = self.artists.write().await;
        artists.remove(&id);
        Ok(())
    }
}

#[async_trait]
impl UserStorage for MemoryStorage {
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.values()
            .find(|u| u.username == username)
            .cloned())
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>> {
        let users = self.users.read().await;
        Ok(users.values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let mut users = self.users.write().await;
        users.remove(&id);
        Ok(())
    }
}

#[async_trait]
impl PlaylistStorage for MemoryStorage {
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>> {
        let playlists = self.playlists.read().await;
        Ok(playlists.get(&id).cloned())
    }

    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>> {
        let playlists = self.playlists.read().await;
        Ok(playlists.values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        let mut playlists = self.playlists.write().await;
        playlists.insert(playlist.id, playlist.clone());
        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<()> {
        let mut playlists = self.playlists.write().await;
        playlists.remove(&id);
        Ok(())
    }

    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()> {
        let mut playlist_tracks = self.playlist_tracks.write().await;
        playlist_tracks.entry(playlist_track.playlist_id)
            .or_insert_with(Vec::new)
            .push(playlist_track.clone());
        Ok(())
    }

    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()> {
        let mut playlist_tracks = self.playlist_tracks.write().await;
        if let Some(tracks) = playlist_tracks.get_mut(&playlist_id) {
            tracks.retain(|pt| pt.track_id != track_id);
        }
        Ok(())
    }

    async fn get_playlist_tracks(&self, playlist_id: Uuid) -> Result<Vec<PlaylistTrack>> {
        let playlist_tracks = self.playlist_tracks.read().await;
        Ok(playlist_tracks.get(&playlist_id).cloned().unwrap_or_default())
    }
}

#[async_trait]
impl FileStorage for MemoryStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self.files.read().await;
        files.get(path)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let mut files = self.files.write().await;
        files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        let files = self.files.read().await;
        Ok(files.contains_key(path))
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let mut files = self.files.write().await;
        files.remove(path);
        Ok(())
    }

    async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let files = self.files.read().await;
        Ok(files.keys()
            .filter(|k| k.starts_with(path))
            .cloned()
            .collect())
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let files = self.files.read().await;
        files.get(path)
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
            .map(|data| FileMetadata {
                size: data.len() as u64,
                modified: std::time::SystemTime::now(),
                is_file: true,
                is_dir: false,
            })
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}
