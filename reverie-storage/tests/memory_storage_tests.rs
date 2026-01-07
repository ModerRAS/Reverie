//! Integration tests for memory storage implementation

use chrono::Utc;
use reverie_core::{Album, Artist, Playlist, PlaylistTrack, Track, User};
use reverie_storage::memory::MemoryStorage;
use reverie_storage::{
    AlbumStorage, ArtistStorage, FileStorage, PlaylistStorage, Storage, TrackStorage, UserStorage,
};
use uuid::Uuid;

#[tokio::test]
async fn test_memory_storage_track_operations() {
    let storage = MemoryStorage::new();

    // Initialize storage
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    // Create a test track
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

    // Test save track
    storage
        .save_track(&track)
        .await
        .expect("Failed to save track");

    // Test get track
    let retrieved = storage
        .get_track(track.id)
        .await
        .expect("Failed to get track");
    assert!(retrieved.is_some());
    let retrieved_track = retrieved.unwrap();
    assert_eq!(retrieved_track.id, track.id);
    assert_eq!(retrieved_track.title, track.title);

    // Test list tracks
    let tracks = storage
        .list_tracks(10, 0)
        .await
        .expect("Failed to list tracks");
    assert_eq!(tracks.len(), 1);

    // Test search tracks
    let search_results = storage
        .search_tracks("Test")
        .await
        .expect("Failed to search tracks");
    assert_eq!(search_results.len(), 1);

    // Test delete track
    storage
        .delete_track(track.id)
        .await
        .expect("Failed to delete track");
    let deleted = storage
        .get_track(track.id)
        .await
        .expect("Failed to get deleted track");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_memory_storage_album_operations() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

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
    storage
        .save_album(&album)
        .await
        .expect("Failed to save album");
    let retrieved = storage
        .get_album(album.id)
        .await
        .expect("Failed to get album");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, album.name);

    // List albums
    let albums = storage
        .list_albums(10, 0)
        .await
        .expect("Failed to list albums");
    assert_eq!(albums.len(), 1);

    // Delete album
    storage
        .delete_album(album.id)
        .await
        .expect("Failed to delete album");
    let deleted = storage
        .get_album(album.id)
        .await
        .expect("Failed to get deleted album");
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_memory_storage_artist_operations() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    let artist = Artist {
        id: Uuid::new_v4(),
        name: "Test Artist".to_string(),
        bio: Some("Test bio".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage
        .save_artist(&artist)
        .await
        .expect("Failed to save artist");
    let retrieved = storage
        .get_artist(artist.id)
        .await
        .expect("Failed to get artist");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, artist.name);
}

#[tokio::test]
async fn test_memory_storage_user_operations() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    let user = User {
        id: Uuid::new_v4(),
        username: "testuser".to_string(),
        password_hash: "hashed_password".to_string(),
        email: Some("test@example.com".to_string()),
        is_admin: false,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    storage.save_user(&user).await.expect("Failed to save user");

    // Get by ID
    let retrieved = storage.get_user(user.id).await.expect("Failed to get user");
    assert!(retrieved.is_some());

    // Get by username
    let by_username = storage
        .get_user_by_username("testuser")
        .await
        .expect("Failed to get user by username");
    assert!(by_username.is_some());
    assert_eq!(by_username.unwrap().id, user.id);
}

#[tokio::test]
async fn test_memory_storage_playlist_operations() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

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

    storage
        .save_playlist(&playlist)
        .await
        .expect("Failed to save playlist");

    // Get playlist
    let retrieved = storage
        .get_playlist(playlist.id)
        .await
        .expect("Failed to get playlist");
    assert!(retrieved.is_some());

    // Get playlists by user
    let user_playlists = storage
        .get_playlists_by_user(user_id)
        .await
        .expect("Failed to get user playlists");
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
        .expect("Failed to add track to playlist");

    // Get playlist tracks
    let tracks = storage
        .get_playlist_tracks(playlist.id)
        .await
        .expect("Failed to get playlist tracks");
    assert_eq!(tracks.len(), 1);

    // Remove track from playlist
    storage
        .remove_track_from_playlist(playlist.id, track_id)
        .await
        .expect("Failed to remove track from playlist");
    let tracks = storage
        .get_playlist_tracks(playlist.id)
        .await
        .expect("Failed to get playlist tracks after removal");
    assert_eq!(tracks.len(), 0);
}

#[tokio::test]
async fn test_memory_storage_file_operations() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    let file_path = "/test/file.txt";
    let file_data = b"Test file content";

    // Write file
    storage
        .write_file(file_path, file_data)
        .await
        .expect("Failed to write file");

    // Check file exists
    let exists = storage
        .file_exists(file_path)
        .await
        .expect("Failed to check file exists");
    assert!(exists);

    // Read file
    let read_data = storage
        .read_file(file_path)
        .await
        .expect("Failed to read file");
    assert_eq!(read_data, file_data);

    // Get file metadata
    let metadata = storage
        .get_file_metadata(file_path)
        .await
        .expect("Failed to get file metadata");
    assert_eq!(metadata.size, file_data.len() as u64);
    assert!(metadata.is_file);

    // List files
    let files = storage
        .list_files("/test")
        .await
        .expect("Failed to list files");
    assert!(files.contains(&file_path.to_string()));

    // Delete file
    storage
        .delete_file(file_path)
        .await
        .expect("Failed to delete file");
    let exists = storage
        .file_exists(file_path)
        .await
        .expect("Failed to check file exists after deletion");
    assert!(!exists);
}

#[tokio::test]
async fn test_memory_storage_health_check() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    let healthy = storage
        .health_check()
        .await
        .expect("Failed to check health");
    assert!(healthy);

    storage.close().await.expect("Failed to close storage");
}

#[tokio::test]
async fn test_memory_storage_relationships() {
    let storage = MemoryStorage::new();
    storage
        .initialize()
        .await
        .expect("Failed to initialize storage");

    // Create artist
    let artist = Artist {
        id: Uuid::new_v4(),
        name: "Test Artist".to_string(),
        bio: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage
        .save_artist(&artist)
        .await
        .expect("Failed to save artist");

    // Create album for artist
    let album = Album {
        id: Uuid::new_v4(),
        name: "Test Album".to_string(),
        artist_id: Some(artist.id),
        year: Some(2024),
        genre: None,
        cover_art_path: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    storage
        .save_album(&album)
        .await
        .expect("Failed to save album");

    // Create tracks for album
    for i in 1..=3 {
        let track = Track {
            id: Uuid::new_v4(),
            title: format!("Track {}", i),
            album_id: Some(album.id),
            artist_id: Some(artist.id),
            duration: 180,
            file_path: format!("/test/track{}.mp3", i),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(i),
            disc_number: Some(1),
            year: Some(2024),
            genre: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage
            .save_track(&track)
            .await
            .expect("Failed to save track");
    }

    // Test get albums by artist
    let artist_albums = storage
        .get_albums_by_artist(artist.id)
        .await
        .expect("Failed to get artist albums");
    assert_eq!(artist_albums.len(), 1);

    // Test get tracks by album
    let album_tracks = storage
        .get_tracks_by_album(album.id)
        .await
        .expect("Failed to get album tracks");
    assert_eq!(album_tracks.len(), 3);

    // Test get tracks by artist
    let artist_tracks = storage
        .get_tracks_by_artist(artist.id)
        .await
        .expect("Failed to get artist tracks");
    assert_eq!(artist_tracks.len(), 3);
}
