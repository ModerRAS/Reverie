//! Subsonic API tests

#[cfg(test)]
use super::create_router;
#[cfg(test)]
use axum::body::Body;
#[cfg(test)]
use http::Request;
#[cfg(test)]
use tower::ServiceExt;

#[tokio::test]
async fn test_ping_returns_ok() {
    let router = create_router();
    let response = router
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_license() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getLicense")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_music_folders() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getMusicFolders")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_genres() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getGenres")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_scan_status() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getScanStatus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_start_scan() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/startScan")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_indexes() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getIndexes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_artists() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getArtists")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_playlists() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getPlaylists")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_music_directory() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getMusicDirectory")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_artist() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getArtist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_album() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getAlbum")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_song() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getSong")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_album_info() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getAlbumInfo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_artist_info() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getArtistInfo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_search2() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/search2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_search3() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/search3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_playlist() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getPlaylist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_create_playlist() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/createPlaylist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_update_playlist() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/updatePlaylist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_delete_playlist() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/deletePlaylist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_stream() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/stream")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_download() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_cover_art() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getCoverArt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_avatar() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getAvatar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_user() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getUser")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_users() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getUsers")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_starred() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getStarred")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_starred2() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getStarred2")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_star() {
    let router = create_router();
    let response = router
        .oneshot(Request::builder().uri("/star").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_unstar() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/unstar")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_set_rating() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/setRating")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_scrobble() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/scrobble")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_now_playing() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getNowPlaying")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_lyrics() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getLyrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_get_random_songs() {
    let router = create_router();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/getRandomSongs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
}
