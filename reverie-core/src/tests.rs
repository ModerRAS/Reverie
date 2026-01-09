//! Unit tests for reverie-core

#[cfg(test)]
mod model_tests {
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
}

#[cfg(test)]
mod media_file_tests {
    use super::super::*;

    #[test]
    fn test_media_file_default() {
        let media_file = MediaFile::default();

        assert_eq!(media_file.id, "");
        assert!(media_file.title.is_empty());
        assert!(!media_file.is_dir);
        assert_eq!(media_file.size, 0);
        assert_eq!(media_file.duration, 0.0);
        assert!(!media_file.missing);
    }

    #[test]
    fn test_media_file_clone() {
        let media_file = MediaFile {
            id: "test-id".to_string(),
            title: "Test Song".to_string(),
            album: Some("Test Album".to_string()),
            artist: Some("Test Artist".to_string()),
            is_dir: false,
            size: 5000000,
            content_type: "audio/mpeg".to_string(),
            suffix: "mp3".to_string(),
            duration: 240.0,
            bit_rate: 320,
            sample_rate: 44100,
            bit_depth: Some(16),
            channels: Some(2),
            path: "/music/test.mp3".to_string(),
            ..Default::default()
        };

        let cloned = media_file.clone();
        assert_eq!(cloned.id, media_file.id);
        assert_eq!(cloned.title, media_file.title);
        assert_eq!(cloned.album, media_file.album);
    }
}

#[cfg(test)]
mod subsonic_models_tests {
    use super::super::*;
    use chrono::Utc;

