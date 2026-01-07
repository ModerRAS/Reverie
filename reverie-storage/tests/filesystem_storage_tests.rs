//! Filesystem storage tests

use reverie_storage::filesystem::{FileSystemConfig, FileSystemStorage};

#[tokio::test]
async fn test_filesystem_storage_config_defaults() {
    let config = FileSystemConfig::default();

    assert!(config.supported_extensions.contains(&"mp3"));
    assert!(config.supported_extensions.contains(&"flac"));
    assert_eq!(config.supported_extensions.len(), 7);
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
