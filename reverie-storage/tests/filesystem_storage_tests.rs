//! Filesystem storage tests

use chrono::Utc;
use reverie_core::{Album, Artist, Playlist, PlaylistTrack, Track, User};
use reverie_storage::filesystem::{FileSystemConfig, FileSystemStorage};
use reverie_storage::{
    AlbumStorage, ArtistStorage, FileStorage, PlaylistStorage, Storage, TrackStorage, UserStorage,
};
use uuid::Uuid;

#[tokio::test]
async fn test_filesystem_storage_config_defaults() {
    let config = FileSystemConfig::default();

    assert!(config.supported_extensions.contains(&"mp3"));
    assert!(config.supported_extensions.contains(&"flac"));
    assert_eq!(config.supported_extensions.len(), 7);
    assert_eq!(config.music_root, std::path::PathBuf::from("./music"));
    assert_eq!(config.metadata_dir, std::path::PathBuf::from("./metadata"));
    assert_eq!(
        config.cover_cache_dir,
        std::path::PathBuf::from("./cache/covers")
    );
}

#[tokio::test]
async fn test_filesystem_storage_config_clone() {
    let config = FileSystemConfig::default();
    let cloned = config.clone();
    assert_eq!(config.supported_extensions, cloned.supported_extensions);
    assert_eq!(config.music_root, cloned.music_root);
}

#[tokio::test]
async fn test_filesystem_storage_create_directories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let music_path = temp_dir.path().join("music");
    let metadata_path = temp_dir.path().join("metadata");
    let cache_path = temp_dir.path().join("cache");

    // This should succeed
    tokio::fs::create_dir_all(&music_path).await.unwrap();
    tokio::fs::create_dir_all(&metadata_path).await.unwrap();
    tokio::fs::create_dir_all(&cache_path).await.unwrap();

    assert!(music_path.exists());
    assert!(metadata_path.exists());
    assert!(cache_path.exists());
}

