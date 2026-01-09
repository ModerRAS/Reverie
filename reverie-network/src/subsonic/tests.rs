//! Subsonic API integration tests

use super::*;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use reverie_storage::{error::StorageError, SubsonicStorage};
use std::sync::Arc;
use tower::ServiceExt;

type Result<T> = std::result::Result<T, StorageError>;

/// Mock storage for testing
#[derive(Clone)]
struct MockSubsonicStorage;

impl MockSubsonicStorage {
    fn new() -> Self {
        MockSubsonicStorage
    }
}

#[async_trait::async_trait]
impl SubsonicStorage for MockSubsonicStorage {
    // === System ===
    async fn get_license(&self) -> Result<bool> {
        Ok(true)
    }

    // === Browsing ===
    async fn get_music_folders(&self) -> Result<Vec<reverie_core::SubsonicMusicFolder>> {
        Ok(vec![reverie_core::SubsonicMusicFolder {
            id: 1,
            name: "Music".to_string(),
        }])
    }

    async fn get_indexes(
        &self,
        _music_folder_id: Option<i32>,
        _if_modified_since: Option<i64>,
    ) -> Result<reverie_core::SubsonicArtistIndexes> {
        Ok(vec![])
    }

    async fn get_genres(&self) -> Result<Vec<reverie_core::SubsonicGenre>> {
        Ok(vec![])
    }

    async fn get_music_directory(&self, _id: &str) -> Result<Option<reverie_core::SubsonicDirectory>> {
        Ok(None)
    }

    async fn get_artists(&self, _music_folder_id: Option<i32>) -> Result<reverie_core::SubsonicArtistIndexes> {
        Ok(vec![reverie_core::SubsonicArtistIndex {
            id: "A".to_string(),
            artists: vec![reverie_core::SubsonicArtist {
                id: "artist-1".to_string(),
                name: "Test Artist".to_string(),
                cover_art: None,
                album_count: 1,
                starred: None,
                user_rating: None,
            }],
        }])
    }

