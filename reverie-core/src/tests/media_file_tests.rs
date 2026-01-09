//! MediaFile tests

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
