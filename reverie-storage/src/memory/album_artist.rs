//! AlbumStorage + ArtistStorage 实现

use crate::error::Result;
use crate::traits::*;

use async_trait::async_trait;
use reverie_core::{Album, Artist};
use uuid::Uuid;

use super::core::MemoryStorage;

#[async_trait]
impl AlbumStorage for MemoryStorage {
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.get(&id).cloned())
    }

    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.values().skip(offset).take(limit).cloned().collect())
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
        Ok(albums
            .values()
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
        Ok(artists.values().skip(offset).take(limit).cloned().collect())
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
