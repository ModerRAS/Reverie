//! Unit tests for reverie-storage

#[cfg(test)]
mod error_tests {
    use crate::StorageError;
    use std::io;

    #[test]
    fn test_storage_error_not_found() {
        let error = StorageError::NotFound("test/path".to_string());
        assert_eq!(format!("{}", error), "File not found: test/path");
    }

    #[test]
    fn test_storage_error_io() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error = StorageError::IoError(io_error);
        assert_eq!(format!("{}", error), "IO error: file not found");
    }

    #[test]
    fn test_storage_error_database() {
        let error = StorageError::DatabaseError("Connection refused".to_string());
        assert_eq!(format!("{}", error), "Database error: Connection refused");
    }

    #[test]
    fn test_storage_error_unavailable() {
        let error = StorageError::Unavailable("Service unavailable".to_string());
        assert_eq!(
            format!("{}", error),
            "Storage unavailable: Service unavailable"
        );
    }

    #[test]
    fn test_storage_error_invalid_path() {
        let error = StorageError::InvalidPath("/invalid".to_string());
        assert_eq!(format!("{}", error), "Invalid path: /invalid");
    }

    #[test]
    fn test_storage_error_permission_denied() {
        let error = StorageError::PermissionDenied("Access denied".to_string());
        assert_eq!(format!("{}", error), "Permission denied: Access denied");
    }

    #[test]
    fn test_storage_error_from_io() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");
        let error: StorageError = io_error.into();
        // io::Error converts to IoError, not PermissionDenied
        assert!(format!("{}", error).contains("IO error"));
    }

    #[test]
    fn test_storage_error_serialization() {
        // Use invalid JSON to trigger serialization error
        let json_error: serde_json::Error =
            serde_json::from_str::<String>("invalid json").unwrap_err();
        let error: StorageError = json_error.into();
        assert!(format!("{}", error).contains("Serialization error"));
    }
}

#[cfg(test)]
mod traits_tests {
    use crate::FileMetadata;
    use std::time::SystemTime;

    #[test]
    fn test_file_metadata_creation() {
        let now = SystemTime::now();
        let metadata = FileMetadata {
            size: 1024,
            modified: now,
            is_file: true,
            is_dir: false,
        };

        assert_eq!(metadata.size, 1024);
        assert!(metadata.is_file);
        assert!(!metadata.is_dir);
    }

    #[test]
    fn test_file_metadata_clone() {
        let now = SystemTime::now();
        let metadata = FileMetadata {
            size: 2048,
            modified: now,
            is_file: false,
            is_dir: true,
        };

        let cloned = metadata.clone();
        assert_eq!(cloned.size, metadata.size);
        assert_eq!(cloned.is_file, metadata.is_file);
        assert_eq!(cloned.is_dir, metadata.is_dir);
    }

    #[test]
    fn test_file_metadata_debug() {
        let now = SystemTime::UNIX_EPOCH;
        let metadata = FileMetadata {
            size: 512,
            modified: now,
            is_file: true,
            is_dir: false,
        };

        let debug_str = format!("{:?}", metadata);
        assert!(debug_str.contains("FileMetadata"));
        assert!(debug_str.contains("size: 512"));
    }
}

#[cfg(test)]
mod subsonic_storage_tests {
    use crate::memory::MemoryStorage;
    use crate::SubsonicStorage;

    #[tokio::test]
    async fn test_get_license_default() {
        let storage = MemoryStorage::new();
        let license = storage.get_license().await.unwrap();
        assert!(license);
    }

    #[tokio::test]
    async fn test_get_open_subsonic_extensions_default() {
        let storage = MemoryStorage::new();
        let extensions = storage.get_open_subsonic_extensions().await.unwrap();

        assert!(!extensions.is_empty());
        assert!(extensions.iter().any(|e| e.name == "transcodeOffset"));
        assert!(extensions.iter().any(|e| e.name == "songLyrics"));
    }

    #[tokio::test]
    async fn test_get_videos_default() {
        let storage = MemoryStorage::new();
        let videos = storage.get_videos().await.unwrap();
        assert!(videos.is_empty());
    }

    #[tokio::test]
    async fn test_search_default_implementation() {
        let storage = MemoryStorage::new();
        let result = storage
            .search(None, Some("album"), Some("title"), None, None, None, None)
            .await
            .unwrap();

        assert!(result.artists.is_empty());
        assert!(result.albums.is_empty());
        assert!(result.songs.is_empty());
    }
}
