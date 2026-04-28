//! Subsonic API 集成测试

use crate::subsonic::create_router;
use crate::subsonic::tests::mock_storage::MockSubsonicStorage;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use std::sync::Arc;
use tower::ServiceExt;

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
