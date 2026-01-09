//! Integration tests for Subsonic API endpoints
//!
//! These tests verify that the Subsonic API endpoints return correctly
//! formatted responses according to the Subsonic API 1.16.1 specification.

#[cfg(test)]
mod subsonic_api_tests {
    use crate::subsonic::{create_router, SubsonicState};
    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use reverie_core::*;
    use reverie_storage::{error::StorageError, SubsonicStorage};
    use serde_json::Value;
    use std::sync::Arc;
    use tower::ServiceExt;

    // === Mock Storage Implementation ===

    #[derive(Clone)]
    struct MockSubsonicStorage {
        // Add fields for custom test data if needed
    }

    impl MockSubsonicStorage {
        fn new() -> Self {
            Self {}
        }
    }

    #[async_trait]
    impl SubsonicStorage for MockSubsonicStorage {
        async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>, StorageError> {
            Ok(vec![
                SubsonicMusicFolder {
                    id: 1,
                    name: "Music".to_string(),
                },
                SubsonicMusicFolder {
                    id: 2,
                    name: "Podcasts".to_string(),
                },
            ])
        }

        async fn get_indexes(
            &self,
            _music_folder_id: Option<i32>,
            _if_modified_since: Option<i64>,
        ) -> Result<SubsonicArtistIndexes, StorageError> {
            Ok(vec![])
        }

        async fn get_genres(&self) -> Result<Vec<SubsonicGenre>, StorageError> {
            Ok(vec![
                SubsonicGenre {
                    name: "Rock".to_string(),
                    song_count: 100,
                    album_count: 10,
                },
            ])
        }

        async fn get_music_directory(
            &self,
            _id: &str,
        ) -> Result<Option<SubsonicDirectory>, StorageError> {
            Ok(None)
        }

        async fn get_artists(
            &self,
            _music_folder_id: Option<i32>,
        ) -> Result<SubsonicArtistIndexes, StorageError> {
            Ok(vec![SubsonicArtistIndex {
                id: "A".to_string(),
                artists: vec![SubsonicArtist {
                    id: "ar-1".to_string(),
                    name: "Artist One".to_string(),
                    cover_art: Some("ar-1".to_string()),
                    album_count: 3,
                    starred: None,
                    user_rating: None,
                }],
            }])
        }

