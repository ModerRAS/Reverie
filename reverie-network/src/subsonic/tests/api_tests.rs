//! Subsonic API 集成测试

use crate::subsonic::create_router;
use crate::subsonic::tests::mock_storage::MockSubsonicStorage;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use reverie_core::Track;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// === 测试辅助函数 ===

pub fn create_test_router() -> axum::Router {
    let storage = Arc::new(MockSubsonicStorage::new());
    let state = crate::subsonic::SubsonicState::new(storage);
    create_router::<MockSubsonicStorage>().with_state(state)
}

async fn get_json_response(router: axum::Router, uri: &str) -> serde_json::Value {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Helper for error response tests (Subsonic always returns 200, error is in body)
async fn get_json_response_error(router: axum::Router, uri: &str) -> serde_json::Value {
    let response = router
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK); // Subsonic always returns 200, error is in body
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

// === 测试用例 ===

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

#[tokio::test]
async fn test_get_artist_info() {
    let router = create_test_router();
    let json = get_json_response(router, "/getArtistInfo?f=json&id=artist-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    // artistInfo 应该存在
    assert!(json["subsonic-response"]["artistInfo"].is_object());
}

#[tokio::test]
async fn test_get_artist_info2() {
    let router = create_test_router();
    let json = get_json_response(router, "/getArtistInfo2?f=json&id=artist-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    // artistInfo2 应该存在
    assert!(json["subsonic-response"]["artistInfo2"].is_object());
}

#[tokio::test]
async fn test_get_album_info() {
    let router = create_test_router();
    let json = get_json_response(router, "/getAlbumInfo?f=json&id=album-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    // albumInfo 应该存在
    assert!(json["subsonic-response"]["albumInfo"].is_object());
}

#[tokio::test]
async fn test_get_album_info2() {
    let router = create_test_router();
    let json = get_json_response(router, "/getAlbumInfo2?f=json&id=album-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    // albumInfo2 与 albumInfo 使用相同的字段名（参考 navidrome 实现）
    assert!(json["subsonic-response"]["albumInfo"].is_object());
}

#[tokio::test]
async fn test_get_similar_songs() {
    let router = create_test_router();
    let json = get_json_response(router, "/getSimilarSongs?f=json&id=song-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_top_songs() {
    let router = create_test_router();
    let json = get_json_response(router, "/getTopSongs?f=json&artist=TestArtist").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_lyrics() {
    let router = create_test_router();
    let json = get_json_response(router, "/getLyrics?f=json&artist=TestArtist&title=TestSong").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === Bookmark 测试 ===

#[tokio::test]
async fn test_get_bookmarks() {
    let router = create_test_router();
    let json = get_json_response(router, "/getBookmarks?f=json").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    // bookmarks 字段应该存在
    assert!(json["subsonic-response"]["bookmarks"].is_object());
}

#[tokio::test]
async fn test_create_bookmark() {
    let router = create_test_router();
    let json = get_json_response(router, "/createBookmark?f=json&id=song-1&position=1000").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_bookmark() {
    let router = create_test_router();
    let json = get_json_response(router, "/deleteBookmark?f=json&id=song-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_play_queue() {
    let router = create_test_router();
    let json = get_json_response(router, "/getPlayQueue?f=json").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_save_play_queue() {
    let router = create_test_router();
    let json = get_json_response(router, "/savePlayQueue?f=json&id=song-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === Share 测试 ===

#[tokio::test]
async fn test_get_shares() {
    let router = create_test_router();
    let json = get_json_response(router, "/getShares?f=json").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["shares"].is_object());
}

#[tokio::test]
async fn test_create_share() {
    let router = create_test_router();
    let json = get_json_response(router, "/createShare?f=json&id=song-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_share() {
    let router = create_test_router();
    let json = get_json_response(router, "/deleteShare?f=json&id=share-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === Internet Radio 测试 ===

#[tokio::test]
async fn test_get_internet_radio_stations() {
    let router = create_test_router();
    let json = get_json_response(router, "/getInternetRadioStations?f=json").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["internetRadioStations"].is_object());
}

#[tokio::test]
async fn test_create_internet_radio_station() {
    let router = create_test_router();
    let json = get_json_response(router, "/createInternetRadioStation?f=json&name=TestRadio&streamUrl=http://example.com/stream").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_internet_radio_station() {
    let router = create_test_router();
    let json = get_json_response(router, "/deleteInternetRadioStation?f=json&id=radio-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === OpenSubsonic Extensions 测试 ===

#[tokio::test]
async fn test_get_open_subsonic_extensions() {
    let router = create_test_router();
    let json = get_json_response(router, "/getOpenSubsonicExtensions?f=json").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_lyrics_by_song_id() {
    let router = create_test_router();
    let json = get_json_response(router, "/getLyricsBySongId?f=json&id=song-1").await;

    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === User Management Tests ===

#[tokio::test]
async fn test_create_user_success() {
    let router = create_test_router();
    let json = get_json_response(
        router,
        "/createUser?f=json&username=testuser&password=test123&email=test@test.com&adminRole=false"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_create_user_missing_username() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/createUser?f=json&password=test123"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

#[tokio::test]
async fn test_create_user_duplicate() {
    let router = create_test_router();
    // Create first user
    let _ = get_json_response(
        router.clone(),
        "/createUser?f=json&username=dupuser&password=test123&email=test@test.com"
    ).await;
    // Try to create duplicate
    let json = get_json_response_error(
        router,
        "/createUser?f=json&username=dupuser&password=test456&email=dup@test.com"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 40);
}

#[tokio::test]
async fn test_update_user_success() {
    let router = create_test_router();
    // First create a user
    let _ = get_json_response(
        router.clone(),
        "/createUser?f=json&username=updateme&password=oldpwd&email=old@test.com"
    ).await;
    // Then update the user
    let json = get_json_response(
        router,
        "/updateUser?f=json&username=updateme&email=new@test.com"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_update_user_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/updateUser?f=json&username=nonexistent&email=test@test.com"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

#[tokio::test]
async fn test_update_user_missing_username() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/updateUser?f=json&email=test@test.com"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

#[tokio::test]
async fn test_delete_user_success() {
    let router = create_test_router();
    // First create a user
    let _ = get_json_response(
        router.clone(),
        "/createUser?f=json&username=todelete&password=secret&email=del@test.com"
    ).await;
    // Then delete the user
    let json = get_json_response(
        router,
        "/deleteUser?f=json&username=todelete"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/deleteUser?f=json&username=nonexistent"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

#[tokio::test]
async fn test_delete_user_missing_username() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/deleteUser?f=json"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

#[tokio::test]
async fn test_change_password_success() {
    let router = create_test_router();
    // First create a user
    let _ = get_json_response(
        router.clone(),
        "/createUser?f=json&username=pwduser&password=oldpwd&email=pwd@test.com"
    ).await;
    // Then change password
    let json = get_json_response(
        router,
        "/changePassword?f=json&username=pwduser&password=newpwd123"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_change_password_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(
        router,
        "/changePassword?f=json&username=nonexistent&password=newpwd"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

#[tokio::test]
async fn test_change_password_missing_params() {
    let router = create_test_router();
    // Missing password
    let json = get_json_response_error(
        router,
        "/changePassword?f=json&username=testuser"
    ).await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

// === Video Tests ===

#[tokio::test]
async fn test_get_videos_empty() {
    let router = create_test_router();
    let json = get_json_response(router, "/getVideos?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    // video should be an array (possibly empty)
    assert!(json["subsonic-response"]["video"].is_array());
}

// === VideoInfo Tests ===

#[tokio::test]
async fn test_get_video_info_success() {
    let router = create_test_router();
    // VideoInfo requires a specific video ID - use mock's test data
    let json = get_json_response(router, "/getVideoInfo?f=json&id=video-1").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["videoInfo"].is_object());
}

#[tokio::test]
async fn test_get_video_info_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/getVideoInfo?f=json&id=nonexistent").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

#[tokio::test]
async fn test_get_video_info_missing_id() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/getVideoInfo?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

// === Captions Tests ===

#[tokio::test]
async fn test_get_captions_empty() {
    let router = create_test_router();
    let json = get_json_response(router, "/getCaptions?f=json&id=video-1").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    // captions should be present (even if empty)
    assert!(json["subsonic-response"]["captions"].is_object());
}

#[tokio::test]
async fn test_get_captions_missing_id() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/getCaptions?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

// === HLS Endpoint Tests ===

#[tokio::test]
async fn test_hls_returns_not_implemented() {
    let router = create_test_router();
    // hls endpoint returns HTTP 501, not a JSON error
    let response = router
        .oneshot(Request::builder().uri("/hls?f=json&id=1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
}

// === Jukebox Tests ===

#[tokio::test]
async fn test_jukebox_control_placeholder() {
    let router = create_test_router();
    let json = get_json_response(router, "/jukeboxControl?f=json&action=status").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["jukeboxStatus"].is_object());
}

// === Chat Messages Tests ===

#[tokio::test]
async fn test_get_chat_messages_empty() {
    let router = create_test_router();
    let json = get_json_response(router, "/getChatMessages?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    // chatMessages should be an object with chatMessage array
    assert!(json["subsonic-response"]["chatMessages"]["chatMessage"].is_array());
}

#[tokio::test]
async fn test_get_chat_messages_with_since() {
    let router = create_test_router();
    // since parameter filters messages by timestamp (milliseconds)
    let json = get_json_response(router, "/getChatMessages?f=json&since=1000000").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["chatMessages"]["chatMessage"].is_array());
}

// === Add Chat Message Tests ===

#[tokio::test]
async fn test_add_chat_message_success() {
    let router = create_test_router();
    let json = get_json_response(router, "/addChatMessage?f=json&message=Hello").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_add_chat_message_missing() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/addChatMessage?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

// === Podcast Tests ===

#[tokio::test]
async fn test_get_podcasts_empty() {
    let router = create_test_router();
    let json = get_json_response(router, "/getPodcasts?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    // podcasts should be an object with channel array
    assert!(json["subsonic-response"]["podcasts"]["channel"].is_array());
}

#[tokio::test]
async fn test_get_podcasts_with_channels() {
    let router = create_test_router();
    let json = get_json_response(router, "/getPodcasts?f=json&includeEpisodes=true").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["podcasts"]["channel"].is_array());
}

// === getNewestPodcasts Tests ===

#[tokio::test]
async fn test_get_newest_podcasts() {
    let router = create_test_router();
    let json = get_json_response(router, "/getNewestPodcasts?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["newestPodcasts"]["episode"].is_array());
}

#[tokio::test]
async fn test_get_newest_podcasts_with_count() {
    let router = create_test_router();
    let json = get_json_response(router, "/getNewestPodcasts?f=json&count=5").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
    assert!(json["subsonic-response"]["newestPodcasts"]["episode"].is_array());
}

#[tokio::test]
async fn test_refresh_podcasts_placeholder() {
    let router = create_test_router();
    let json = get_json_response(router, "/refreshPodcasts?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

// === createPodcastChannel Tests ===

#[tokio::test]
async fn test_create_podcast_channel_success() {
    let router = create_test_router();
    let json = get_json_response(router, "/createPodcastChannel?f=json&url=https://example.com/feed.xml&title=TestPodcast").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_create_podcast_channel_missing_url() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/createPodcastChannel?f=json").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 10);
}

// === deletePodcastChannel Tests ===

#[tokio::test]
async fn test_delete_podcast_channel_success() {
    let router = create_test_router();
    // First create a channel
    let create_json = get_json_response(router.clone(), "/createPodcastChannel?f=json&url=https://example.com/feed.xml&title=TestPodcast").await;
    assert_eq!(create_json["subsonic-response"]["status"], "ok");

    // Now delete the channel we just created (channel-1)
    let json = get_json_response(router, "/deletePodcastChannel?f=json&id=channel-1").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_podcast_channel_not_found() {
    let router = create_test_router();
    // Deleting a non-existent channel should return error 70
    let json = get_json_response_error(router, "/deletePodcastChannel?f=json&id=nonexistent").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

// === deletePodcastEpisode Tests ===

#[tokio::test]
async fn test_delete_podcast_episode_success() {
    let router = create_test_router();
    // Delete an episode by ID (mock returns error 70 for non-existent)
    let json = get_json_response(router, "/deletePodcastEpisode?f=json&id=episode-1").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_podcast_episode_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/deletePodcastEpisode?f=json&id=nonexistent").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

// === downloadPodcastEpisode Tests ===

#[tokio::test]
async fn test_download_podcast_episode() {
    let router = create_test_router();
    // Download an episode by ID (mock returns error 70 for non-existent)
    let json = get_json_response(router, "/downloadPodcastEpisode?f=json&id=episode-1").await;
    assert_eq!(json["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_download_podcast_episode_not_found() {
    let router = create_test_router();
    let json = get_json_response_error(router, "/downloadPodcastEpisode?f=json&id=nonexistent").await;
    assert_eq!(json["subsonic-response"]["status"], "failed");
    assert_eq!(json["subsonic-response"]["error"]["code"], 70);
}

// === Stream CUE Virtual Track Tests ===

/// Helper to create a router with a CUE virtual track pre-populated
pub(super) fn create_cue_test_router(
    track_id: Uuid,
    source_file: &str,
    file_data: Vec<u8>,
    file_size: u64,
    byte_offset_start: u64,
    byte_offset_end: Option<u64>,
    format: &str,
) -> axum::Router {
    use chrono::Utc;

    let storage = Arc::new(MockSubsonicStorage::new());

    // Register the track
    let track = Track {
        id: track_id,
        title: "CUE Virtual Track".to_string(),
        album_id: None,
        artist_id: None,
        duration: 180,
        file_path: "/tmp/nonexistent.flac".to_string(), // not used for CUE tracks
        file_size,
        bitrate: 320,
        format: format.to_string(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Rock".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_file: Some(source_file.to_string()),
        byte_offset_start: Some(byte_offset_start),
        byte_offset_end,
        cue_path: Some("/music/test.cue".to_string()),
        is_cue_virtual: Some(true),
    };
    storage.tracks.write().unwrap().insert(track_id, track);

    // Write the source file data
    storage
        .file_data
        .write()
        .unwrap()
        .insert(source_file.to_string(), file_data);

    let state = crate::subsonic::SubsonicState::new(storage);
    create_router::<MockSubsonicStorage>().with_state(state)
}

/// Helper to create a router with a normal (non-CUE) track
pub(super) fn create_normal_track_test_router(track_id: Uuid, file_path: &str, file_data: Vec<u8>) -> axum::Router {
    use chrono::Utc;

    let storage = Arc::new(MockSubsonicStorage::new());

    let track = Track {
        id: track_id,
        title: "Normal Track".to_string(),
        album_id: None,
        artist_id: None,
        duration: 180,
        file_path: file_path.to_string(),
        file_size: file_data.len() as u64,
        bitrate: 320,
        format: "mp3".to_string(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Pop".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_file: None,
        byte_offset_start: None,
        byte_offset_end: None,
        cue_path: None,
        is_cue_virtual: None,
    };
    storage.tracks.write().unwrap().insert(track_id, track);

    storage
        .file_data
        .write()
        .unwrap()
        .insert(file_path.to_string(), file_data);

    let state = crate::subsonic::SubsonicState::new(storage);
    create_router::<MockSubsonicStorage>().with_state(state)
}

#[tokio::test]
async fn test_stream_cue_virtual_track_returns_206() {
    use axum::http::header;

    let track_id = Uuid::new_v4();
    // Create 50000 bytes of dummy audio data
    let file_data: Vec<u8> = (0..50000u64).map(|i| (i % 256) as u8).collect();
    let source_file = "/music/cue_test.flac";
    let byte_start = 1024u64;
    let byte_end = 20480u64;

    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        50000,
        byte_start,
        Some(byte_end),
        "flac",
    );

    let uri = format!("/stream?id={}", track_id);
    let response = router
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let headers = response.headers();
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "audio/flac"
    );
    assert!(headers.contains_key(header::CONTENT_RANGE));
    assert_eq!(
        headers.get(header::ACCEPT_RANGES).unwrap(),
        "bytes"
    );

    let actual_content_range = headers
        .get(header::CONTENT_RANGE)
        .unwrap()
        .to_str()
        .unwrap();
    let expected_range = format!("bytes {}-{}/{}", byte_start, byte_end - 1, 50000);
    assert_eq!(actual_content_range, expected_range);

    // Verify the body is the correct slice
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let expected_size = (byte_end - byte_start) as usize;
    assert_eq!(body.len(), expected_size);
    // Verify content matches the range
    for i in 0..expected_size {
        assert_eq!(body[i], ((byte_start as usize + i) % 256) as u8);
    }
}

#[tokio::test]
async fn test_stream_cue_track_content_range_header() {
    use axum::http::header;

    let track_id = Uuid::new_v4();
    let file_data: Vec<u8> = (0..10000u64).map(|i| (i % 256) as u8).collect();
    let source_file = "/music/cue_header_test.flac";
    let byte_start = 500u64;
    let byte_end = 2000u64;

    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        10000,
        byte_start,
        Some(byte_end),
        "flac",
    );

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/stream?id={}", track_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);

    let headers = response.headers();
    let content_range = headers
        .get(header::CONTENT_RANGE)
        .unwrap()
        .to_str()
        .unwrap();
    // Content-Range format: "bytes {start}-{end}/{total}"
    assert_eq!(content_range, "bytes 500-1999/10000");

    let content_length = headers
        .get(header::CONTENT_LENGTH)
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert_eq!(content_length, 1500); // 2000 - 500
}

#[tokio::test]
async fn test_stream_normal_track_returns_200() {
    use axum::http::header;

    let track_id = Uuid::new_v4();
    let file_data = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    let file_path = "/music/normal_test.mp3";

    let router = create_normal_track_test_router(track_id, file_path, file_data.clone());

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/stream?id={}", track_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "audio/mpeg"
    );
    assert_eq!(
        headers.get(header::ACCEPT_RANGES).unwrap(),
        "bytes"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), file_data.as_slice());
}

#[tokio::test]
async fn test_stream_cue_track_range_exceeded() {
    let track_id = Uuid::new_v4();
    let file_data: Vec<u8> = (0..1000u64).map(|i| (i % 256) as u8).collect();
    let source_file = "/music/cue_416_test.flac";
    // byte_offset_start is >= file size
    let byte_start = 2000u64; // >= 1000

    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        1000,
        byte_start,
        Some(2100),
        "flac",
    );

    let response = router
        .oneshot(
            Request::builder()
                .uri(format!("/stream?id={}", track_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::RANGE_NOT_SATISFIABLE);
}