    async fn get_artist(&self, _id: &str) -> Result<Option<reverie_core::SubsonicArtist>> {
        Ok(Some(reverie_core::SubsonicArtist {
            id: "artist-1".to_string(),
            name: "Test Artist".to_string(),
            cover_art: None,
            album_count: 1,
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_album(&self, _id: &str) -> Result<Option<reverie_core::SubsonicAlbum>> {
        Ok(Some(reverie_core::SubsonicAlbum {
            id: "album-1".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("artist-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            cover_art: None,
            song_count: 10,
            duration: 3600.0,
            play_count: None,
            created: None,
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_song(&self, _id: &str) -> Result<Option<reverie_core::MediaFile>> {
        Ok(Some(reverie_core::MediaFile::default()))
    }

    async fn get_artist_info(
        &self,
        _id: &str,
        _count: Option<i32>,
        _include_not_present: Option<bool>,
    ) -> Result<reverie_core::SubsonicArtistInfo> {
        Ok(reverie_core::SubsonicArtistInfo {
            biography: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
            similar_artists: vec![],
        })
    }

    async fn get_artist_info2(
        &self,
        _id: &str,
        _count: Option<i32>,
        _include_not_present: Option<bool>,
    ) -> Result<reverie_core::SubsonicArtistInfo> {
        self.get_artist_info(_id, _count, _include_not_present).await
    }

    async fn get_album_info(&self, _id: &str) -> Result<reverie_core::SubsonicAlbumInfo> {
        Ok(reverie_core::SubsonicAlbumInfo {
            notes: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
        })
    }

    async fn get_album_info2(&self, _id: &str) -> Result<reverie_core::SubsonicAlbumInfo> {
        self.get_album_info(_id).await
    }

    async fn get_similar_songs(&self, _id: &str, _count: Option<i32>) -> Result<Vec<reverie_core::MediaFile>> {
        Ok(vec![])
    }

    async fn get_similar_songs2(&self, _id: &str, _count: Option<i32>) -> Result<Vec<reverie_core::MediaFile>> {
        Ok(vec![])
    }

    async fn get_top_songs(&self, _artist: &str, _count: Option<i32>) -> Result<reverie_core::SubsonicTopSongs> {
        Ok(reverie_core::SubsonicTopSongs { songs: vec![] })
    }

    async fn get_album_list(
        &self,
        _list_type: &str,
        _size: Option<i32>,
        _offset: Option<i32>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _genre: Option<&str>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<reverie_core::SubsonicAlbum>> {
        Ok(vec![])
    }

    async fn get_album_list2(
        &self,
        _list_type: &str,
        _size: Option<i32>,
        _offset: Option<i32>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _genre: Option<&str>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<reverie_core::SubsonicAlbum>> {
        Ok(vec![reverie_core::SubsonicAlbum {
            id: "album-1".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("artist-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            cover_art: None,
            song_count: 10,
            duration: 3600.0,
            play_count: None,
            created: None,
            starred: None,
            user_rating: None,
        }])
    }

    async fn get_random_songs(
        &self,
        _size: Option<i32>,
        _genre: Option<&str>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<reverie_core::MediaFile>> {
        Ok(vec![])
    }

    async fn get_songs_by_genre(
        &self,
        _genre: &str,
        _count: Option<i32>,
        _offset: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<reverie_core::MediaFile>> {
        Ok(vec![])
    }

    async fn get_now_playing(&self) -> Result<Vec<reverie_core::SubsonicNowPlaying>> {
        Ok(vec![])
    }

    async fn get_starred(&self, _music_folder_id: Option<i32>) -> Result<reverie_core::SubsonicStarred> {
        Ok(reverie_core::SubsonicStarred {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn get_starred2(&self, _music_folder_id: Option<i32>) -> Result<reverie_core::SubsonicStarred> {
        self.get_starred(_music_folder_id).await
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
    ) -> Result<reverie_core::SubsonicSearchResult2> {
        Ok(reverie_core::SubsonicSearchResult2 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn search3(
        &self,
        _query: &str,
        _artist_count: Option<i32>,
        _artist_offset: Option<i32>,
        _album_count: Option<i32>,
        _album_offset: Option<i32>,
        _song_count: Option<i32>,
        _song_offset: Option<i32>,
    ) -> Result<reverie_core::SubsonicSearchResult3> {
        Ok(reverie_core::SubsonicSearchResult3 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn get_playlists(&self, _username: Option<&str>) -> Result<Vec<reverie_core::SubsonicPlaylist>> {
        Ok(vec![])
    }

    async fn get_playlist(&self, _id: &str) -> Result<Option<reverie_core::SubsonicPlaylistWithSongs>> {
        Ok(None)
    }

    async fn create_playlist(
        &self,
        _name: Option<&str>,
        _playlist_id: Option<&str>,
        _song_ids: &[&str],
    ) -> Result<reverie_core::SubsonicPlaylistWithSongs> {
        Ok(reverie_core::SubsonicPlaylistWithSongs {
            id: "playlist-1".to_string(),
            name: "Test Playlist".to_string(),
            comment: None,
            owner: "admin".to_string(),
            public: false,
            song_count: 0,
            duration: 0,
            created: chrono::Utc::now(),
            changed: chrono::Utc::now(),
            cover_art: None,
            entries: vec![],
        })
    }

    async fn update_playlist(
        &self,
        _playlist_id: &str,
        _name: Option<&str>,
        _comment: Option<&str>,
        _public: Option<bool>,
        _song_ids_to_add: &[&str],
        _song_indexes_to_remove: &[i32],
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_playlist(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_stream_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(Some("/music/test.mp3".to_string()))
    }

    async fn get_cover_art_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(Some("/covers/test.jpg".to_string()))
    }

    async fn get_lyrics(&self, _artist: Option<&str>, _title: Option<&str>) -> Result<Option<reverie_core::SubsonicLyrics>> {
        Ok(None)
    }

    async fn get_lyrics_by_song_id(&self, _id: &str) -> Result<Vec<reverie_core::SubsonicStructuredLyrics>> {
        Ok(vec![])
    }

    async fn get_avatar_path(&self, _username: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn star(&self, _ids: &[&str], _album_ids: &[&str], _artist_ids: &[&str]) -> Result<()> {
        Ok(())
    }

    async fn unstar(&self, _ids: &[&str], _album_ids: &[&str], _artist_ids: &[&str]) -> Result<()> {
        Ok(())
    }

    async fn set_rating(&self, _id: &str, _rating: i32) -> Result<()> {
        Ok(())
    }

    async fn scrobble(&self, _id: &str, _time: Option<i64>, _submission: bool) -> Result<()> {
        Ok(())
    }

    async fn get_bookmarks(&self) -> Result<Vec<reverie_core::SubsonicBookmark>> {
        Ok(vec![])
    }

    async fn create_bookmark(&self, _id: &str, _position: i64, _comment: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn delete_bookmark(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_play_queue(&self) -> Result<Option<reverie_core::SubsonicPlayQueue>> {
        Ok(None)
    }

    async fn save_play_queue(&self, _ids: &[&str], _current: Option<&str>, _position: Option<i64>) -> Result<()> {
        Ok(())
    }

    async fn get_shares(&self) -> Result<Vec<reverie_core::SubsonicShare>> {
        Ok(vec![])
    }

    async fn create_share(&self, _ids: &[&str], _description: Option<&str>, _expires: Option<i64>) -> Result<reverie_core::SubsonicShare> {
        Ok(reverie_core::SubsonicShare {
            id: "share-1".to_string(),
            url: "http://example.com/share/1".to_string(),
            description: None,
            username: "admin".to_string(),
            created: chrono::Utc::now(),
            expires: None,
            last_visited: None,
            visit_count: 0,
            entries: vec![],
        })
    }

    async fn update_share(&self, _id: &str, _description: Option<&str>, _expires: Option<i64>) -> Result<()> {
        Ok(())
    }

    async fn delete_share(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_internet_radio_stations(&self) -> Result<Vec<reverie_core::SubsonicInternetRadioStation>> {
        Ok(vec![])
    }

    async fn create_internet_radio_station(&self, _stream_url: &str, _name: &str, _homepage_url: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn update_internet_radio_station(&self, _id: &str, _stream_url: &str, _name: &str, _homepage_url: Option<&str>) -> Result<()> {
        Ok(())
    }

    async fn delete_internet_radio_station(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_user(&self, _username: &str) -> Result<Option<reverie_core::SubsonicUser>> {
        Ok(Some(reverie_core::SubsonicUser {
            username: "admin".to_string(),
            email: None,
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
            jukebox_role: true,
            share_role: true,
            video_conversion_role: false,
            avatar_last_changed: None,
            folders: vec![1],
        }))
    }

    async fn get_users(&self) -> Result<Vec<reverie_core::SubsonicUser>> {
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
    ) -> Result<()> {
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
        _video_conversion_role: Option<bool>,
        _music_folder_ids: Option<&[i32]>,
        _max_bit_rate: Option<i32>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_user(&self, _username: &str) -> Result<()> {
        Ok(())
    }

    async fn change_password(&self, _username: &str, _password: &str) -> Result<()> {
        Ok(())
    }

    async fn get_scan_status(&self) -> Result<reverie_core::SubsonicScanStatus> {
        Ok(reverie_core::SubsonicScanStatus {
            scanning: false,
            count: 100,
            folder_count: 1,
            last_scan: None,
            error: None,
            scan_type: None,
            elapsed_time: None,
        })
    }

    async fn start_scan(&self) -> Result<reverie_core::SubsonicScanStatus> {
        self.get_scan_status().await
    }
}

// === Test Helper Functions ===

fn create_test_router() -> axum::Router {
    let storage = Arc::new(MockSubsonicStorage::new());
    let state = SubsonicState::new(storage);
    create_router::<MockSubsonicStorage>().with_state(state)
}

async fn get_json_response(router: axum::Router, uri: &str) -> serde_json::Value {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

// === Test Cases ===

#[tokio::test]
async fn test_ping_returns_ok() {
    let router = create_test_router();
    let json = get_json_response(router, "/ping?f=json").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_license_returns_valid() {
    let router = create_test_router();
    let json = get_json_response(router, "/getLicense?f=json").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert_eq!(json["subsonic-response"]["license"]["valid"], true);
}

#[tokio::test]
async fn test_get_music_folders() {
    let router = create_test_router();
    let json = get_json_response(router, "/getMusicFolders?f=json").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
    let folders = &json["subsonic-response"]["musicFolders"]["musicFolder"];
    assert!(folders.is_array());
}

#[tokio::test]
async fn test_get_artists() {
    let router = create_test_router();
    let json = get_json_response(router, "/getArtists?f=json").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_album_list2() {
    let router = create_test_router();
    let json = get_json_response(router, "/getAlbumList2?f=json&type=recent").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_search3() {
    let router = create_test_router();
    let json = get_json_response(router, "/search3?f=json&query=test").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_scan_status() {
    let router = create_test_router();
    let json = get_json_response(router, "/getScanStatus?f=json").await;
    
    assert_eq!(json["subsonic-response"]["status"], "ok");
}
