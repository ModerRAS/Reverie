use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
};
use http::header::AUTHORIZATION;
use reverie_storage::SubsonicStorage;
use std::sync::Arc;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub username: String,
    pub is_admin: bool,
    pub token: Option<String>,
    pub salt: Option<String>,
}

pub async fn auth_middleware<S: SubsonicStorage>(
    State(storage): State<Arc<S>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (i32, String)> {
    let auth_context = extract_auth(&storage, &req).await?;

    req.extensions_mut().insert(auth_context);

    Ok(next.run(req).await)
}

async fn extract_auth<S: SubsonicStorage>(
    storage: &Arc<S>,
    req: &Request<Body>,
) -> Result<AuthContext, (i32, String)> {
    let query = req.uri().query().unwrap_or("");

    let mut username = None;
    let mut password = None;
    let mut token = None;
    let mut salt = None;

    for pair in query.split('&') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0];
            let value = urlencoding::decode(parts[1]).unwrap_or(parts[1].to_string());

            match key {
                "u" => username = Some(value),
                "p" => password = Some(value),
                "t" => token = Some(value),
                "s" => salt = Some(value),
                _ => {}
            }
        }
    }

    if let Some(ref user) = username {
        if let Some(ref pw) = password {
            let user_result = storage.get_user(user).await;

            match user_result {
                Ok(Some(user)) => {
                    if let Some(ref stored_password) = user.password {
                        if pw == *stored_password || pw == format!("enc:{}", stored_password) {
                            return Ok(AuthContext {
                                username: user.username,
                                is_admin: user.admin_role.unwrap_or(false),
                                token: None,
                                salt: None,
                            });
                        }
                    }
                    return Err((40, "Invalid username or password".to_string()));
                }
                Ok(None) => return Err((40, "User not found".to_string())),
                Err(e) => return Err((0, format!("Database error: {}", e))),
            }
        }

        if let (Some(ref tk), Some(ref sl)) = (token, salt) {
            let user_result = storage.get_user(user).await;

            match user_result {
                Ok(Some(user)) => {
                    let expected_token = format!("{}{}", user.password.unwrap_or_default(), sl);
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(expected_token);
                    let result = format!("{:x}", hasher.finalize());

                    if &result == tk {
                        return Ok(AuthContext {
                            username: user.username,
                            is_admin: user.admin_role.unwrap_or(false),
                            token: Some(tk.clone()),
                            salt: Some(sl.clone()),
                        });
                    }
                    return Err((40, "Invalid token".to_string()));
                }
                Ok(None) => return Err((40, "User not found".to_string())),
                Err(e) => return Err((0, format!("Database error: {}", e))),
            }
        }

        return Err((10, "Missing authentication".to_string()));
    }

    let auth_header = req.headers().get(AUTHORIZATION);

    if let Some(auth) = auth_header {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                let user_result = storage.get_user("admin").await;

                match user_result {
                    Ok(Some(user)) => {
                        return Ok(AuthContext {
                            username: user.username,
                            is_admin: user.admin_role.unwrap_or(false),
                            token: Some(token.to_string()),
                            salt: None,
                        });
                    }
                    _ => return Err((40, "Invalid token".to_string())),
                }
            }
        }
    }

    Err((10, "Missing authentication".to_string()))
}

pub fn get_auth_context(req: &Request) -> Option<&AuthContext> {
    req.extensions().get()
}

pub fn get_username(req: &Request) -> Option<String> {
    get_auth_context(req).map(|ctx| ctx.username.clone())
}