        async fn get_artist(
            &self,
            id: &str,
        ) -> Result<Option<SubsonicArtist>, StorageError> {
            if id == "ar-1" {
                Ok(Some(SubsonicArtist {
                    id: "ar-1".to_string(),
                    name: "Artist One".to_string(),
                    cover_art: Some("ar-1".to_string()),
                    album_count: 3,
                    starred: None,
                    user_rating: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn get_album(&self, id: &str) -> Result<Option<SubsonicAlbum>, StorageError> {
            if id == "al-1" {
                Ok(Some(SubsonicAlbum {
                    id: "al-1".to_string(),
                    name: "Album One".to_string(),
                    album_artist: Some("Artist One".to_string()),
                    album_artist_id: Some("ar-1".to_string()),
                    artist: Some("Artist One".to_string()),
                    artist_id: Some("ar-1".to_string()),
                    year: Some(2024),
                    genre: Some("Rock".to_string()),
                    cover_art: Some("al-1".to_string()),
                    song_count: 10,
                    duration: 3600.0,
                    play_count: Some(100),
                    created: None,
                    starred: None,
                    user_rating: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn get_song(&self, id: &str) -> Result<Option<MediaFile>, StorageError> {
            if id == "song-1" {
                Ok(Some(MediaFile {
                    id: "song-1".to_string(),
                    path: "/music/song1.mp3".to_string(),
                    title: "Song One".to_string(),
                    album: Some("Album One".to_string()),
                    artist: Some("Artist One".to_string()),
                    album_artist: None,
                    track_number: Some(1),
                    disc_number: Some(1),
                    year: Some(2024),
                    genre: Some("Rock".to_string()),
                    duration: Some(180.0),
                    bit_rate: Some(320),
                    size: Some(5_000_000),
                    suffix: Some("mp3".to_string()),
                    content_type: Some("audio/mpeg".to_string()),
                    cover_art: Some("song-1".to_string()),
                    album_id: Some("al-1".to_string()),
                    artist_id: Some("ar-1".to_string()),
                    created: None,
                    starred: None,
                    play_count: None,
                    user_rating: None,
                    bpm: None,
                    comment: None,
                    sort_name: None,
                    media_type: Some("music".to_string()),
                    channels: None,
                    sample_rate: None,
                    bit_depth: None,
                    replay_gain: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn get_artist_info(
            &self,
            _id: &str,
            _count: Option<i32>,
            _include_not_present: Option<bool>,
        ) -> Result<SubsonicArtistInfo, StorageError> {
            Ok(SubsonicArtistInfo::default())
        }

        async fn get_artist_info2(
            &self,
            _id: &str,
            _count: Option<i32>,
            _include_not_present: Option<bool>,
        ) -> Result<SubsonicArtistInfo, StorageError> {
            Ok(SubsonicArtistInfo::default())
        }

        async fn get_album_info(&self, _id: &str) -> Result<SubsonicAlbumInfo, StorageError> {
            Ok(SubsonicAlbumInfo::default())
        }

        async fn get_album_info2(&self, _id: &str) -> Result<SubsonicAlbumInfo, StorageError> {
            Ok(SubsonicAlbumInfo::default())
        }

        async fn get_similar_songs(
            &self,
            _id: &str,
            _count: Option<i32>,
        ) -> Result<Vec<MediaFile>, StorageError> {
            Ok(vec![])
        }

        async fn get_similar_songs2(
            &self,
            _id: &str,
            _count: Option<i32>,
        ) -> Result<Vec<MediaFile>, StorageError> {
            Ok(vec![])
        }

        async fn get_top_songs(
            &self,
            _artist: &str,
            _count: Option<i32>,
        ) -> Result<SubsonicTopSongs, StorageError> {
            Ok(SubsonicTopSongs::default())
        }

        async fn get_album_list(
            &self,
            _type_: &str,
            _size: Option<i32>,
            _offset: Option<i32>,
            _from_year: Option<i32>,
            _to_year: Option<i32>,
            _genre: Option<&str>,
            _music_folder_id: Option<i32>,
        ) -> Result<Vec<SubsonicAlbum>, StorageError> {
            Ok(vec![])
        }

        async fn get_album_list2(
            &self,
            type_: &str,
            _size: Option<i32>,
            _offset: Option<i32>,
            _from_year: Option<i32>,
            _to_year: Option<i32>,
            _genre: Option<&str>,
            _music_folder_id: Option<i32>,
        ) -> Result<Vec<SubsonicAlbum>, StorageError> {
            if type_ == "recent" {
                Ok(vec![SubsonicAlbum {
                    id: "al-1".to_string(),
                    name: "Album One".to_string(),
                    album_artist: Some("Artist One".to_string()),
                    album_artist_id: Some("ar-1".to_string()),
                    artist: Some("Artist One".to_string()),
                    artist_id: Some("ar-1".to_string()),
                    year: Some(2024),
                    genre: Some("Rock".to_string()),
                    cover_art: Some("al-1".to_string()),
                    song_count: 10,
                    duration: 3600.0,
                    play_count: Some(100),
                    created: None,
                    starred: None,
                    user_rating: None,
                }])
            } else {
                Ok(vec![])
            }
        }

        async fn get_random_songs(
            &self,
            _size: Option<i32>,
            _genre: Option<&str>,
            _from_year: Option<i32>,
            _to_year: Option<i32>,
            _music_folder_id: Option<i32>,
        ) -> Result<Vec<MediaFile>, StorageError> {
            Ok(vec![])
        }

        async fn get_songs_by_genre(
            &self,
            _genre: &str,
            _count: Option<i32>,
            _offset: Option<i32>,
            _music_folder_id: Option<i32>,
        ) -> Result<Vec<MediaFile>, StorageError> {
            Ok(vec![])
        }

        async fn get_now_playing(&self) -> Result<Vec<SubsonicNowPlaying>, StorageError> {
            Ok(vec![])
        }

        async fn get_starred(
            &self,
            _music_folder_id: Option<i32>,
        ) -> Result<SubsonicStarred, StorageError> {
            Ok(SubsonicStarred::default())
        }

        async fn get_starred2(
            &self,
            _music_folder_id: Option<i32>,
        ) -> Result<SubsonicStarred, StorageError> {
            Ok(SubsonicStarred::default())
        }

        async fn search2(
            &self,
            _query: &str,
            _artist_count: Option<i32>,
            _artist_offset: Option<i32>,
            _album_count: Option<i32>,
            _album_offset: Option<i32>,
            _song_count: Option<i32>,
            _song_offset: Option<i32>,
        ) -> Result<SubsonicSearchResult2, StorageError> {
            Ok(SubsonicSearchResult2::default())
        }

        async fn search3(
            &self,
            query: &str,
            _artist_count: Option<i32>,
            _artist_offset: Option<i32>,
            _album_count: Option<i32>,
            _album_offset: Option<i32>,
            _song_count: Option<i32>,
            _song_offset: Option<i32>,
        ) -> Result<SubsonicSearchResult3, StorageError> {
            if query == "Artist" {
                Ok(SubsonicSearchResult3 {
                    artists: vec![SubsonicArtist {
                        id: "ar-1".to_string(),
                        name: "Artist One".to_string(),
                        cover_art: Some("ar-1".to_string()),
                        album_count: 3,
                        starred: None,
                        user_rating: None,
                    }],
                    albums: vec![],
                    songs: vec![],
                })
            } else {
                Ok(SubsonicSearchResult3::default())
            }
        }

        async fn get_playlists(
            &self,
            _username: Option<&str>,
        ) -> Result<Vec<SubsonicPlaylist>, StorageError> {
            Ok(vec![])
        }

        async fn get_playlist(
            &self,
            _id: &str,
        ) -> Result<Option<SubsonicPlaylistWithSongs>, StorageError> {
            Ok(None)
        }

        async fn create_playlist(
            &self,
            _playlist_id: Option<&str>,
            _name: Option<&str>,
            _song_ids: &[&str],
        ) -> Result<SubsonicPlaylistWithSongs, StorageError> {
            Ok(SubsonicPlaylistWithSongs::default())
        }

        async fn update_playlist(
            &self,
            _playlist_id: &str,
            _name: Option<&str>,
            _comment: Option<&str>,
            _public: Option<bool>,
            _song_ids_to_add: &[&str],
            _song_indexes_to_remove: &[i32],
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn delete_playlist(&self, _id: &str) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_stream_path(&self, id: &str) -> Result<Option<String>, StorageError> {
            if id == "song-1" {
                Ok(Some("/music/song1.mp3".to_string()))
            } else {
                Ok(None)
            }
        }

        async fn get_cover_art_path(&self, id: &str) -> Result<Option<String>, StorageError> {
            if id.starts_with("al-") || id.starts_with("ar-") {
                Ok(Some(format!("/covers/{}.jpg", id)))
            } else {
                Ok(None)
            }
        }

        async fn get_lyrics(
            &self,
            _artist: Option<&str>,
            _title: Option<&str>,
        ) -> Result<Option<SubsonicLyrics>, StorageError> {
            Ok(None)
        }

        async fn get_lyrics_by_song_id(
            &self,
            _id: &str,
        ) -> Result<Vec<SubsonicStructuredLyrics>, StorageError> {
            Ok(vec![])
        }

        async fn get_avatar_path(&self, _username: &str) -> Result<Option<String>, StorageError> {
            Ok(None)
        }

        async fn star(
            &self,
            _ids: &[&str],
            _album_ids: &[&str],
            _artist_ids: &[&str],
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn unstar(
            &self,
            _ids: &[&str],
            _album_ids: &[&str],
            _artist_ids: &[&str],
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn set_rating(&self, _id: &str, _rating: i32) -> Result<(), StorageError> {
            Ok(())
        }

        async fn scrobble(
            &self,
            _id: &str,
            _time: Option<i64>,
            _submission: bool,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_bookmarks(&self) -> Result<Vec<SubsonicBookmark>, StorageError> {
            Ok(vec![])
        }

        async fn create_bookmark(
            &self,
            _id: &str,
            _position: i64,
            _comment: Option<&str>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn delete_bookmark(&self, _id: &str) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>, StorageError> {
            Ok(None)
        }

        async fn save_play_queue(
            &self,
            _ids: &[&str],
            _current: Option<&str>,
            _position: Option<i64>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_shares(&self) -> Result<Vec<SubsonicShare>, StorageError> {
            Ok(vec![])
        }

        async fn create_share(
            &self,
            _ids: &[&str],
            _description: Option<&str>,
            _expires: Option<i64>,
        ) -> Result<SubsonicShare, StorageError> {
            Ok(SubsonicShare::default())
        }

        async fn update_share(
            &self,
            _id: &str,
            _description: Option<&str>,
            _expires: Option<i64>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn delete_share(&self, _id: &str) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_internet_radio_stations(
            &self,
        ) -> Result<Vec<SubsonicInternetRadioStation>, StorageError> {
            Ok(vec![])
        }

        async fn create_internet_radio_station(
            &self,
            _stream_url: &str,
            _name: &str,
            _homepage_url: Option<&str>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn update_internet_radio_station(
            &self,
            _id: &str,
            _stream_url: &str,
            _name: &str,
            _homepage_url: Option<&str>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn delete_internet_radio_station(&self, _id: &str) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>, StorageError> {
            if username == "admin" {
                Ok(Some(SubsonicUser {
                    username: "admin".to_string(),
                    email: Some("admin@reverie.local".to_string()),
                    scrobbling_enabled: true,
                    max_bit_rate: None,
                    admin_role: true,
                    settings_role: true,
                    download_role: true,
                    upload_role: true,
                    playlist_role: true,
                    cover_art_role: true,
                    comment_role: true,
                    podcast_role: true,
                    stream_role: true,
                    jukebox_role: false,
                    share_role: true,
                    video_conversion_role: false,
                    avatar_last_changed: None,
                    folders: vec![1],
                }))
            } else {
                Ok(None)
            }
        }

        async fn get_users(&self) -> Result<Vec<SubsonicUser>, StorageError> {
            Ok(vec![])
        }

        async fn create_user(
            &self,
            _username: &str,
            _password: &str,
            _email: Option<&str>,
            _admin_role: bool,
            _settings_role: bool,
            _stream_role: bool,
            _jukebox_role: bool,
            _download_role: bool,
            _upload_role: bool,
            _playlist_role: bool,
            _cover_art_role: bool,
            _comment_role: bool,
            _podcast_role: bool,
            _share_role: bool,
            _video_conversion_role: bool,
            _music_folder_ids: &[i32],
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn update_user(
            &self,
            _username: &str,
            _password: Option<&str>,
            _email: Option<&str>,
            _admin_role: Option<bool>,
            _settings_role: Option<bool>,
            _stream_role: Option<bool>,
            _jukebox_role: Option<bool>,
            _download_role: Option<bool>,
            _upload_role: Option<bool>,
            _playlist_role: Option<bool>,
            _cover_art_role: Option<bool>,
            _comment_role: Option<bool>,
            _podcast_role: Option<bool>,
            _share_role: Option<bool>,
            _max_bit_rate: Option<i32>,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn delete_user(&self, _username: &str) -> Result<(), StorageError> {
            Ok(())
        }

        async fn change_password(
            &self,
            _username: &str,
            _password: &str,
        ) -> Result<(), StorageError> {
            Ok(())
        }

        async fn get_scan_status(&self) -> Result<SubsonicScanStatus, StorageError> {
            Ok(SubsonicScanStatus::default())
        }

        async fn start_scan(&self) -> Result<SubsonicScanStatus, StorageError> {
            Ok(SubsonicScanStatus::default())
        }
    }

    // === Test Helpers ===

    fn create_test_router() -> axum::Router {
        let storage = Arc::new(MockSubsonicStorage::new());
        let state = SubsonicState::new(storage);
        create_router::<MockSubsonicStorage>().with_state(state)
    }

    async fn get_json_response(router: axum::Router, uri: &str) -> (StatusCode, Value) {
        let response = router
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);

        (status, json)
    }

    fn assert_subsonic_ok(json: &Value) {
        let resp = &json["subsonic-response"];
        assert_eq!(resp["status"], "ok", "Expected status 'ok', got: {}", resp);
        assert_eq!(resp["version"], "1.16.1", "Expected version '1.16.1'");
        assert_eq!(resp["type"], "reverie", "Expected type 'reverie'");
        assert!(resp["openSubsonic"].as_bool().unwrap_or(false), "Expected openSubsonic true");
    }

    fn assert_subsonic_error(json: &Value, expected_code: i32) {
        let resp = &json["subsonic-response"];
        assert_eq!(resp["status"], "failed", "Expected status 'failed'");
        assert_eq!(
            resp["error"]["code"].as_i64().unwrap_or(-1) as i32,
            expected_code,
            "Expected error code {}, got: {}",
            expected_code,
            resp["error"]["code"]
        );
    }

    // === System Endpoint Tests ===

    #[tokio::test]
    async fn test_ping_returns_ok() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/ping?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);
    }

    #[tokio::test]
    async fn test_get_license_returns_valid_license() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getLicense?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let license = &json["subsonic-response"]["license"];
        assert!(license["valid"].as_bool().unwrap_or(false), "License should be valid");
    }

    #[tokio::test]
    async fn test_get_music_folders_returns_folders() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getMusicFolders?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let folders = &json["subsonic-response"]["musicFolders"]["musicFolder"];
        assert!(folders.is_array(), "musicFolder should be an array");
        assert_eq!(folders.as_array().unwrap().len(), 2, "Should have 2 folders");
        assert_eq!(folders[0]["name"], "Music");
        assert_eq!(folders[1]["name"], "Podcasts");
    }

    // === Browsing Endpoint Tests ===

    #[tokio::test]
    async fn test_get_artists_returns_artist_index() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getArtists?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let artists = &json["subsonic-response"]["artists"];
        assert!(artists.is_object(), "artists should be an object");
        // Check that ignoredArticles is present (per Subsonic spec)
        assert!(artists["ignoredArticles"].is_string(), "ignoredArticles should be present");
    }

    #[tokio::test]
    async fn test_get_artist_with_valid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getArtist?f=json&id=ar-1").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let artist = &json["subsonic-response"]["artist"];
        assert_eq!(artist["id"], "ar-1");
        assert_eq!(artist["name"], "Artist One");
    }

    #[tokio::test]
    async fn test_get_artist_with_invalid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getArtist?f=json&id=nonexistent").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 70); // Error code 70 = data not found
    }

    #[tokio::test]
    async fn test_get_artist_without_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getArtist?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 10); // Error code 10 = missing parameter
    }

    #[tokio::test]
    async fn test_get_album_with_valid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getAlbum?f=json&id=al-1").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let album = &json["subsonic-response"]["album"];
        assert_eq!(album["id"], "al-1");
        assert_eq!(album["name"], "Album One");
        assert_eq!(album["artist"], "Artist One");
    }

    #[tokio::test]
    async fn test_get_album_with_invalid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getAlbum?f=json&id=nonexistent").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 70);
    }

    #[tokio::test]
    async fn test_get_song_with_valid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getSong?f=json&id=song-1").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let song = &json["subsonic-response"]["song"];
        assert_eq!(song["id"], "song-1");
        assert_eq!(song["title"], "Song One");
    }

    #[tokio::test]
    async fn test_get_song_with_invalid_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getSong?f=json&id=nonexistent").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 70);
    }

    // === Album List Tests ===

    #[tokio::test]
    async fn test_get_album_list2_recent() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getAlbumList2?f=json&type=recent").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let album_list = &json["subsonic-response"]["albumList2"];
        assert!(album_list.is_object(), "albumList2 should be an object");
    }

    #[tokio::test]
    async fn test_get_album_list2_without_type() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getAlbumList2?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 10); // Missing required parameter
    }

    // === Search Tests ===

    #[tokio::test]
    async fn test_search3_with_query() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/search3?f=json&query=Artist").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_ok(&json);

        let result = &json["subsonic-response"]["searchResult3"];
        assert!(result.is_object(), "searchResult3 should be an object");
    }

    #[tokio::test]
    async fn test_search3_without_query() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/search3?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 10); // Missing required parameter
    }

    // === Media Retrieval Tests ===

    #[tokio::test]
    async fn test_get_cover_art_with_valid_id() {
        let router = create_test_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/getCoverArt?id=al-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        // Cover art returns image data, not JSON
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""));
        assert_eq!(content_type, Some("image/jpeg"));
    }

    #[tokio::test]
    async fn test_get_cover_art_without_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/getCoverArt?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 10); // Missing required parameter
    }

    #[tokio::test]
    async fn test_stream_with_valid_id() {
        let router = create_test_router();
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/stream?id=song-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .map(|v| v.to_str().unwrap_or(""));
        assert_eq!(content_type, Some("audio/mpeg"));
    }

    #[tokio::test]
    async fn test_stream_without_id() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/stream?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert_subsonic_error(&json, 10); // Missing required parameter
    }

