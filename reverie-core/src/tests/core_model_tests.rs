//! Core model tests for Track, Album, Artist, User, Playlist

use super::super::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn test_track_creation() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let track = Track {
        id,
        title: "Test Track".to_string(),
        album_id: Some(Uuid::new_v4()),
        artist_id: Some(Uuid::new_v4()),
        duration: 180,
        file_path: "/music/test.mp3".to_string(),
        file_size: 5_000_000,
        bitrate: 320,
        format: "mp3".to_string(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Rock".to_string()),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(track.title, "Test Track");
    assert_eq!(track.duration, 180);
    assert_eq!(track.bitrate, 320);
    assert_eq!(track.format, "mp3");
    assert!(track.album_id.is_some());
    assert!(track.artist_id.is_some());
}

#[test]
fn test_album_creation() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let album = Album {
        id,
        name: "Test Album".to_string(),
        artist_id: Some(Uuid::new_v4()),
        year: Some(2024),
        genre: Some("Pop".to_string()),
        cover_art_path: Some("/covers/test.jpg".to_string()),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(album.name, "Test Album");
    assert_eq!(album.year, Some(2024));
    assert!(album.cover_art_path.is_some());
}

#[test]
fn test_artist_creation() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let artist = Artist {
        id,
        name: "Test Artist".to_string(),
        bio: Some("A great artist".to_string()),
        created_at: now,
        updated_at: now,
    };

    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.bio, Some("A great artist".to_string()));
}

#[test]
fn test_user_creation() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let user = User {
        id,
        username: "testuser".to_string(),
        password_hash: "hashed_password".to_string(),
        email: Some("test@example.com".to_string()),
        is_admin: false,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, Some("test@example.com".to_string()));
    assert!(!user.is_admin);
    assert!(user.password_hash != "plain_password");
}

#[test]
fn test_playlist_creation() {
    let id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let playlist = Playlist {
        id,
        name: "My Playlist".to_string(),
        description: Some("A cool playlist".to_string()),
        user_id,
        is_public: true,
        created_at: now,
        updated_at: now,
    };

    assert_eq!(playlist.name, "My Playlist");
    assert!(playlist.is_public);
    assert_eq!(playlist.user_id, user_id);
}

#[test]
fn test_playlist_track_creation() {
    let playlist_id = Uuid::new_v4();
    let track_id = Uuid::new_v4();
    let playlist_track = PlaylistTrack {
        playlist_id,
        track_id,
        position: 1,
        added_at: Utc::now(),
    };

    assert_eq!(playlist_track.playlist_id, playlist_id);
    assert_eq!(playlist_track.track_id, track_id);
    assert_eq!(playlist_track.position, 1);
}

#[test]
fn test_scan_stats_defaults() {
    let stats = ScanStats {
        total_tracks: 1000,
        total_albums: 100,
        total_artists: 50,
        scanned_files: 800,
        new_tracks: 50,
        updated_tracks: 20,
        deleted_tracks: 5,
        errors: 2,
    };

    assert_eq!(stats.total_tracks, 1000);
    assert_eq!(stats.errors, 2);
}
