//! UserStorage + PlaylistStorage 实现

use crate::error::Result;
use crate::traits::*;

use async_trait::async_trait;
use reverie_core::{Playlist, PlaylistTrack, User};
use uuid::Uuid;

use super::core::MemoryStorage;

#[async_trait]
impl UserStorage for MemoryStorage {
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.username == username).cloned())
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>> {
        let users = self.users.read().await;
        Ok(users.values().skip(offset).take(limit).cloned().collect())
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
        Ok(playlists
            .values()
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
        playlist_tracks
            .entry(playlist_track.playlist_id)
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
        Ok(playlist_tracks
            .get(&playlist_id)
            .cloned()
            .unwrap_or_default())
    }
}
