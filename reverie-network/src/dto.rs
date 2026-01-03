//! Data Transfer Objects (DTOs) for API requests and responses

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Response for track information
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackResponse {
    pub id: Uuid,
    pub title: String,
    pub album_id: Option<Uuid>,
    pub artist_id: Option<Uuid>,
    pub duration: u32,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// Response for album information
#[derive(Debug, Serialize, Deserialize)]
pub struct AlbumResponse {
    pub id: Uuid,
    pub name: String,
    pub artist_id: Option<Uuid>,
    pub year: Option<u32>,
    pub genre: Option<String>,
}

/// Response for artist information
#[derive(Debug, Serialize, Deserialize)]
pub struct ArtistResponse {
    pub id: Uuid,
    pub name: String,
    pub bio: Option<String>,
}

/// Response for playlist information
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaylistResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub is_public: bool,
}

/// Request to create a new playlist
#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
}

/// Request to add tracks to a playlist
#[derive(Debug, Deserialize)]
pub struct AddTrackToPlaylistRequest {
    pub track_id: Uuid,
}

/// Generic list response with pagination
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub offset: usize,
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