    #[test]
    fn test_subsonic_album_creation() {
        let album = SubsonicAlbum {
            id: "al-123".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("ar-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("ar-1".to_string()),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            cover_art: Some("ca-123".to_string()),
            song_count: 12,
            duration: 3600.0,
            play_count: Some(100),
            created: Some(Utc::now()),
            starred: None,
            user_rating: Some(5),
        };

        assert_eq!(album.id, "al-123");
        assert_eq!(album.song_count, 12);
        assert_eq!(album.user_rating, Some(5));
    }

    #[test]
    fn test_subsonic_artist_creation() {
        let artist = SubsonicArtist {
            id: "ar-123".to_string(),
            name: "Test Artist".to_string(),
            cover_art: Some("ca-123".to_string()),
            album_count: 10,
            starred: Some(Utc::now()),
            user_rating: Some(4),
        };

        assert_eq!(artist.id, "ar-123");
        assert_eq!(artist.album_count, 10);
    }

    #[test]
    fn test_subsonic_artist_index_creation() {
        let index = SubsonicArtistIndex {
            id: "idx-a".to_string(),
            artists: vec![
                SubsonicArtist {
                    id: "ar-1".to_string(),
                    name: "Artist 1".to_string(),
                    cover_art: None,
                    album_count: 5,
                    starred: None,
                    user_rating: None,
                },
                SubsonicArtist {
                    id: "ar-2".to_string(),
                    name: "Artist 2".to_string(),
                    cover_art: None,
                    album_count: 3,
                    starred: None,
                    user_rating: None,
                },
            ],
        };

        assert_eq!(index.id, "idx-a");
        assert_eq!(index.artists.len(), 2);
    }

    #[test]
    fn test_subsonic_genre_creation() {
        let genre = SubsonicGenre {
            name: "Rock".to_string(),
            song_count: 500,
            album_count: 100,
        };

        assert_eq!(genre.name, "Rock");
        assert_eq!(genre.song_count, 500);
    }

    #[test]
    fn test_subsonic_music_folder_creation() {
        let folder = SubsonicMusicFolder {
            id: 1,
            name: "Music Library".to_string(),
        };

        assert_eq!(folder.id, 1);
        assert_eq!(folder.name, "Music Library");
    }

    #[test]
    fn test_subsonic_scan_status_creation() {
        let status = SubsonicScanStatus {
            scanning: false,
            count: 1000,
            folder_count: 2,
            last_scan: Some(Utc::now()),
            error: None,
            scan_type: Some("incremental".to_string()),
            elapsed_time: Some(120_000),
        };

        assert!(!status.scanning);
        assert_eq!(status.count, 1000);
        assert_eq!(status.folder_count, 2);
    }

    #[test]
    fn test_subsonic_directory_from_album() {
        let album = SubsonicAlbum {
            id: "al-123".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("ar-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("ar-1".to_string()),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            cover_art: Some("ca-123".to_string()),
            song_count: 10,
            duration: 2400.0,
            play_count: None,
            created: Some(Utc::now()),
            starred: None,
            user_rating: None,
        };

        let children = vec![];
        let directory = SubsonicDirectory::from_album(&album, children);

        assert_eq!(directory.id, album.id);
        assert_eq!(directory.name, album.name);
        assert_eq!(directory.cover_art, album.cover_art);
        assert_eq!(directory.child_count, Some(0));
    }

    #[test]
    fn test_subsonic_play_queue_creation() {
        let queue = SubsonicPlayQueue {
            entries: vec![],
            current: Some("song-1".to_string()),
            position: 5,
            username: "testuser".to_string(),
            changed: Utc::now(),
            changed_by: "testuser".to_string(),
        };

        assert_eq!(queue.position, 5);
        assert_eq!(queue.username, "testuser");
        assert!(queue.current.is_some());
    }

    #[test]
    fn test_subsonic_user_creation() {
        let user = SubsonicUser {
            username: "admin".to_string(),
            email: Some("admin@example.com".to_string()),
            scrobbling_enabled: true,
            max_bit_rate: Some(320),
            admin_role: true,
            settings_role: true,
            download_role: true,
            upload_role: false,
            playlist_role: true,
            cover_art_role: true,
            comment_role: true,
            podcast_role: false,
            stream_role: true,
            jukebox_role: false,
            share_role: true,
            video_conversion_role: false,
            avatar_last_changed: None,
            folders: vec![1, 2],
        };

        assert_eq!(user.username, "admin");
        assert!(user.admin_role);
        assert!(user.stream_role);
        assert_eq!(user.folders.len(), 2);
    }

    #[test]
    fn test_subsonic_starred_creation() {
        let starred = SubsonicStarred {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        };

        assert!(starred.artists.is_empty());
        assert!(starred.albums.is_empty());
        assert!(starred.songs.is_empty());
    }

    #[test]
    fn test_subsonic_search_result_creation() {
        let result = SubsonicSearchResult2 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        };

        assert!(result.artists.is_empty());
        assert!(result.albums.is_empty());
        assert!(result.songs.is_empty());
    }

    #[test]
    fn test_subsonic_playlist_creation() {
        let playlist = SubsonicPlaylist {
            id: "pl-123".to_string(),
            name: "My Playlist".to_string(),
            comment: Some("A great playlist".to_string()),
            owner: "testuser".to_string(),
            public: true,
            song_count: 20,
            duration: 3600,
            created: Utc::now(),
            changed: Utc::now(),
            cover_art: Some("ca-123".to_string()),
        };

        assert_eq!(playlist.id, "pl-123");
        assert_eq!(playlist.song_count, 20);
        assert!(playlist.public);
    }

    #[test]
    fn test_subsonic_lyrics_creation() {
        let lyrics = SubsonicLyrics {
            artist: Some("Test Artist".to_string()),
            title: Some("Test Song".to_string()),
            value: "[00:00] Test lyrics".to_string(),
        };

        assert_eq!(lyrics.artist, Some("Test Artist".to_string()));
        assert!(!lyrics.value.is_empty());
    }

    #[test]
    fn test_subsonic_structured_lyrics_creation() {
        let lines = vec![
            SubsonicLyricLine {
                start: Some(0),
                value: "Line 1".to_string(),
            },
            SubsonicLyricLine {
                start: Some(5000),
                value: "Line 2".to_string(),
            },
        ];

        let structured = SubsonicStructuredLyrics {
            display_artist: Some("Artist".to_string()),
            display_title: Some("Title".to_string()),
            lang: "en".to_string(),
            offset: Some(100),
            synced: true,
            lines,
        };

        assert_eq!(structured.lang, "en");
        assert!(structured.synced);
        assert_eq!(structured.lines.len(), 2);
    }

    #[test]
    fn test_subsonic_internet_radio_station_creation() {
        let station = SubsonicInternetRadioStation {
            id: "radio-1".to_string(),
            name: "Rock Radio".to_string(),
            stream_url: "https://example.com/stream".to_string(),
            homepage_url: Some("https://example.com".to_string()),
        };

        assert_eq!(station.name, "Rock Radio");
        assert!(station.homepage_url.is_some());
    }

    #[test]
    fn test_subsonic_share_creation() {
        let share = SubsonicShare {
            id: "share-1".to_string(),
            url: "https://example.com/share/abc".to_string(),
            description: Some("Check this out".to_string()),
            username: "testuser".to_string(),
            created: Utc::now(),
            expires: Some(Utc::now() + chrono::Duration::days(7)),
            last_visited: None,
            visit_count: 10,
            entries: vec![],
        };

        assert_eq!(share.visit_count, 10);
        assert!(share.expires.is_some());
    }

    #[test]
    fn test_subsonic_bookmark_creation() {
        let bookmark = SubsonicBookmark {
            position: 120000,
            username: "testuser".to_string(),
            comment: Some("Resume here".to_string()),
            created: Utc::now(),
            changed: Utc::now(),
            entry: MediaFile::default(),
        };

        assert_eq!(bookmark.position, 120000);
        assert!(bookmark.comment.is_some());
    }

    #[test]
    fn test_subsonic_now_playing_creation() {
        let now_playing = SubsonicNowPlaying {
            entry: MediaFile {
                id: "song-1".to_string(),
                title: "Test Song".to_string(),
                ..Default::default()
            },
            username: "testuser".to_string(),
            minutes_ago: 2,
            player_id: Some("player-1".to_string()),
            player_name: Some("Web Player".to_string()),
        };

        assert_eq!(now_playing.minutes_ago, 2);
        assert!(now_playing.player_name.is_some());
    }

    #[test]
    fn test_subsonic_artist_info_creation() {
        let info = SubsonicArtistInfo {
            biography: Some("A great biography".to_string()),
            music_brainz_id: Some("mbid-123".to_string()),
            last_fm_url: Some("https://last.fm/artist".to_string()),
            small_image_url: Some("https://example.com/small.jpg".to_string()),
            medium_image_url: Some("https://example.com/medium.jpg".to_string()),
            large_image_url: Some("https://example.com/large.jpg".to_string()),
            similar_artists: vec![],
        };

        assert!(info.biography.is_some());
        assert!(info.music_brainz_id.is_some());
    }

    #[test]
    fn test_subsonic_album_info_creation() {
        let info = SubsonicAlbumInfo {
            notes: Some("Great album!".to_string()),
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
        };

        assert_eq!(info.notes, Some("Great album!".to_string()));
    }

    #[test]
    fn test_subsonic_top_songs_creation() {
        let top_songs = SubsonicTopSongs { songs: vec![] };
        assert!(top_songs.songs.is_empty());
    }

    #[test]
    fn test_subsonic_open_subsonic_extension_creation() {
        let ext = SubsonicOpenSubsonicExtension {
            name: "transcodeOffset".to_string(),
            versions: vec![1],
        };

        assert_eq!(ext.name, "transcodeOffset");
        assert_eq!(ext.versions, vec![1]);
    }

    #[test]
    fn test_subsonic_playlist_with_songs_creation() {
        let playlist = SubsonicPlaylistWithSongs {
            id: "pl-1".to_string(),
            name: "My Playlist".to_string(),
            comment: None,
            owner: "testuser".to_string(),
            public: false,
            song_count: 5,
            duration: 600,
            created: Utc::now(),
            changed: Utc::now(),
            cover_art: None,
            entries: vec![],
        };

        assert_eq!(playlist.song_count, 5);
        assert!(!playlist.public);
    }
}

#[cfg(test)]
mod constants_tests {
    use super::super::*;

    #[test]
    fn test_subsonic_api_version() {
        assert_eq!(SUBSONIC_API_VERSION, "1.16.1");
    }
}
