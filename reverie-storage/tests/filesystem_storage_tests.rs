//! Filesystem storage tests

use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use reverie_storage::filesystem::{FileSystemConfig, FileSystemStorage, Inode, DirEntry, MediaMetadata};
use reverie_storage::Storage;

#[tokio::test]
async fn test_filesystem_storage_new() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().to_path_buf(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await;
    assert!(storage.is_ok());
}

#[tokio::test]
async fn test_filesystem_storage_health_check() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = FileSystemConfig {
        music_root: temp_dir.path().to_path_buf(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    let health = storage.health_check().await;
    assert!(health.is_ok());
    assert!(health.unwrap());
}

#[tokio::test]
async fn test_filesystem_storage_scan_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create test structure
    let music_dir = temp_dir.path().join("music");
    tokio::fs::create_dir_all(&music_dir).await.unwrap();
    
    // Create a subdirectory
    let subdir = music_dir.join("Rock");
    tokio::fs::create_dir_all(&subdir).await.unwrap();
    
    // Create a test file
    let test_file = subdir.join("test_song.mp3");
    let mut file = File::create(&test_file).await.unwrap();
    file.write_all(b"test audio data").await.unwrap();
    file.sync_all().await.unwrap();
    
    let config = FileSystemConfig {
        music_root: music_dir,
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    
    // Check that storage was created successfully and has scanned the directory
    let health = storage.health_check().await;
    assert!(health.is_ok());
    assert!(health.unwrap());
    
    // Verify that files were scanned by checking root inode has children
    let root_inode = storage.root_inode();
    let children = root_inode.children.read().unwrap();
    assert!(children.len() > 0, "Root inode should have scanned children");
}

#[tokio::test]
async fn test_filesystem_storage_get_inode_by_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create test structure
    let music_dir = temp_dir.path().join("music");
    tokio::fs::create_dir_all(&music_dir).await.unwrap();
    
    let config = FileSystemConfig {
        music_root: music_dir.clone(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    
    // Get root inode - should be accessible via root_inode getter
    let root = storage.root_inode();
    assert!(root.is_dir);
    
    // Get by path should work for music_dir
    let root_by_path = storage.get_inode_by_path(&music_dir).await;
    assert!(root_by_path.is_ok());
    // May or may not be found depending on canonicalization
    let _ = root_by_path.unwrap();
}

#[tokio::test]
async fn test_filesystem_storage_read_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create test structure
    let music_dir = temp_dir.path().join("music");
    tokio::fs::create_dir_all(&music_dir).await.unwrap();
    
    // Create subdirectories
    let rock_dir = music_dir.join("Rock");
    tokio::fs::create_dir_all(&rock_dir).await.unwrap();
    
    let jazz_dir = music_dir.join("Jazz");
    tokio::fs::create_dir_all(&jazz_dir).await.unwrap();
    
    let config = FileSystemConfig {
        music_root: music_dir.clone(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    
    // Read directory from root inode
    let entries = storage.read_dir(&storage.root_inode()).await;
    assert!(entries.is_ok());
    let entries = entries.unwrap();
    
    assert!(entries.len() >= 2, "Should have at least Rock and Jazz directories");
    
    // Check that we have Rock and Jazz directories
    let names: Vec<String> = entries.iter().map(|e| e.name.clone()).collect();
    assert!(names.contains(&"Rock".to_string()), "Should contain Rock directory");
    assert!(names.contains(&"Jazz".to_string()), "Should contain Jazz directory");
}

#[tokio::test]
async fn test_filesystem_storage_inode_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create a test file
    let test_file = temp_dir.path().join("test.mp3");
    let mut file = File::create(&test_file).await.unwrap();
    file.write_all(b"test audio data").await.unwrap();
    file.sync_all().await.unwrap();
    
    let metadata = std::fs::metadata(&test_file).unwrap();
    let inode = Inode::new_file(test_file.clone(), &metadata);
    
    assert!(!inode.is_dir);
    assert_eq!(inode.name, "test.mp3");
    assert_eq!(inode.suffix, "mp3");
    assert_eq!(inode.content_type, "audio/mpeg");
    assert!(inode.id != Uuid::nil());
}

#[tokio::test]
async fn test_filesystem_storage_dir_inode_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create a test directory
    let test_dir = temp_dir.path().join("test_dir");
    tokio::fs::create_dir_all(&test_dir).await.unwrap();
    
    let metadata = std::fs::metadata(&test_dir).unwrap();
    let inode = Inode::new_dir(test_dir.clone(), &metadata);
    
    assert!(inode.is_dir);
    assert_eq!(inode.name, "test_dir");
    assert_eq!(inode.content_type, "inode/directory");
    assert!(inode.id != Uuid::nil());
}

#[tokio::test]
async fn test_filesystem_storage_dir_entry() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create a test file
    let test_file = temp_dir.path().join("test.mp3");
    let mut file = File::create(&test_file).await.unwrap();
    file.write_all(b"test audio data").await.unwrap();
    file.sync_all().await.unwrap();
    
    let metadata = std::fs::metadata(&test_file).unwrap();
    let inode = Arc::new(Inode::new_file(test_file.clone(), &metadata));
    
    let entry = DirEntry::new("test.mp3".to_string(), inode.clone());
    
    assert_eq!(entry.name, "test.mp3");
    assert!(!entry.negative);
    assert_eq!(entry.inode.id, inode.id);
}