pub fn is_admin(req: &Request) -> bool {
    get_auth_context(req).map(|ctx| ctx.is_admin).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reverie_storage::{SubsonicUser, UserStorage};
    use async_trait::async_trait;
    use reverie_storage::Storage;

    #[derive(Clone)]
    struct MockStorage;

    #[async_trait]
    impl SubsonicStorage for MockStorage {
        async fn get_music_folders(&self) -> Result<Vec<reverie_core::SubsonicMusicFolder>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_indexes(&self, _: Option<i32>, _: Option<i64>) -> Result<reverie_core::SubsonicArtistIndexes, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_genres(&self) -> Result<Vec<reverie_core::SubsonicGenre>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_music_directory(&self, _: &str) -> Result<Option<reverie_core::SubsonicDirectory>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_artists(&self, _: Option<i32>) -> Result<reverie_core::SubsonicArtistIndexes, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_artist(&self, _: &str) -> Result<Option<reverie_core::SubsonicArtist>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_album(&self, _: &str) -> Result<Option<reverie_core::SubsonicAlbum>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_song(&self, _: &str) -> Result<Option<reverie_core::MediaFile>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_artist_info(&self, _: &str, _: Option<i32>, _: Option<bool>) -> Result<reverie_core::SubsonicArtistInfo, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_artist_info2(&self, _: &str, _: Option<i32>, _: Option<bool>) -> Result<reverie_core::SubsonicArtistInfo, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_album_info(&self, _: &str) -> Result<reverie_core::SubsonicAlbumInfo, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_album_info2(&self, _: &str) -> Result<reverie_core::SubsonicAlbumInfo, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_similar_songs(&self, _: &str, _: Option<i32>) -> Result<Vec<reverie_core::MediaFile>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_similar_songs2(&self, _: &str, _: Option<i32>) -> Result<Vec<reverie_core::MediaFile>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_top_songs(&self, _: &str, _: Option<i32>) -> Result<reverie_core::SubsonicTopSongs, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_album_list(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<&str>, _: Option<i32>) -> Result<Vec<reverie_core::SubsonicAlbum>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_album_list2(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<&str>, _: Option<i32>) -> Result<Vec<reverie_core::SubsonicAlbum>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_random_songs(&self, _: Option<i32>, _: Option<&str>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<Vec<reverie_core::MediaFile>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_songs_by_genre(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<Vec<reverie_core::MediaFile>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_now_playing(&self) -> Result<Vec<reverie_core::SubsonicNowPlaying>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_starred(&self, _: Option<i32>) -> Result<reverie_core::SubsonicStarred, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_starred2(&self, _: Option<i32>) -> Result<reverie_core::SubsonicStarred, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn search2(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<reverie_core::SubsonicSearchResult2, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn search3(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<reverie_core::SubsonicSearchResult3, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn get_playlists(&self, _: Option<&str>) -> Result<Vec<reverie_core::SubsonicPlaylist>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_playlist(&self, _: &str) -> Result<Option<reverie_core::SubsonicPlaylistWithSongs>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn create_playlist(&self, _: Option<&str>, _: Option<&str>, _: &[&str]) -> Result<reverie_core::SubsonicPlaylistWithSongs, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn update_playlist(&self, _: &str, _: Option<&str>, _: Option<&str>, _: Option<bool>, _: &[&str], _: &[i32]) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn delete_playlist(&self, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_stream_path(&self, _: &str) -> Result<Option<String>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_cover_art_path(&self, _: &str) -> Result<Option<String>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_lyrics(&self, _: Option<&str>, _: Option<&str>) -> Result<Option<reverie_core::SubsonicLyrics>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_lyrics_by_song_id(&self, _: &str) -> Result<Vec<reverie_core::SubsonicStructuredLyrics>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn get_avatar_path(&self, _: &str) -> Result<Option<String>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn star(&self, _: &[&str], _: &[&str], _: &[&str]) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn unstar(&self, _: &[&str], _: &[&str], _: &[&str]) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn set_rating(&self, _: &str, _: i32) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn scrobble(&self, _: &str, _: Option<i64>, _: bool) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_bookmarks(&self) -> Result<Vec<reverie_core::SubsonicBookmark>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn create_bookmark(&self, _: &str, _: i64, _: Option<&str>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn delete_bookmark(&self, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_play_queue(&self) -> Result<Option<reverie_core::SubsonicPlayQueue>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn save_play_queue(&self, _: &[&str], _: Option<&str>, _: Option<i64>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_shares(&self) -> Result<Vec<reverie_core::SubsonicShare>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn create_share(&self, _: &[&str], _: Option<&str>, _: Option<i64>) -> Result<reverie_core::SubsonicShare, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn update_share(&self, _: &str, _: Option<&str>, _: Option<i64>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn delete_share(&self, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_internet_radio_stations(&self) -> Result<Vec<reverie_core::SubsonicInternetRadioStation>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn create_internet_radio_station(&self, _: &str, _: &str, _: Option<&str>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn update_internet_radio_station(&self, _: &str, _: &str, _: &str, _: Option<&str>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn delete_internet_radio_station(&self, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_user(&self, _: &str) -> Result<Option<reverie_core::SubsonicUser>, reverie_storage::error::StorageError> {
            Ok(None)
        }
        async fn get_users(&self) -> Result<Vec<reverie_core::SubsonicUser>, reverie_storage::error::StorageError> {
            Ok(vec![])
        }
        async fn create_user(&self, _: &str, _: &str, _: Option<&str>, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: &[i32]) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn update_user(&self, _: &str, _: Option<&str>, _: Option<&str>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<i32>) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn delete_user(&self, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn change_password(&self, _: &str, _: &str) -> Result<(), reverie_storage::error::StorageError> {
            Ok(())
        }
        async fn get_scan_status(&self) -> Result<reverie_core::SubsonicScanStatus, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
        async fn start_scan(&self) -> Result<reverie_core::SubsonicScanStatus, reverie_storage::error::StorageError> {
            Ok(Default::default())
        }
    }

    #[tokio::test]
    async fn test_missing_auth() {
        let storage = Arc::new(MockStorage);
        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();

        let result = extract_auth(&storage, &req).await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().0, 10);
    }

    #[tokio::test]
    async fn test_get_auth_context_helper() {
        let ctx = AuthContext {
            username: "testuser".to_string(),
            is_admin: true,
            token: Some("token123".to_string()),
            salt: Some("salt456".to_string()),
        };

        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();
        let mut extensions = req.extensions_mut();
        extensions.insert(ctx);

        let extracted = get_auth_context(&req);
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap().username, "testuser");
        assert!(extracted.unwrap().is_admin);
    }

    #[tokio::test]
    async fn test_get_username_helper() {
        let ctx = AuthContext {
            username: "admin".to_string(),
            is_admin: false,
            token: None,
            salt: None,
        };

        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();
        let mut extensions = req.extensions_mut();
        extensions.insert(ctx);

        let username = get_username(&req);
        assert_eq!(username, Some("admin".to_string()));
    }

    #[tokio::test]
    async fn test_get_username_no_context() {
        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();

        let username = get_username(&req);
        assert!(username.is_none());
    }

    #[tokio::test]
    async fn test_is_admin_helper() {
        let admin_ctx = AuthContext {
            username: "admin".to_string(),
            is_admin: true,
            token: None,
            salt: None,
        };

        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();
        let mut extensions = req.extensions_mut();
        extensions.insert(admin_ctx);

        assert!(is_admin(&req));

        let user_ctx = AuthContext {
            username: "user".to_string(),
            is_admin: false,
            token: None,
            salt: None,
        };

        let req2 = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();
        let mut extensions2 = req2.extensions_mut();
        extensions2.insert(user_ctx);

        assert!(!is_admin(&req2));
    }

    #[tokio::test]
    async fn test_is_admin_no_context() {
        let req = Request::builder()
            .uri("/rest/ping")
            .body(Body::empty())
            .unwrap();

        assert!(!is_admin(&req));
    }

    #[tokio::test]
    async fn test_auth_context_clone() {
        let ctx = AuthContext {
            username: "test".to_string(),
            is_admin: false,
            token: Some("tok".to_string()),
            salt: Some("sal".to_string()),
        };

        let cloned = ctx.clone();
        assert_eq!(ctx.username, cloned.username);
        assert_eq!(ctx.is_admin, cloned.is_admin);
        assert_eq!(ctx.token, cloned.token);
        assert_eq!(ctx.salt, cloned.salt);
    }

    #[test]
    fn test_subsonic_api_version_constant() {
        assert_eq!(SUBSONIC_API_VERSION, "1.16.1");
    }
}
