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
