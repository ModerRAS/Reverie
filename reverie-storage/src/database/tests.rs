//! DatabaseStorage 单元测试
//!
//! 按照 TDD 原则编写测试

#[cfg(test)]
mod bookmark_tests {
    use crate::database::DatabaseStorage;
    use crate::traits::SubsonicStorage;
    use crate::database::config::DatabaseConfig;
    use crate::vfs::VfsConfig;

    async fn create_test_storage() -> DatabaseStorage {
        let config = DatabaseConfig {
            database_url: ":memory:".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::memory(),
        };
        let storage = DatabaseStorage::new(config).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_create_and_get_bookmarks() {
        let storage = create_test_storage().await;

        // 先创建一个测试曲目
        use chrono::Utc;
        use uuid::Uuid;
        use reverie_core::Track;
        use crate::traits::TrackStorage;

        let track = Track {
            id: Uuid::new_v4(),
            title: "Test Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/test.mp3".to_string(),
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

        // 创建书签
        let track_id = track.id.to_string();
        storage.create_bookmark(&track_id, 30000, Some("Test bookmark")).await.unwrap();

        // 获取书签列表
        let bookmarks = storage.get_bookmarks().await.unwrap();
        assert!(!bookmarks.is_empty(), "Should have at least one bookmark");

        let bookmark = &bookmarks[0];
        assert_eq!(bookmark.position, 30000);
        assert_eq!(bookmark.comment.as_deref(), Some("Test bookmark"));
    }

    #[tokio::test]
    async fn test_delete_bookmark() {
        let storage = create_test_storage().await;

        // 创建测试曲目
        use chrono::Utc;
        use uuid::Uuid;
        use reverie_core::Track;
        use crate::traits::TrackStorage;

        let track = Track {
            id: Uuid::new_v4(),
            title: "Test Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/test.mp3".to_string(),
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

        let track_id = track.id.to_string();
        
        // 创建书签
        storage.create_bookmark(&track_id, 30000, None).await.unwrap();
        
        // 验证书签存在
        let bookmarks = storage.get_bookmarks().await.unwrap();
        assert_eq!(bookmarks.len(), 1);
        
        // 删除书签
        storage.delete_bookmark(&track_id).await.unwrap();
        
        // 验证书签已删除
        let bookmarks = storage.get_bookmarks().await.unwrap();
        assert!(bookmarks.is_empty());
    }
}

#[cfg(test)]
mod internet_radio_tests {
    use crate::database::DatabaseStorage;
    use crate::traits::SubsonicStorage;
    use crate::database::config::DatabaseConfig;
    use crate::vfs::VfsConfig;

    async fn create_test_storage() -> DatabaseStorage {
        let config = DatabaseConfig {
            database_url: ":memory:".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::memory(),
        };
        let storage = DatabaseStorage::new(config).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_create_and_get_radio_stations() {
        let storage = create_test_storage().await;

        // 创建电台
        storage.create_internet_radio_station(
            "http://stream.example.com/radio",
            "Test Radio",
            Some("http://example.com"),
        ).await.unwrap();

        // 获取电台列表
        let stations = storage.get_internet_radio_stations().await.unwrap();
        assert!(!stations.is_empty(), "Should have at least one station");

        let station = &stations[0];
        assert_eq!(station.name, "Test Radio");
        assert_eq!(station.stream_url, "http://stream.example.com/radio");
        assert_eq!(station.homepage_url.as_deref(), Some("http://example.com"));
    }

    #[tokio::test]
    async fn test_update_radio_station() {
        let storage = create_test_storage().await;

        // 创建电台
        storage.create_internet_radio_station(
            "http://stream.example.com/radio",
            "Original Name",
            None,
        ).await.unwrap();

        // 获取电台 ID
        let stations = storage.get_internet_radio_stations().await.unwrap();
        let station_id = stations[0].id.clone();

        // 更新电台
        storage.update_internet_radio_station(
            &station_id,
            "http://new.stream.com/radio",
            "Updated Name",
            Some("http://homepage.com"),
        ).await.unwrap();

        // 验证更新
        let stations = storage.get_internet_radio_stations().await.unwrap();
        let station = &stations[0];
        assert_eq!(station.name, "Updated Name");
        assert_eq!(station.stream_url, "http://new.stream.com/radio");
        assert_eq!(station.homepage_url.as_deref(), Some("http://homepage.com"));
    }

    #[tokio::test]
    async fn test_delete_radio_station() {
        let storage = create_test_storage().await;

        // 创建电台
        storage.create_internet_radio_station(
            "http://stream.example.com/radio",
            "Test Radio",
            None,
        ).await.unwrap();

        // 获取电台 ID
        let stations = storage.get_internet_radio_stations().await.unwrap();
        let station_id = stations[0].id.clone();

        // 删除电台
        storage.delete_internet_radio_station(&station_id).await.unwrap();

        // 验证删除
        let stations = storage.get_internet_radio_stations().await.unwrap();
        assert!(stations.is_empty());
    }
}

#[cfg(test)]
mod star_rating_tests {
    use crate::database::DatabaseStorage;
    use crate::traits::{SubsonicStorage, TrackStorage};
    use crate::database::config::DatabaseConfig;
    use crate::vfs::VfsConfig;
    use chrono::Utc;
    use uuid::Uuid;
    use reverie_core::Track;

    async fn create_test_storage() -> DatabaseStorage {
        let config = DatabaseConfig {
            database_url: ":memory:".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::memory(),
        };
        let storage = DatabaseStorage::new(config).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_star_and_unstar_track() {
        let storage = create_test_storage().await;

        // 创建测试曲目
        let track = Track {
            id: Uuid::new_v4(),
            title: "Starrable Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/star.mp3".to_string(),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Pop".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await.unwrap();

        let track_id = track.id.to_string();

        // 收藏曲目
        storage.star(&[track_id.as_str()], &[], &[]).await.unwrap();

        // 验证收藏状态 (通过 get_starred)
        let starred = storage.get_starred(None).await.unwrap();
        assert!(!starred.songs.is_empty(), "Should have starred songs");

        // 取消收藏
        storage.unstar(&[track_id.as_str()], &[], &[]).await.unwrap();

        // 验证取消收藏
        let starred = storage.get_starred(None).await.unwrap();
        assert!(starred.songs.is_empty(), "Should have no starred songs");
    }

    #[tokio::test]
    async fn test_set_rating() {
        let storage = create_test_storage().await;

        // 创建测试曲目
        let track = Track {
            id: Uuid::new_v4(),
            title: "Ratable Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/rate.mp3".to_string(),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Jazz".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track).await.unwrap();

        let track_id = track.id.to_string();

        // 设置评分
        storage.set_rating(&track_id, 5).await.unwrap();

        // 验证评分 (通过 get_song)
        let song = storage.get_song(&track_id).await.unwrap().unwrap();
        assert_eq!(song.user_rating, Some(5));
    }
}

#[cfg(test)]
mod play_queue_tests {
    use crate::database::DatabaseStorage;
    use crate::traits::{SubsonicStorage, TrackStorage};
    use crate::database::config::DatabaseConfig;
    use crate::vfs::VfsConfig;
    use chrono::Utc;
    use uuid::Uuid;
    use reverie_core::Track;

    async fn create_test_storage() -> DatabaseStorage {
        let config = DatabaseConfig {
            database_url: ":memory:".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::memory(),
        };
        let storage = DatabaseStorage::new(config).await.unwrap();
        storage.initialize().await.unwrap();
        storage
    }

    #[tokio::test]
    async fn test_save_and_get_play_queue() {
        let storage = create_test_storage().await;

        // 创建测试曲目
        let track1 = Track {
            id: Uuid::new_v4(),
            title: "Queue Track 1".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/queue1.mp3".to_string(),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Electronic".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let track2 = Track {
            id: Uuid::new_v4(),
            title: "Queue Track 2".to_string(),
            album_id: None,
            artist_id: None,
            duration: 200,
            file_path: "/music/queue2.mp3".to_string(),
            file_size: 6000000,
            bitrate: 256,
            format: "mp3".to_string(),
            track_number: Some(2),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Electronic".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        storage.save_track(&track1).await.unwrap();
        storage.save_track(&track2).await.unwrap();

        let track_id1 = track1.id.to_string();
        let track_id2 = track2.id.to_string();

        // 保存播放队列
        storage.save_play_queue(
            &[track_id1.as_str(), track_id2.as_str()],
            Some(track_id1.as_str()),
            Some(60000),
        ).await.unwrap();

        // 获取播放队列
        let queue = storage.get_play_queue().await.unwrap();
        assert!(queue.is_some(), "Should have a play queue");

        let queue = queue.unwrap();
        assert_eq!(queue.entries.len(), 2);
        assert_eq!(queue.current, Some(track_id1));
        assert_eq!(queue.position, 60000);
    }
}
