//! E2E integration tests for all Subsonic API endpoints.
//!
//! Tests the full server stack: HTTP → router → handler → storage → response.
//! Uses the `TestContext` harness from `tests/common/mod.rs`.
//!
//! Convention: each endpoint gets a happy-path test and an error-path test.
//! - Happy: asserts `"status":"ok"` in `subsonic-response`
//! - Error: asserts `"status":"failed"` in `subsonic-response`
//! - Binary endpoints (stream, download, getCoverArt, getAvatar): error
//!   path tests check for Subsonic JSON errors (missing params); happy path
//!   tests validate HTTP success and content-type headers.

use crate::common::TestContext;
use chrono::Utc;
use reverie_core::{Album, Artist, Track};
use reverie_storage::{AlbumStorage, ArtistStorage, FileStorage, TrackStorage};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper — create test data for endpoints that need albums/artists/tracks
// ---------------------------------------------------------------------------

fn make_test_track(artist_id: Uuid, album_id: Uuid) -> Track {
    Track {
        id: Uuid::new_v4(),
        title: "Test Song".into(),
        album_id: Some(album_id),
        artist_id: Some(artist_id),
        duration: 180,
        file_path: "/music/test.mp3".into(),
        file_size: 1024,
        bitrate: 320,
        format: "mp3".into(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Rock".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_file: None,
        byte_offset_start: None,
        byte_offset_end: None,
        cue_path: None,
        is_cue_virtual: None,
    }
}

fn make_test_album(artist_id: Uuid) -> Album {
    Album {
        id: Uuid::new_v4(),
        name: "Test Album".into(),
        artist_id: Some(artist_id),
        year: Some(2024),
        genre: Some("Rock".into()),
        cover_art_path: Some("/covers/test.jpg".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

fn make_test_artist() -> Artist {
    Artist {
        id: Uuid::new_v4(),
        name: "Test Artist".into(),
        bio: Some("A test artist.".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Seed a full artist+album+track hierarchy and return their IDs.
async fn seed_test_hierarchy(
    ctx: &TestContext,
) -> Result<(Uuid, Uuid, Uuid), Box<dyn std::error::Error + Send + Sync>> {
    let artist = make_test_artist();
    let album = make_test_album(artist.id);
    let track = make_test_track(artist.id, album.id);
    ctx.storage().save_artist(&artist).await?;
    ctx.storage().save_album(&album).await?;
    ctx.storage().save_track(&track).await?;
    Ok((artist.id, album.id, track.id))
}

// ===========================================================================
// System Endpoints
// ===========================================================================

#[tokio::test]
async fn test_ping_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("ping", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// ping has no error case — always returns ok

#[tokio::test]
async fn test_get_license_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getLicense", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
    assert_eq!(resp["subsonic-response"]["license"]["valid"], true);
}

// getLicense has no error case — always returns ok

// ===========================================================================
// Browsing Endpoints
// ===========================================================================

#[tokio::test]
async fn test_get_music_folders_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getMusicFolders", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getMusicFolders has no required params → always returns ok

#[tokio::test]
async fn test_get_genres_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getGenres", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getGenres has no required params → always returns ok

#[tokio::test]
async fn test_get_indexes_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getIndexes", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getIndexes has no required params → always returns ok

#[tokio::test]
async fn test_get_artists_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getArtists", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getArtists has no required params → always returns ok

#[tokio::test]
async fn test_get_music_directory_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (artist_id, _album_id, _track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    // music directory lookup by artist id (artists are top-level directories)
    let resp = ctx
        .subsonic_get(
            "getMusicDirectory",
            &[("f", "json"), ("id", &artist_id.to_string())],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_music_directory_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getMusicDirectory", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_artist_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (artist_id, _album_id, _track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get(
            "getArtist",
            &[("f", "json"), ("id", &artist_id.to_string())],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_artist_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getArtist", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_album_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (_artist_id, album_id, _track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get("getAlbum", &[("f", "json"), ("id", &album_id.to_string())])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_album_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getAlbum", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_song_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (_artist_id, _album_id, track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get("getSong", &[("f", "json"), ("id", &track_id.to_string())])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_song_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("getSong", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_album_info_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (_artist_id, album_id, _track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get(
            "getAlbumInfo",
            &[("f", "json"), ("id", &album_id.to_string())],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_album_info_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getAlbumInfo", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_artist_info_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (artist_id, _album_id, _track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get(
            "getArtistInfo",
            &[("f", "json"), ("id", &artist_id.to_string())],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_artist_info_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getArtistInfo", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_album_list2_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getAlbumList2", &[("f", "json"), ("type", "recent")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_album_list2_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getAlbumList2", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// Search Endpoints
// ===========================================================================

#[tokio::test]
async fn test_search2_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("search2", &[("f", "json"), ("query", "test")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_search2_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("search2", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_search3_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("search3", &[("f", "json"), ("query", "test")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_search3_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("search3", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// Playlist Endpoints
// ===========================================================================

#[tokio::test]
async fn test_get_playlists_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getPlaylists", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getPlaylists (no required params) always returns ok

#[tokio::test]
async fn test_get_playlist_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getPlaylist", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_create_playlist_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("createPlaylist", &[("f", "json"), ("name", "MyPlaylist")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_create_playlist_error() {
    let ctx = TestContext::new().await.unwrap();
    // Neither playlistId nor name
    let resp = ctx
        .subsonic_get("createPlaylist", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_update_playlist_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("updatePlaylist", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_delete_playlist_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("deletePlaylist", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// User Endpoints
// ===========================================================================

#[tokio::test]
async fn test_get_users_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getUsers", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getUsers has no required params → always returns ok

#[tokio::test]
async fn test_get_user_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("getUser", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// Media Endpoints (binary responses)
// ===========================================================================

#[tokio::test]
async fn test_stream_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx.subsonic_get("stream", &[("f", "json")]).await.unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_download_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("download", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_cover_art_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getCoverArt", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_avatar_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getAvatar", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// Annotation Endpoints
// ===========================================================================

#[tokio::test]
async fn test_get_starred_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getStarred", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getStarred has no required params → always returns ok

#[tokio::test]
async fn test_get_starred2_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getStarred2", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getStarred2 has no required params → always returns ok

#[tokio::test]
async fn test_star_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("star", &[("f", "json"), ("id", "song-1")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// star accepts optional ids → calling with ids is fine; calling with no ids works too

#[tokio::test]
async fn test_unstar_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("unstar", &[("f", "json"), ("id", "song-1")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_set_rating_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (_artist_id, _album_id, track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get(
            "setRating",
            &[
                ("f", "json"),
                ("id", &track_id.to_string()),
                ("rating", "4"),
            ],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_set_rating_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("setRating", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_scrobble_happy() {
    let ctx = TestContext::new().await.unwrap();
    let (_artist_id, _album_id, track_id) = seed_test_hierarchy(&ctx).await.unwrap();
    let resp = ctx
        .subsonic_get(
            "scrobble",
            &[
                ("f", "json"),
                ("id", &track_id.to_string()),
                ("submission", "true"),
            ],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_scrobble_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("scrobble", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_now_playing_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getNowPlaying", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getNowPlaying has no required params → always returns ok

#[tokio::test]
async fn test_get_random_songs_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getRandomSongs", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getRandomSongs has no required params → always returns ok

#[tokio::test]
async fn test_get_lyrics_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get(
            "getLyrics",
            &[("f", "json"), ("artist", "Test"), ("title", "Song")],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getLyrics has no required params → always returns ok (even if no lyrics found)

// ===========================================================================
// Scanning Endpoints
// ===========================================================================

#[tokio::test]
async fn test_get_scan_status_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getScanStatus", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getScanStatus has no required params → always returns ok

#[tokio::test]
async fn test_start_scan_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("startScan", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// startScan has no required params → always returns ok

// ===========================================================================
// Other Endpoints — Videos, Captions, HLS, Jukebox, Chat, Podcasts
// ===========================================================================

#[tokio::test]
async fn test_get_videos_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getVideos", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getVideos has no required params → always returns ok

#[tokio::test]
async fn test_get_captions_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getCaptions", &[("f", "json"), ("id", "video-1")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_get_captions_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getCaptions", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// hls returns HTTP 501, not JSON — test raw response
#[tokio::test]
async fn test_hls_returns_not_implemented() {
    let ctx = TestContext::new().await.unwrap();
    let url = ctx.subsonic_url("hls", &[("id", "1")]);
    let resp = ctx.client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 501);
}

// hls error: also returns 501 (no special error path)

#[tokio::test]
async fn test_jukebox_control_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("jukeboxControl", &[("f", "json"), ("action", "status")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// jukeboxControl defaults action to "status" → always returns ok (no error case easily triggered)

#[tokio::test]
async fn test_get_chat_messages_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getChatMessages", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getChatMessages has no required params → always returns ok

#[tokio::test]
async fn test_add_chat_message_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("addChatMessage", &[("f", "json"), ("message", "Hello")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_add_chat_message_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("addChatMessage", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_get_podcasts_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getPodcasts", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getPodcasts has no required params → always returns ok

#[tokio::test]
async fn test_get_newest_podcasts_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("getNewestPodcasts", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// getNewestPodcasts has no required params → always returns ok

#[tokio::test]
async fn test_refresh_podcasts_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("refreshPodcasts", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// refreshPodcasts has no required params → always returns ok

#[tokio::test]
async fn test_create_podcast_channel_happy() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get(
            "createPodcastChannel",
            &[("f", "json"), ("url", "https://example.com/feed.xml")],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_create_podcast_channel_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("createPodcastChannel", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_delete_podcast_channel_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("deletePodcastChannel", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_delete_podcast_episode_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("deletePodcastEpisode", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

#[tokio::test]
async fn test_download_podcast_episode_error() {
    let ctx = TestContext::new().await.unwrap();
    let resp = ctx
        .subsonic_get("downloadPodcastEpisode", &[("f", "json")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "failed");
}

// ===========================================================================
// Playlist Lifecycle Tests (happy paths for CRUD)
// ===========================================================================

#[tokio::test]
async fn test_get_playlist_happy() {
    let ctx = TestContext::new().await.unwrap();
    // Create playlist via API
    let create_resp = ctx
        .subsonic_get("createPlaylist", &[("f", "json"), ("name", "ReadTest")])
        .await
        .unwrap();
    assert_eq!(create_resp["subsonic-response"]["status"], "ok");
    let playlist_id = create_resp["subsonic-response"]["playlist"]["id"]
        .as_str()
        .unwrap();

    // Get the playlist
    let resp = ctx
        .subsonic_get("getPlaylist", &[("f", "json"), ("id", playlist_id)])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
    assert_eq!(resp["subsonic-response"]["playlist"]["name"], "ReadTest");
}

#[tokio::test]
async fn test_update_playlist_happy() {
    let ctx = TestContext::new().await.unwrap();
    // Create playlist
    let create_resp = ctx
        .subsonic_get("createPlaylist", &[("f", "json"), ("name", "UpdateTest")])
        .await
        .unwrap();
    let playlist_id = create_resp["subsonic-response"]["playlist"]["id"]
        .as_str()
        .unwrap();

    // Update the playlist
    let resp = ctx
        .subsonic_get(
            "updatePlaylist",
            &[
                ("f", "json"),
                ("playlistId", playlist_id),
                ("name", "UpdatedName"),
            ],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");

    // Verify the update took effect
    let get_resp = ctx
        .subsonic_get("getPlaylist", &[("f", "json"), ("id", playlist_id)])
        .await
        .unwrap();
    assert_eq!(
        get_resp["subsonic-response"]["playlist"]["name"],
        "UpdatedName"
    );
}

#[tokio::test]
async fn test_delete_playlist_happy() {
    let ctx = TestContext::new().await.unwrap();
    // Create playlist
    let create_resp = ctx
        .subsonic_get("createPlaylist", &[("f", "json"), ("name", "DeleteTest")])
        .await
        .unwrap();
    let playlist_id = create_resp["subsonic-response"]["playlist"]["id"]
        .as_str()
        .unwrap();

    // Delete the playlist
    let resp = ctx
        .subsonic_get("deletePlaylist", &[("f", "json"), ("id", playlist_id)])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

// ===========================================================================
// User — happy path
// ===========================================================================

#[tokio::test]
async fn test_get_user_happy() {
    let ctx = TestContext::new().await.unwrap();
    // Create a user via the API
    let create_resp = ctx
        .subsonic_get(
            "createUser",
            &[
                ("f", "json"),
                ("username", "happyuser"),
                ("password", "secret"),
                ("email", "happy@test.com"),
                ("adminRole", "false"),
            ],
        )
        .await
        .unwrap();
    assert_eq!(create_resp["subsonic-response"]["status"], "ok");

    // Get the user
    let resp = ctx
        .subsonic_get("getUser", &[("f", "json"), ("username", "happyuser")])
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
    assert_eq!(resp["subsonic-response"]["user"]["username"], "happyuser");
}

// ===========================================================================
// Media Happy Path — seed file and test binary responses
// ===========================================================================

#[tokio::test]
async fn test_stream_happy() {
    let ctx = TestContext::new().await.unwrap();
    let track_id = Uuid::new_v4();
    let file_path = format!("/music/stream-{}.mp3", track_id);
    let audio_data = b"fake mp3 audio data for streaming test";

    // Seed track with file path and write the file to VFS
    let track = Track {
        id: track_id,
        title: "StreamTest".into(),
        album_id: None,
        artist_id: None,
        duration: 30,
        file_path: file_path.clone(),
        file_size: audio_data.len() as u64,
        bitrate: 128,
        format: "mp3".into(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Test".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_file: None,
        byte_offset_start: None,
        byte_offset_end: None,
        cue_path: None,
        is_cue_virtual: None,
    };
    ctx.storage().save_track(&track).await.unwrap();
    ctx.storage()
        .write_file(&file_path, audio_data)
        .await
        .unwrap();

    let url = ctx.subsonic_url("stream", &[("id", &track_id.to_string())]);
    let resp = ctx.client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("audio/"));
}

#[tokio::test]
async fn test_download_happy() {
    let ctx = TestContext::new().await.unwrap();
    let track_id = Uuid::new_v4();
    let file_path = format!("/music/download-{}.mp3", track_id);
    let audio_data = b"fake mp3 audio data for download test";

    let track = Track {
        id: track_id,
        title: "DownloadTest".into(),
        album_id: None,
        artist_id: None,
        duration: 30,
        file_path: file_path.clone(),
        file_size: audio_data.len() as u64,
        bitrate: 128,
        format: "mp3".into(),
        track_number: Some(1),
        disc_number: Some(1),
        year: Some(2024),
        genre: Some("Test".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        source_file: None,
        byte_offset_start: None,
        byte_offset_end: None,
        cue_path: None,
        is_cue_virtual: None,
    };
    ctx.storage().save_track(&track).await.unwrap();
    ctx.storage()
        .write_file(&file_path, audio_data)
        .await
        .unwrap();

    let url = ctx.subsonic_url("download", &[("id", &track_id.to_string())]);
    let resp = ctx.client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    assert!(resp
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("attachment"));
}

#[tokio::test]
async fn test_get_cover_art_happy() {
    let ctx = TestContext::new().await.unwrap();
    let artist = make_test_artist();
    ctx.storage().save_artist(&artist).await.unwrap();

    let cover_path = format!("/covers/cover-{}.jpg", artist.id);
    let image_data = b"fake jpeg image data";

    let album = Album {
        id: Uuid::new_v4(),
        name: "CoverArtAlbum".into(),
        artist_id: Some(artist.id),
        year: Some(2024),
        genre: Some("Rock".into()),
        cover_art_path: Some(cover_path.clone()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    ctx.storage().save_album(&album).await.unwrap();
    ctx.storage()
        .write_file(&cover_path, image_data)
        .await
        .unwrap();

    // Look up cover art by album ID
    let url = ctx.subsonic_url("getCoverArt", &[("id", &album.id.to_string())]);
    let resp = ctx.client.get(&url).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("image/"));
}

// ===========================================================================
// Podcast lifecycle — happy paths for create → delete
// ===========================================================================

#[tokio::test]
async fn test_delete_podcast_channel_happy() {
    let ctx = TestContext::new().await.unwrap();
    // Create a channel first, then delete it
    let _create = ctx
        .subsonic_get(
            "createPodcastChannel",
            &[
                ("f", "json"),
                ("url", "https://example.com/feed.xml"),
                ("title", "ToDelete"),
            ],
        )
        .await
        .unwrap();
    // delete by id (mock storage uses sequential "channel-N" ids)
    let resp = ctx
        .subsonic_get(
            "deletePodcastChannel",
            &[("f", "json"), ("id", "channel-1")],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_delete_podcast_episode_happy() {
    let ctx = TestContext::new().await.unwrap();
    // deletePodcastEpisode by ID - uses pre-seeded episode-2 (separate from
    // episode-1 used by download test, to avoid races in parallel execution)
    let resp = ctx
        .subsonic_get(
            "deletePodcastEpisode",
            &[("f", "json"), ("id", "episode-2")],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}

#[tokio::test]
async fn test_download_podcast_episode_happy() {
    let ctx = TestContext::new().await.unwrap();
    // downloadPodcastEpisode by ID - mock storage uses pre-seeded episode-1
    // (real implementation would stream the file)
    let resp = ctx
        .subsonic_get(
            "downloadPodcastEpisode",
            &[("f", "json"), ("id", "episode-1")],
        )
        .await
        .unwrap();
    assert_eq!(resp["subsonic-response"]["status"], "ok");
}