#[tokio::test]
async fn test_filesystem_storage_read_nonexistent_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let nonexistent = temp_dir.path().join("nonexistent.json");

    // Reading a nonexistent file should return an error
    let result = tokio::fs::read_to_string(&nonexistent).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_filesystem_storage_with_config_simple() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    // Create the storage first (directories created in with_config)
    let storage = FileSystemStorage::with_config(config).await;
    assert!(storage.is_ok());

    // Then initialize (loads metadata and creates default user)
    let storage = storage.unwrap();
    let result = storage.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_filesystem_storage_track_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let track = Track {
        id: Uuid::new_v4(),
        title: "Test Track".to_string(),
        album_id: None,
        artist_id: None,
        duration: 180,
        file_path: "/test/path.mp3".to_string(),
        file_size: 5000000,
        bitrate: 320,
        format: "mp3".to_string(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Test Genre".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Save track
    storage.save_track(&track).await.unwrap();

    // Get track
    let retrieved = storage.get_track(track.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, track.title);

    // List tracks
    let tracks = storage.list_tracks(10, 0).await.unwrap();
    assert_eq!(tracks.len(), 1);

    // Search tracks
    let search_results = storage.search_tracks("Test").await.unwrap();
    assert_eq!(search_results.len(), 1);

    // Search tracks (no match)
    let no_match = storage.search_tracks("NonExistent").await.unwrap();
    assert!(no_match.is_empty());

    // Delete track
    storage.delete_track(track.id).await.unwrap();
    let deleted = storage.get_track(track.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_filesystem_storage_album_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let album = Album {
        id: Uuid::new_v4(),
        name: "Test Album".to_string(),
        artist_id: None,
        year: Some(2024),
        genre: Some("Test Genre".to_string()),
        cover_art_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Save and retrieve album
    storage.save_album(&album).await.unwrap();
    let retrieved = storage.get_album(album.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, album.name);

    // List albums
    let albums = storage.list_albums(10, 0).await.unwrap();
    assert_eq!(albums.len(), 1);

    // Delete album
    storage.delete_album(album.id).await.unwrap();
    let deleted = storage.get_album(album.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_filesystem_storage_artist_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let artist = Artist {
        id: Uuid::new_v4(),
        name: "Test Artist".to_string(),
        bio: Some("Test bio".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.save_artist(&artist).await.unwrap();
    let retrieved = storage.get_artist(artist.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, artist.name);

    // List artists
    let artists = storage.list_artists(10, 0).await.unwrap();
    assert_eq!(artists.len(), 1);

    // Delete artist
    storage.delete_artist(artist.id).await.unwrap();
    let deleted = storage.get_artist(artist.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_filesystem_storage_user_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        password_hash: "hashed_password".to_string(),
        email: Some("test@example.com".to_string()),
        is_admin: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.save_user(&user).await.unwrap();

    // Get by ID
    let retrieved = storage.get_user(user.id).await.unwrap();
    assert!(retrieved.is_some());

    // Get by username
    let by_username = storage.get_user_by_username("testuser").await.unwrap();
    assert!(by_username.is_some());
    assert_eq!(by_username.unwrap().id, user.id);

    // List users
    let users = storage.list_users(10, 0).await.unwrap();
    assert!(users.len() >= 1); // At least the default admin user

    // Delete user
    storage.delete_user(user.id).await.unwrap();
    let deleted = storage.get_user(user.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_filesystem_storage_playlist_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let user_id = Uuid::new_v4();
    let playlist = Playlist {
        id: Uuid::new_v4(),
        name: "Test Playlist".to_string(),
        description: Some("Test description".to_string()),
        user_id,
        is_public: true,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.save_playlist(&playlist).await.unwrap();

    // Get playlist
    let retrieved = storage.get_playlist(playlist.id).await.unwrap();
    assert!(retrieved.is_some());

    // Get playlists by user
    let user_playlists = storage.get_playlists_by_user(user_id).await.unwrap();
    assert_eq!(user_playlists.len(), 1);

    // Add track to playlist
    let track_id = Uuid::new_v4();
    let playlist_track = PlaylistTrack {
        playlist_id: playlist.id,
        track_id,
        position: 1,
        added_at: Utc::now(),
    };
    storage
        .add_track_to_playlist(&playlist_track)
        .await
        .unwrap();

    // Get playlist tracks
    let tracks = storage.get_playlist_tracks(playlist.id).await.unwrap();
    assert_eq!(tracks.len(), 1);

    // Remove track from playlist
    storage
        .remove_track_from_playlist(playlist.id, track_id)
        .await
        .unwrap();
    let tracks = storage.get_playlist_tracks(playlist.id).await.unwrap();
    assert_eq!(tracks.len(), 0);

    // Delete playlist
    storage.delete_playlist(playlist.id).await.unwrap();
    let deleted = storage.get_playlist(playlist.id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_filesystem_storage_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let file_path = temp_dir.path().join("test_file.txt");
    let file_data = b"Test file content";

    // Write file
    storage
        .write_file(file_path.to_str().unwrap(), file_data)
        .await
        .unwrap();

    // Check file exists
    let exists = storage
        .file_exists(file_path.to_str().unwrap())
        .await
        .unwrap();
    assert!(exists);

    // Read file
    let read_data = storage
        .read_file(file_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(read_data, file_data);

    // Get file metadata
    let metadata = storage
        .get_file_metadata(file_path.to_str().unwrap())
        .await
        .unwrap();
    assert_eq!(metadata.size, file_data.len() as u64);
    assert!(metadata.is_file);

    // Delete file
    storage
        .delete_file(file_path.to_str().unwrap())
        .await
        .unwrap();
    let exists = storage
        .file_exists(file_path.to_str().unwrap())
        .await
        .unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_filesystem_storage_health_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let healthy = storage.health_check().await.unwrap();
    assert!(healthy);

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_filesystem_storage_update_track() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let track = Track {
        id: Uuid::new_v4(),
        title: "Original Title".to_string(),
        album_id: None,
        artist_id: None,
        duration: 180,
        file_path: "/test/path.mp3".to_string(),
        file_size: 5000000,
        bitrate: 320,
        format: "mp3".to_string(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Rock".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.save_track(&track).await.unwrap();

    // Update track
    let mut updated_track = track.clone();
    updated_track.title = "Updated Title".to_string();
    updated_track.duration = 200;
    storage.save_track(&updated_track).await.unwrap();

    // Verify update
    let retrieved = storage.get_track(track.id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Updated Title");
    assert_eq!(retrieved.unwrap().duration, 200);
}

#[tokio::test]
async fn test_filesystem_storage_pagination() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    // Create multiple tracks
    for i in 0..15 {
        let track = Track {
            id: Uuid::new_v4(),
            title: format!("Track {}", i),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: format!("/test/track{}.mp3", i),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(i),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await.unwrap();
    }

    // Test pagination
    let tracks = storage.list_tracks(10, 0).await.unwrap();
    assert_eq!(tracks.len(), 10);

    let tracks = storage.list_tracks(10, 10).await.unwrap();
    assert_eq!(tracks.len(), 5);

    let tracks = storage.list_tracks(5, 20).await.unwrap();
    assert!(tracks.is_empty());
}

#[tokio::test]
async fn test_filesystem_storage_get_tracks_by_album() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let album_id = Uuid::new_v4();

    // Create tracks for album
    for i in 1..=5 {
        let track = Track {
            id: Uuid::new_v4(),
            title: format!("Album Track {}", i),
            album_id: Some(album_id),
            artist_id: None,
            duration: 180,
            file_path: format!("/test/album_track{}.mp3", i),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(i),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await.unwrap();
    }

    // Get tracks by album
    let tracks = storage.get_tracks_by_album(album_id).await.unwrap();
    assert_eq!(tracks.len(), 5);
}

#[tokio::test]
async fn test_filesystem_storage_get_tracks_by_artist() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let artist_id = Uuid::new_v4();

    // Create tracks for artist
    for i in 1..=3 {
        let track = Track {
            id: Uuid::new_v4(),
            title: format!("Artist Track {}", i),
            album_id: None,
            artist_id: Some(artist_id),
            duration: 180,
            file_path: format!("/test/artist_track{}.mp3", i),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(i),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await.unwrap();
    }

    // Get tracks by artist
    let tracks = storage.get_tracks_by_artist(artist_id).await.unwrap();
    assert_eq!(tracks.len(), 3);
}

#[tokio::test]
async fn test_filesystem_storage_get_albums_by_artist() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().join("music").to_path_buf(),
        metadata_dir: temp_dir.path().join("metadata").to_path_buf(),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    storage.initialize().await.unwrap();

    let artist_id = Uuid::new_v4();

    // Create albums for artist
    for i in 1..=3 {
        let album = Album {
            id: Uuid::new_v4(),
            name: format!("Album {}", i),
            artist_id: Some(artist_id),
            year: Some(2020 + i),
            genre: Some("Rock".to_string()),
            cover_art_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_album(&album).await.unwrap();
    }

    // Get albums by artist
    let albums = storage.get_albums_by_artist(artist_id).await.unwrap();
    assert_eq!(albums.len(), 3);
}
