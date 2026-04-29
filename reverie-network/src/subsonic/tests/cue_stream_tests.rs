//! Integration tests for CUE HTTP Range streaming.
//!
//! Validates HTTP 206 Partial Content responses, Content-Range headers,
//! byte content correctness, range-exceeded 416 handling, and normal track fallback.

use crate::subsonic::tests::api_tests::{create_cue_test_router, create_normal_track_test_router};
use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a deterministic byte pattern that is easy to verify at any offset.
/// Returns `(pattern * 37 + 128) % 256` — a cheap pseudo-random sequence.
fn make_pattern_audio(len: u64) -> Vec<u8> {
    (0..len).map(|i| ((i.wrapping_mul(37).wrapping_add(128)) % 256) as u8).collect()
}

/// Verify that `data` matches `make_pattern_audio` starting at byte `offset`.
fn assert_pattern_match(data: &[u8], offset: u64) {
    for (i, byte) in data.iter().enumerate() {
        let expected = ((offset + i as u64).wrapping_mul(37).wrapping_add(128) % 256) as u8;
        assert_eq!(
            *byte, expected,
            "byte mismatch at position {} (absolute offset {}): got {}, expected {}",
            i, offset + i as u64, byte, expected
        );
    }
}

// ===========================================================================
// Test 1 – CUE virtual track returns HTTP 206 and Content-Range header
// ===========================================================================

#[tokio::test]
async fn test_stream_cue_virtual_track_returns_206_partial_content() {
    let track_id = Uuid::new_v4();
    let source_file = "/music/album.cue.flac";
    let total_size: u64 = 100_000;
    let byte_start: u64 = 10_000;
    let byte_end: u64 = 25_000;

    let file_data = make_pattern_audio(total_size);

    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        total_size,
        byte_start,
        Some(byte_end),
        "flac",
    );

    let uri = format!("/stream?id={}", track_id);
    let response = router
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    // Status
    assert_eq!(
        response.status(),
        StatusCode::PARTIAL_CONTENT,
        "expected 206 Partial Content"
    );

    let headers = response.headers();

    // Content-Type
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "audio/flac"
    );

    // Accept-Ranges
    assert_eq!(
        headers.get(header::ACCEPT_RANGES).unwrap(),
        "bytes"
    );

    // Content-Range format: "bytes {start}-{end}/{total}"
    let content_range = headers
        .get(header::CONTENT_RANGE)
        .expect("missing Content-Range header")
        .to_str()
        .unwrap();
    assert_eq!(
        content_range,
        format!("bytes {}-{}/{}", byte_start, byte_end - 1, total_size)
    );
}

// ===========================================================================
// Test 2 – Returned bytes match the expected range from parent file
// ===========================================================================

#[tokio::test]
async fn test_stream_cue_track_content_matches_expected_bytes() {
    let track_id = Uuid::new_v4();
    let source_file = "/music/live_concert.flac";
    let total_size: u64 = 200_000;
    let byte_start: u64 = 5_000;
    let byte_end: u64 = 15_000;

    let file_data = make_pattern_audio(total_size);
    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        total_size,
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let expected_len = (byte_end - byte_start) as usize;
    assert_eq!(body.len(), expected_len, "body length mismatch");

    // Byte-by-byte validation against the known pattern
    assert_pattern_match(&body, byte_start);
}

// ===========================================================================
// Test 3 – Range exceeded returns HTTP 416 Range Not Satisfiable
// ===========================================================================

#[tokio::test]
async fn test_stream_cue_track_range_exceeded_returns_416() {
    let track_id = Uuid::new_v4();
    let source_file = "/music/tiny_file.flac";
    let total_size: u64 = 1_000;
    // byte_offset_start >= file_size triggers 416
    let byte_start: u64 = 5_000; // > total_size
    let byte_end: u64 = 6_000;

    let file_data = make_pattern_audio(total_size);
    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        total_size,
        byte_start,
        Some(byte_end),
        "flac",
    );

    let uri = format!("/stream?id={}", track_id);
    let response = router
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::RANGE_NOT_SATISFIABLE,
        "expected 416 Range Not Satisfiable"
    );

    // Content-Range should be "bytes */{total}"
    let content_range = response
        .headers()
        .get(header::CONTENT_RANGE)
        .expect("missing Content-Range header on 416")
        .to_str()
        .unwrap();
    assert_eq!(content_range, format!("bytes */{}", total_size));

    // Body should be empty
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(body.is_empty(), "expected empty body for 416 response");
}

// ===========================================================================
// Test 4 – Normal (non-CUE) track returns HTTP 200 with full file
// ===========================================================================

#[tokio::test]
async fn test_stream_normal_track_returns_200_full_file() {
    let track_id = Uuid::new_v4();
    let file_path = "/music/song.mp3";

    let file_data: Vec<u8> = b"FAKE_MP3_HEADER\x00\x01\x02\x03\x04\x05MPEG_FRAME_DATA"
        .iter()
        .copied()
        .chain((0..200u8).map(|i| i.wrapping_mul(13)))
        .collect();

    let router = create_normal_track_test_router(track_id, file_path, file_data.clone());

    let uri = format!("/stream?id={}", track_id);
    let response = router
        .oneshot(Request::builder().uri(&uri).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "normal track should return 200 OK"
    );

    let headers = response.headers();
    assert_eq!(
        headers.get(header::CONTENT_TYPE).unwrap(),
        "audio/mpeg"
    );
    assert_eq!(
        headers.get(header::ACCEPT_RANGES).unwrap(),
        "bytes"
    );

    // Normal track should NOT have a Content-Range header
    assert!(
        !headers.contains_key(header::CONTENT_RANGE),
        "normal track should not have Content-Range header"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.as_ref(), file_data.as_slice(), "full file content mismatch");
}

// ===========================================================================
// Test 5 – Content-Length equals byte_offset_end - byte_offset_start
// ===========================================================================

#[tokio::test]
async fn test_stream_cue_track_content_length_matches_range() {
    let track_id = Uuid::new_v4();
    let source_file = "/music/demo.flac";
    let total_size: u64 = 50_000;
    let byte_start: u64 = 2_048;
    let byte_end: u64 = 30_000;

    let file_data = make_pattern_audio(total_size);
    let router = create_cue_test_router(
        track_id,
        source_file,
        file_data,
        total_size,
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

    let content_length = response
        .headers()
        .get(header::CONTENT_LENGTH)
        .expect("missing Content-Length header")
        .to_str()
        .unwrap()
        .parse::<u64>()
        .expect("Content-Length should be a valid integer");

    let expected_len = byte_end - byte_start;
    assert_eq!(
        content_length, expected_len,
        "Content-Length ({}) should equal byte_end - byte_start ({})",
        content_length, expected_len
    );

    // Sanity: body length should also match
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body.len() as u64, expected_len);
}