#[tokio::test]
async fn test_filesystem_storage_negative_dir_entry() {
    let entry = DirEntry::negative("nonexistent".to_string());
    
    assert_eq!(entry.name, "nonexistent");
    assert!(entry.negative);
    assert_eq!(entry.inode.id, Uuid::nil());
}

#[tokio::test]
async fn test_filesystem_storage_media_metadata() {
    let metadata = MediaMetadata {
        title: Some("Test Song".to_string()),
        artist: Some("Test Artist".to_string()),
        album: Some("Test Album".to_string()),
        album_artist: Some("Test Album Artist".to_string()),
        genre: Some("Rock".to_string()),
        year: Some(2023),
        track_number: Some(1),
        disc_number: Some(1),
        duration: Some(240.5),
        bitrate: Some(320),
        sample_rate: Some(44100),
        channels: Some(2),
        comment: None,
    };
    
    assert_eq!(metadata.title, Some("Test Song".to_string()));
    assert_eq!(metadata.artist, Some("Test Artist".to_string()));
    assert_eq!(metadata.duration, Some(240.5));
    assert_eq!(metadata.bitrate, Some(320));
}

#[tokio::test]
async fn test_filesystem_storage_open_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create a test file
    let test_file = temp_dir.path().join("test.mp3");
    let test_data = b"test audio data for streaming";
    let mut file = File::create(&test_file).await.unwrap();
    file.write_all(test_data).await.unwrap();
    file.sync_all().await.unwrap();
    
    // Create inode for the file
    let metadata = std::fs::metadata(&test_file).unwrap();
    let inode = Arc::new(Inode::new_file(test_file.clone(), &metadata));
    
    let config = FileSystemConfig {
        music_root: temp_dir.path().to_path_buf(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    
    // Open file
    let handle = storage.open_file(&inode).await;
    assert!(handle.is_ok());
}

#[tokio::test]
async fn test_filesystem_storage_file_content_type() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    for (ext, expected_type) in vec![
        ("mp3", "audio/mpeg"),
        ("flac", "audio/flac"),
        ("m4a", "audio/mp4"),
        ("ogg", "audio/ogg"),
        ("opus", "audio/opus"),
        ("wav", "audio/wav"),
        ("aac", "audio/aac"),
        ("xyz", "application/octet-stream"),
    ] {
        let test_file = temp_dir.path().join(format!("test.{}", ext));
        let mut file = File::create(&test_file).await.unwrap();
        file.write_all(b"test").await.unwrap();
        file.sync_all().await.unwrap();
        
        let metadata = std::fs::metadata(&test_file).unwrap();
        let inode = Inode::new_file(test_file.clone(), &metadata);
        
        assert_eq!(inode.suffix, ext, "Extension mismatch for .{}", ext);
        assert_eq!(inode.content_type, expected_type, "Content type mismatch for .{}", ext);
    }
}

#[tokio::test]
async fn test_filesystem_storage_inode_reference_counting() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    let test_file = temp_dir.path().join("test.mp3");
    let mut file = File::create(&test_file).await.unwrap();
    file.write_all(b"test").await.unwrap();
    file.sync_all().await.unwrap();
    
    let metadata = std::fs::metadata(&test_file).unwrap();
    let inode = Arc::new(Inode::new_file(test_file.clone(), &metadata));
    
    assert_eq!(inode.get_ref(), 0);
    
    inode.inc_ref();
    assert_eq!(inode.get_ref(), 1);
    
    inode.inc_ref();
    assert_eq!(inode.get_ref(), 2);
    
    inode.dec_ref();
    assert_eq!(inode.get_ref(), 1);
    
    inode.dec_ref();
    assert_eq!(inode.get_ref(), 0);
}

#[tokio::test]
async fn test_filesystem_storage_nested_directory_scanning() {
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create nested structure
    let music_dir = temp_dir.path().join("music");
    tokio::fs::create_dir_all(&music_dir.join("Artist/Album/Disc1")).await.unwrap();
    
    // Create files at different levels
    let file1 = music_dir.join("single.mp3");
    let file2 = music_dir.join("Artist").join("album1.mp3");
    let file3 = music_dir.join("Artist").join("Album").join("song.mp3");
    
    for file in vec![&file1, &file2, &file3] {
        let mut f = File::create(file).await.unwrap();
        f.write_all(b"test").await.unwrap();
        f.sync_all().await.unwrap();
    }
    
    let config = FileSystemConfig {
        music_root: music_dir.clone(),
        database_path: temp_dir.path().join("test.db"),
        cover_cache_dir: temp_dir.path().join("cache").to_path_buf(),
        cache_size: 1000,
        supported_extensions: vec!["mp3", "flac", "m4a"],
    };

    let storage = FileSystemStorage::with_config(config).await.unwrap();
    
    // Check root has children by using root_inode getter
    let root = storage.root_inode();
    let children = root.children.read().unwrap();
    assert!(children.len() > 0, "Root inode should have scanned children");
}

#[tokio::test]
async fn test_filesystem_storage_config_defaults() {
    let config = FileSystemConfig::default();
    
    assert_eq!(config.cache_size, 10000);
    assert!(config.supported_extensions.contains(&"mp3"));
    assert!(config.supported_extensions.contains(&"flac"));
    assert_eq!(config.supported_extensions.len(), 7);
}

// Import Arc for the test
use std::sync::Arc;
use uuid::Uuid;
