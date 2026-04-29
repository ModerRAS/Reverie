//! Basic smoke tests to verify the server starts and the ping endpoint works.

use crate::common::TestContext;

#[tokio::test]
async fn test_ping_returns_ok() {
    let ctx = TestContext::new().await.expect("Failed to start server");
    let resp = ctx
        .subsonic_get("ping", &[("f", "json")])
        .await
        .expect("Request failed");
    assert_eq!(
        resp["subsonic-response"]["status"].as_str(),
        Some("ok")
    );
}

#[tokio::test]
async fn test_server_starts_and_stops() {
    let ctx = TestContext::new().await.expect("Failed to start server");
    assert!(ctx.base_url().starts_with("http://127.0.0.1:"));
}
