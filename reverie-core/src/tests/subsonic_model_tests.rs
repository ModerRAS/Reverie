//! Subsonic model tests

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
    assert!(info.large_image_url.is_some());
}
