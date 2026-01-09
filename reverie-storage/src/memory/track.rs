//! TrackStorage 实现

use crate::error::Result;
use crate::traits::*;

use async_trait::async_trait;
use reverie_core::Track;
use uuid::Uuid;

use super::core::MemoryStorage;

#[async_trait]
impl TrackStorage for MemoryStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.get(&id).cloned())
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.values().skip(offset).take(limit).cloned().collect())
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
        Ok(tracks
            .values()
            .filter(|t| t.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks
            .values()
            .filter(|t| t.album_id == Some(album_id))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks
            .values()
            .filter(|t| t.artist_id == Some(artist_id))
            .cloned()
            .collect())
    }
}
