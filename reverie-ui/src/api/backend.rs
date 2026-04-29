use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;

use super::{Album, Artist, Song};

#[derive(Debug, Deserialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

#[derive(Debug, Deserialize)]
struct AlbumResponse {
    pub id: String,
    pub name: String,
    pub artist_id: Option<String>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TrackResponse {
    pub id: String,
    pub title: String,
    pub album_id: Option<String>,
    pub artist_id: Option<String>,
    pub duration: u32,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArtistResponse {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
}

async fn get_json<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    let client = reqwest::Client::new();
    client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

async fn artist_name_map(limit: usize) -> Result<HashMap<String, String>, String> {
    let resp: ListResponse<ArtistResponse> =
        get_json(&format!("/api/artists?limit={}&offset=0", limit)).await?;

    Ok(resp
        .items
        .into_iter()
        .map(|a| (a.id, a.name))
        .collect())
}

async fn album_name_map(limit: usize) -> Result<HashMap<String, String>, String> {
    let resp: ListResponse<AlbumResponse> =
        get_json(&format!("/api/albums?limit={}&offset=0", limit)).await?;

    Ok(resp
        .items
        .into_iter()
        .map(|a| (a.id, a.name))
        .collect())
}

pub async fn list_albums(limit: usize, offset: usize) -> Result<Vec<Album>, String> {
    let resp: ListResponse<AlbumResponse> =
        get_json(&format!("/api/albums?limit={}&offset={}", limit, offset)).await?;

    let artist_names = artist_name_map(2000).await.unwrap_or_default();

    Ok(resp
        .items
        .into_iter()
        .map(|a| Album {
            id: a.id,
            name: a.name,
            artist: a
                .artist_id
                .as_ref()
                .and_then(|id| artist_names.get(id).cloned()),
            artist_id: a.artist_id,
            cover_art: None,
            song_count: None,
            duration: None,
            year: a.year.map(|y| y as i32),
            genre: a.genre,
            created: None,
            starred: None,
            play_count: 0,
        })
        .collect())
}

pub async fn get_album(id: &str) -> Result<Album, String> {
    let a: AlbumResponse = get_json(&format!("/api/albums/{}", id)).await?;
    let artist_names = artist_name_map(2000).await.unwrap_or_default();

    Ok(Album {
        id: a.id,
        name: a.name,
        artist: a
            .artist_id
            .as_ref()
            .and_then(|id| artist_names.get(id).cloned()),
        artist_id: a.artist_id,
        cover_art: None,
        song_count: None,
        duration: None,
        year: a.year.map(|y| y as i32),
        genre: a.genre,
        created: None,
        starred: None,
        play_count: 0,
    })
}

pub async fn get_album_tracks(album_id: &str, limit_for_names: usize) -> Result<Vec<Song>, String> {
    let tracks: Vec<TrackResponse> = get_json(&format!("/api/albums/{}/tracks", album_id)).await?;

    let artist_names = artist_name_map(limit_for_names).await.unwrap_or_default();
    let album_names = album_name_map(limit_for_names).await.unwrap_or_default();

    Ok(tracks
        .into_iter()
        .map(|t| Song {
            id: t.id,
            title: t.title,
            album: t.album_id.as_ref().and_then(|id| album_names.get(id).cloned()),
            album_id: t.album_id,
            artist: t
                .artist_id
                .as_ref()
                .and_then(|id| artist_names.get(id).cloned()),
            artist_id: t.artist_id,
            track: t.track_number.map(|n| n as i32),
            year: t.year.map(|y| y as i32),
            genre: t.genre,
            cover_art: None,
            duration: Some(t.duration as i32),
            bit_rate: None,
            suffix: None,
            content_type: None,
            path: None,
            starred: None,
            play_count: 0,
        })
        .collect())
}

pub async fn list_tracks(limit: usize, offset: usize) -> Result<Vec<Song>, String> {
    let resp: ListResponse<TrackResponse> =
        get_json(&format!("/api/tracks?limit={}&offset={}", limit, offset)).await?;

    let artist_names = artist_name_map(2000).await.unwrap_or_default();
    let album_names = album_name_map(2000).await.unwrap_or_default();

    Ok(resp
        .items
        .into_iter()
        .map(|t| Song {
            id: t.id,
            title: t.title,
            album: t.album_id.as_ref().and_then(|id| album_names.get(id).cloned()),
            album_id: t.album_id,
            artist: t
                .artist_id
                .as_ref()
                .and_then(|id| artist_names.get(id).cloned()),
            artist_id: t.artist_id,
            track: t.track_number.map(|n| n as i32),
            year: t.year.map(|y| y as i32),
            genre: t.genre,
            cover_art: None,
            duration: Some(t.duration as i32),
            bit_rate: None,
            suffix: None,
            content_type: None,
            path: None,
            starred: None,
            play_count: 0,
        })
        .collect())
}

pub async fn list_artists(limit: usize, offset: usize) -> Result<Vec<Artist>, String> {
    let resp: ListResponse<ArtistResponse> =
        get_json(&format!("/api/artists?limit={}&offset={}", limit, offset)).await?;

    Ok(resp
        .items
        .into_iter()
        .map(|a| Artist {
            id: a.id,
            name: a.name,
            album_count: 0,
            cover_art: None,
            artist_image_url: None,
            starred: None,
        })
        .collect())
}

pub async fn get_artist(id: &str) -> Result<Artist, String> {
    let a: ArtistResponse = get_json(&format!("/api/artists/{}", id)).await?;

    Ok(Artist {
        id: a.id,
        name: a.name,
        album_count: 0,
        cover_art: None,
        artist_image_url: None,
        starred: None,
    })
}

pub async fn get_artist_albums(artist_id: &str) -> Result<Vec<Album>, String> {
    let albums: Vec<AlbumResponse> = get_json(&format!("/api/artists/{}/albums", artist_id)).await?;

    Ok(albums
        .into_iter()
        .map(|a| Album {
            id: a.id,
            name: a.name,
            artist: None,
            artist_id: a.artist_id,
            cover_art: None,
            song_count: None,
            duration: None,
            year: a.year.map(|y| y as i32),
            genre: a.genre,
            created: None,
            starred: None,
            play_count: 0,
        })
        .collect())
}