    // === Response Format Tests ===

    #[tokio::test]
    async fn test_json_format_parameter() {
        let router = create_test_router();
        let (status, json) = get_json_response(router, "/ping?f=json").await;

        assert_eq!(status, StatusCode::OK);
        assert!(json["subsonic-response"].is_object(), "Response should be JSON");
    }

    #[tokio::test]
    async fn test_response_has_required_fields() {
        let router = create_test_router();
        let (_, json) = get_json_response(router, "/ping?f=json").await;

        let resp = &json["subsonic-response"];
        assert!(resp["status"].is_string(), "status field required");
        assert!(resp["version"].is_string(), "version field required");
        assert!(resp["type"].is_string(), "type field required (OpenSubsonic)");
        assert!(resp["serverVersion"].is_string(), "serverVersion field required (OpenSubsonic)");
        assert!(resp["openSubsonic"].is_boolean(), "openSubsonic field required");
    }

    // === Stub Handler Tests (verify they don't crash) ===

    #[tokio::test]
    async fn test_stub_endpoints_return_ok() {
        let router = create_test_router();
        
        let stub_endpoints = [
            "/getIndexes?f=json",
            "/getMusicDirectory?f=json&id=1",
            "/getGenres?f=json",
            "/getPlaylists?f=json",
            "/getStarred?f=json",
            "/getStarred2?f=json",
        ];

        for endpoint in stub_endpoints {
            let (status, json) = get_json_response(router.clone(), endpoint).await;
            assert_eq!(
                status,
                StatusCode::OK,
                "Endpoint {} should return OK",
                endpoint
            );
            assert_subsonic_ok(&json);
        }
    }
}
