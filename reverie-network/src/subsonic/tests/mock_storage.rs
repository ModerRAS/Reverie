//! Mock Subsonic Storage 实现

use reverie_core::{
    Caption, ChatMessage, JukeboxStatus, MediaFile, PodcastChannel, PodcastEpisode, SubsonicAlbum,
    SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndex, SubsonicArtistIndexes,
    SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre,
    SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying,
    SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs, SubsonicScanStatus,
    SubsonicShare, SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser,
    Track, VideoInfo,
};
use reverie_storage::{
    error::StorageError, FileMetadata, FileStorage, SubsonicStorage, TrackStorage,
};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;
use uuid::Uuid;

type Result<T> = std::result::Result<T, StorageError>;

/// 用于测试的模拟存储
#[derive(Clone)]
pub struct MockSubsonicStorage {
    pub users: Arc<RwLock<HashMap<String, SubsonicUser>>>,
    passwords: Arc<RwLock<HashMap<String, String>>>,
    chat_messages: Arc<TokioRwLock<Vec<ChatMessage>>>,
    podcast_channels: Arc<TokioRwLock<Vec<PodcastChannel>>>,
    podcast_episodes: Arc<TokioRwLock<Vec<PodcastEpisode>>>,
    /// Tracks for TrackStorage (including CUE fields)
    pub tracks: Arc<RwLock<HashMap<Uuid, Track>>>,
    /// File data for FileStorage (path -> bytes)
    pub file_data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockSubsonicStorage {
    pub fn new() -> Self {
        MockSubsonicStorage {
            users: Arc::new(RwLock::new(HashMap::new())),
            passwords: Arc::new(RwLock::new(HashMap::new())),
            chat_messages: Arc::new(TokioRwLock::new(vec![
                ChatMessage {
                    username: "admin".to_string(),
                    message: "Welcome to the chat!".to_string(),
                    time: 1704067200000,
                },
                ChatMessage {
                    username: "user1".to_string(),
                    message: "Hello everyone!".to_string(),
                    time: 1704067201000,
                },
            ])),
            podcast_channels: Arc::new(TokioRwLock::new(vec![])),
            podcast_episodes: Arc::new(TokioRwLock::new(vec![PodcastEpisode {
                id: "episode-1".to_string(),
                channel_id: "channel-1".to_string(),
                title: "Test Episode 1".to_string(),
                description: None,
                publish_date: None,
                status: "completed".to_string(),
                stream_id: None,
                duration: None,
                size: None,
                url: None,
                cover_art: None,
            }])),
            tracks: Arc::new(RwLock::new(HashMap::new())),
            file_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn get_user_internal(&self, username: &str) -> Option<SubsonicUser> {
        self.users.read().ok()?.get(username).cloned()
    }

    fn get_password_internal(&self, username: &str) -> Option<String> {
        self.passwords.read().ok()?.get(username).cloned()
    }

    fn insert_user(&self, user: &SubsonicUser, password_hash: String) {
        if let Ok(mut users) = self.users.write() {
            users.insert(user.username.clone(), user.clone());
        }
        if let Ok(mut passwords) = self.passwords.write() {
            passwords.insert(user.username.clone(), password_hash);
        }
    }

    fn remove_user(&self, username: &str) {
        if let Ok(mut users) = self.users.write() {
            users.remove(username);
        }
        if let Ok(mut passwords) = self.passwords.write() {
            passwords.remove(username);
        }
    }
}

impl Default for MockSubsonicStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MockSubsonicStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockSubsonicStorage")
    }
}

#[async_trait::async_trait]
impl TrackStorage for MockSubsonicStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let tracks = self
            .tracks
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        Ok(tracks.get(&id).cloned())
    }

    async fn list_tracks(&self, _limit: usize, _offset: usize) -> Result<Vec<Track>> {
        let tracks = self
            .tracks
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        Ok(tracks.values().cloned().collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        let mut tracks = self
            .tracks
            .write()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        tracks.insert(track.id, track.clone());
        Ok(())
    }

    async fn delete_track(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn search_tracks(&self, _query: &str) -> Result<Vec<Track>> {
        Ok(vec![])
    }

    async fn get_tracks_by_album(&self, _album_id: Uuid) -> Result<Vec<Track>> {
        Ok(vec![])
    }

    async fn get_tracks_by_artist(&self, _artist_id: Uuid) -> Result<Vec<Track>> {
        Ok(vec![])
    }
}

#[async_trait::async_trait]
impl SubsonicStorage for MockSubsonicStorage {
    // === System ===
    async fn get_license(&self) -> Result<bool> {
        Ok(true)
    }

    // === Browsing ===
    async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>> {
        Ok(vec![SubsonicMusicFolder {
            id: 1,
            name: "Music".to_string(),
        }])
    }

    async fn get_indexes(
        &self,
        _music_folder_id: Option<i32>,
        _if_modified_since: Option<i64>,
    ) -> Result<SubsonicArtistIndexes> {
        Ok(vec![])
    }

    async fn get_genres(&self) -> Result<Vec<SubsonicGenre>> {
        Ok(vec![])
    }

    async fn get_music_directory(&self, _id: &str) -> Result<Option<SubsonicDirectory>> {
        Ok(None)
    }

    async fn get_artists(&self, _music_folder_id: Option<i32>) -> Result<SubsonicArtistIndexes> {
        Ok(vec![SubsonicArtistIndex {
            id: "A".to_string(),
            artists: vec![SubsonicArtist {
                id: "artist-1".to_string(),
                name: "Test Artist".to_string(),
                cover_art: None,
                album_count: 1,
                starred: None,
                user_rating: None,
            }],
        }])
    }

    async fn get_artist(&self, _id: &str) -> Result<Option<SubsonicArtist>> {
        Ok(Some(SubsonicArtist {
            id: "artist-1".to_string(),
            name: "Test Artist".to_string(),
            cover_art: None,
            album_count: 1,
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_album(&self, _id: &str) -> Result<Option<SubsonicAlbum>> {
        Ok(Some(SubsonicAlbum {
            id: "album-1".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("artist-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            cover_art: None,
            song_count: 10,
            duration: 3600.0,
            play_count: None,
            created: None,
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_song(&self, _id: &str) -> Result<Option<MediaFile>> {
        Ok(Some(MediaFile::default()))
    }

    async fn get_songs_by_album(&self, _album_id: &str) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_video_info(&self, id: &str) -> Result<Option<VideoInfo>> {
        if id == "video-1" {
            Ok(Some(VideoInfo {
                id: "video-1".to_string(),
                title: "Test Video".to_string(),
                path: Some("/videos/test.mp4".to_string()),
                cover_art: Some("cover-1".to_string()),
                original_width: Some(1920),
                original_height: Some(1080),
                audio_track_id: Some("audio-1".to_string()),
                duration: Some(3600),
                bit_rate: Some(2000),
                created: Some(chrono::Utc::now()),
            }))
        } else {
            Ok(None) // Return None for unknown IDs (for not found test)
        }
    }

    async fn get_captions(&self, _id: &str, _format: Option<&str>) -> Result<Vec<Caption>> {
        Ok(vec![])
    }

    async fn get_artist_info(
        &self,
        _id: &str,
        _count: Option<i32>,
        _include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo> {
        Ok(SubsonicArtistInfo {
            biography: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
            similar_artists: vec![],
        })
    }

    async fn get_artist_info2(
        &self,
        _id: &str,
        _count: Option<i32>,
        _include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo> {
        self.get_artist_info(_id, _count, _include_not_present)
            .await
    }

    async fn get_album_info(&self, _id: &str) -> Result<SubsonicAlbumInfo> {
        Ok(SubsonicAlbumInfo {
            notes: None,
            music_brainz_id: None,
            last_fm_url: None,
            small_image_url: None,
            medium_image_url: None,
            large_image_url: None,
        })
    }

    async fn get_album_info2(&self, _id: &str) -> Result<SubsonicAlbumInfo> {
        self.get_album_info(_id).await
    }

    async fn get_similar_songs(&self, _id: &str, _count: Option<i32>) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_similar_songs2(&self, _id: &str, _count: Option<i32>) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_top_songs(&self, _artist: &str, _count: Option<i32>) -> Result<SubsonicTopSongs> {
        Ok(SubsonicTopSongs { songs: vec![] })
    }

    async fn get_album_list(
        &self,
        _list_type: &str,
        _size: Option<i32>,
        _offset: Option<i32>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _genre: Option<&str>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>> {
        Ok(vec![])
    }

    async fn get_album_list2(
        &self,
        _list_type: &str,
        _size: Option<i32>,
        _offset: Option<i32>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _genre: Option<&str>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>> {
        Ok(vec![SubsonicAlbum {
            id: "album-1".to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("artist-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            year: Some(2024),
            genre: Some("Rock".to_string()),
            cover_art: None,
            song_count: 10,
            duration: 3600.0,
            play_count: None,
            created: None,
            starred: None,
            user_rating: None,
        }])
    }

    async fn get_random_songs(
        &self,
        _size: Option<i32>,
        _genre: Option<&str>,
        _from_year: Option<i32>,
        _to_year: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_songs_by_genre(
        &self,
        _genre: &str,
        _count: Option<i32>,
        _offset: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_now_playing(&self) -> Result<Vec<SubsonicNowPlaying>> {
        Ok(vec![])
    }

    async fn get_starred(&self, _music_folder_id: Option<i32>) -> Result<SubsonicStarred> {
        Ok(SubsonicStarred {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn get_starred2(&self, _music_folder_id: Option<i32>) -> Result<SubsonicStarred> {
        self.get_starred(_music_folder_id).await
    }

    async fn search2(
        &self,
        _query: &str,
        _artist_count: Option<i32>,
        _artist_offset: Option<i32>,
        _album_count: Option<i32>,
        _album_offset: Option<i32>,
        _song_count: Option<i32>,
        _song_offset: Option<i32>,
    ) -> Result<reverie_core::SubsonicSearchResult2> {
        Ok(reverie_core::SubsonicSearchResult2 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn search3(
        &self,
        _query: &str,
        _artist_count: Option<i32>,
        _artist_offset: Option<i32>,
        _album_count: Option<i32>,
        _album_offset: Option<i32>,
        _song_count: Option<i32>,
        _song_offset: Option<i32>,
    ) -> Result<reverie_core::SubsonicSearchResult3> {
        Ok(reverie_core::SubsonicSearchResult3 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn get_playlists(&self, _username: Option<&str>) -> Result<Vec<SubsonicPlaylist>> {
        Ok(vec![])
    }

    async fn get_playlist(&self, _id: &str) -> Result<Option<SubsonicPlaylistWithSongs>> {
        Ok(None)
    }

    async fn create_playlist(
        &self,
        _name: Option<&str>,
        _playlist_id: Option<&str>,
        _song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs> {
        Ok(SubsonicPlaylistWithSongs {
            id: "playlist-1".to_string(),
            name: "Test Playlist".to_string(),
            comment: None,
            owner: "admin".to_string(),
            public: false,
            song_count: 0,
            duration: 0,
            created: chrono::Utc::now(),
            changed: chrono::Utc::now(),
            cover_art: None,
            entries: vec![],
        })
    }

    async fn update_playlist(
        &self,
        _playlist_id: &str,
        _name: Option<&str>,
        _comment: Option<&str>,
        _public: Option<bool>,
        _song_ids_to_add: &[&str],
        _song_indexes_to_remove: &[i32],
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_playlist(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_stream_path(&self, id: &str) -> Result<Option<String>> {
        // Try to look up the track by UUID to get its file_path
        if let Ok(uuid) = Uuid::parse_str(id) {
            if let Ok(tracks) = self.tracks.read() {
                if let Some(track) = tracks.get(&uuid) {
                    return Ok(Some(track.file_path.clone()));
                }
            }
        }
        // Fallback for tests that don't register tracks
        Ok(Some("/music/test.mp3".to_string()))
    }

    async fn get_cover_art_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(Some("/covers/test.jpg".to_string()))
    }

    async fn get_lyrics(
        &self,
        _artist: Option<&str>,
        _title: Option<&str>,
    ) -> Result<Option<SubsonicLyrics>> {
        Ok(None)
    }

    async fn get_lyrics_by_song_id(&self, _id: &str) -> Result<Vec<SubsonicStructuredLyrics>> {
        Ok(vec![])
    }

    async fn get_avatar_path(&self, _username: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn star(&self, _ids: &[&str], _album_ids: &[&str], _artist_ids: &[&str]) -> Result<()> {
        Ok(())
    }

    async fn unstar(&self, _ids: &[&str], _album_ids: &[&str], _artist_ids: &[&str]) -> Result<()> {
        Ok(())
    }

    async fn set_rating(&self, _id: &str, _rating: i32) -> Result<()> {
        Ok(())
    }

    async fn scrobble(&self, _id: &str, _time: Option<i64>, _submission: bool) -> Result<()> {
        Ok(())
    }

    async fn get_bookmarks(&self) -> Result<Vec<SubsonicBookmark>> {
        Ok(vec![])
    }

    async fn create_bookmark(
        &self,
        _id: &str,
        _position: i64,
        _comment: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_bookmark(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>> {
        Ok(None)
    }

    async fn save_play_queue(
        &self,
        _ids: &[&str],
        _current: Option<&str>,
        _position: Option<i64>,
    ) -> Result<()> {
        Ok(())
    }

    async fn get_shares(&self) -> Result<Vec<SubsonicShare>> {
        Ok(vec![])
    }

    async fn create_share(
        &self,
        _ids: &[&str],
        _description: Option<&str>,
        _expires: Option<i64>,
    ) -> Result<SubsonicShare> {
        Ok(SubsonicShare {
            id: "share-1".to_string(),
            url: "http://example.com/share/1".to_string(),
            description: None,
            username: "admin".to_string(),
            created: chrono::Utc::now(),
            expires: None,
            last_visited: None,
            visit_count: 0,
            entries: vec![],
        })
    }

    async fn update_share(
        &self,
        _id: &str,
        _description: Option<&str>,
        _expires: Option<i64>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_share(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_internet_radio_stations(&self) -> Result<Vec<SubsonicInternetRadioStation>> {
        Ok(vec![])
    }

    async fn create_internet_radio_station(
        &self,
        _stream_url: &str,
        _name: &str,
        _homepage_url: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    async fn update_internet_radio_station(
        &self,
        _id: &str,
        _stream_url: &str,
        _name: &str,
        _homepage_url: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_internet_radio_station(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>> {
        Ok(self.get_user_internal(username))
    }

    async fn get_users(&self) -> Result<Vec<SubsonicUser>> {
        let users = self
            .users
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        Ok(users.values().cloned().collect())
    }

    async fn create_user(
        &self,
        username: &str,
        password: &str,
        email: Option<&str>,
        admin_role: bool,
        settings_role: bool,
        stream_role: bool,
        jukebox_role: bool,
        download_role: bool,
        upload_role: bool,
        playlist_role: bool,
        cover_art_role: bool,
        comment_role: bool,
        podcast_role: bool,
        share_role: bool,
        video_conversion_role: bool,
        music_folder_ids: &[i32],
    ) -> Result<()> {
        // Check for duplicate
        if self.get_user_internal(username).is_some() {
            return Err(StorageError::Unavailable("User already exists".to_string()));
        }

        // Hash the password
        let hashed_password = reverie_core::hash_password(password)
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        let user = SubsonicUser {
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            scrobbling_enabled: true,
            max_bit_rate: None,
            admin_role,
            settings_role,
            download_role,
            upload_role,
            playlist_role,
            cover_art_role,
            comment_role,
            podcast_role,
            stream_role,
            jukebox_role,
            share_role,
            video_conversion_role,
            avatar_last_changed: None,
            folders: music_folder_ids.to_vec(),
        };

        self.insert_user(&user, hashed_password);
        Ok(())
    }

    async fn update_user(
        &self,
        username: &str,
        password: Option<&str>,
        email: Option<&str>,
        admin_role: Option<bool>,
        settings_role: Option<bool>,
        stream_role: Option<bool>,
        jukebox_role: Option<bool>,
        download_role: Option<bool>,
        upload_role: Option<bool>,
        playlist_role: Option<bool>,
        cover_art_role: Option<bool>,
        comment_role: Option<bool>,
        podcast_role: Option<bool>,
        share_role: Option<bool>,
        video_conversion_role: Option<bool>,
        music_folder_ids: Option<&[i32]>,
        max_bit_rate: Option<i32>,
    ) -> Result<()> {
        let user = self
            .get_user_internal(username)
            .ok_or(StorageError::NotFound(format!(
                "User {} not found",
                username
            )))?;

        let updated_email = email.map(|s| s.to_string()).or(user.email.clone());

        let updated_user = SubsonicUser {
            username: user.username.clone(),
            email: updated_email,
            scrobbling_enabled: user.scrobbling_enabled,
            max_bit_rate: max_bit_rate.or(user.max_bit_rate),
            admin_role: admin_role.unwrap_or(user.admin_role),
            settings_role: settings_role.unwrap_or(user.settings_role),
            download_role: download_role.unwrap_or(user.download_role),
            upload_role: upload_role.unwrap_or(user.upload_role),
            playlist_role: playlist_role.unwrap_or(user.playlist_role),
            cover_art_role: cover_art_role.unwrap_or(user.cover_art_role),
            comment_role: comment_role.unwrap_or(user.comment_role),
            podcast_role: podcast_role.unwrap_or(user.podcast_role),
            stream_role: stream_role.unwrap_or(user.stream_role),
            jukebox_role: jukebox_role.unwrap_or(user.jukebox_role),
            share_role: share_role.unwrap_or(user.share_role),
            video_conversion_role: video_conversion_role.unwrap_or(user.video_conversion_role),
            avatar_last_changed: user.avatar_last_changed,
            folders: music_folder_ids
                .map(|ids| ids.to_vec())
                .unwrap_or(user.folders),
        };

        // Get existing password hash
        let existing_password = self.get_password_internal(username).unwrap_or_default();

        if let Ok(mut users) = self.users.write() {
            users.insert(username.to_string(), updated_user);
        }

        // Update password if provided
        if let Some(new_password) = password {
            let hashed_password = reverie_core::hash_password(new_password)
                .map_err(|e| StorageError::Unavailable(e.to_string()))?;
            if let Ok(mut passwords) = self.passwords.write() {
                passwords.insert(username.to_string(), hashed_password);
            }
        } else {
            // Restore existing password hash
            if let Ok(mut passwords) = self.passwords.write() {
                passwords.insert(username.to_string(), existing_password);
            }
        }

        Ok(())
    }

    async fn delete_user(&self, username: &str) -> Result<()> {
        if self.get_user_internal(username).is_none() {
            return Err(StorageError::NotFound(format!(
                "User {} not found",
                username
            )));
        }
        self.remove_user(username);
        Ok(())
    }

    async fn change_password(&self, username: &str, password: &str) -> Result<()> {
        if self.get_user_internal(username).is_none() {
            return Err(StorageError::NotFound(format!(
                "User {} not found",
                username
            )));
        }

        let hashed_password = reverie_core::hash_password(password)
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;

        if let Ok(mut passwords) = self.passwords.write() {
            passwords.insert(username.to_string(), hashed_password);
        }

        Ok(())
    }

    async fn get_scan_status(&self) -> Result<SubsonicScanStatus> {
        Ok(SubsonicScanStatus {
            scanning: false,
            count: 100,
            folder_count: 1,
            last_scan: None,
            error: None,
            scan_type: None,
            elapsed_time: None,
        })
    }

    async fn start_scan(&self) -> Result<SubsonicScanStatus> {
        self.get_scan_status().await
    }

    async fn jukebox_control(
        &self,
        _action: &str,
        _index: Option<i32>,
        _offset: Option<i32>,
        _timeout: Option<i32>,
    ) -> Result<JukeboxStatus> {
        Ok(JukeboxStatus::default())
    }

    async fn get_chat_messages(&self, since: Option<i64>) -> Result<Vec<ChatMessage>> {
        let messages = self.chat_messages.read().await;
        let filtered: Vec<ChatMessage> = messages
            .iter()
            .filter(|m| since.map_or(true, |t| m.time > t))
            .cloned()
            .collect();
        Ok(filtered)
    }

    async fn add_chat_message(&self, message: &str) -> Result<()> {
        let chat_msg = ChatMessage {
            username: "admin".to_string(),
            message: message.to_string(),
            time: chrono::Utc::now().timestamp_millis(),
        };
        self.chat_messages.write().await.push(chat_msg);
        Ok(())
    }

    // === Podcasts ===
    async fn get_podcasts(&self, _include_episodes: bool) -> Result<Vec<PodcastChannel>> {
        Ok(vec![])
    }

    async fn get_newest_podcasts(&self, _count: Option<i32>) -> Result<Vec<PodcastEpisode>> {
        Ok(vec![])
    }

    async fn refresh_podcasts(&self) -> Result<()> {
        Ok(())
    }

    async fn create_podcast_channel(
        &self,
        url: &str,
        title: Option<&str>,
    ) -> Result<PodcastChannel> {
        let channel = PodcastChannel::new(
            format!("channel-{}", self.podcast_channels.read().await.len() + 1),
            url.to_string(),
            title.unwrap_or("New Podcast").to_string(),
        );
        self.podcast_channels.write().await.push(channel.clone());
        Ok(channel)
    }

    async fn delete_podcast_channel(&self, id: &str) -> Result<()> {
        let mut channels = self.podcast_channels.write().await;
        let len_before = channels.len();
        channels.retain(|c| c.id != id);
        if channels.len() == len_before {
            return Err(StorageError::NotFound(
                "Podcast channel not found".to_string(),
            ));
        }
        Ok(())
    }

    async fn delete_podcast_episode(&self, id: &str) -> Result<()> {
        let mut episodes = self.podcast_episodes.write().await;
        let len_before = episodes.len();
        episodes.retain(|e| e.id != id);
        if episodes.len() == len_before {
            return Err(StorageError::NotFound(
                "Podcast episode not found".to_string(),
            ));
        }
        Ok(())
    }

    async fn get_podcast_episode_path(&self, id: &str) -> Result<Option<String>> {
        // Return a path for known episodes
        if id == "episode-1" {
            Ok(Some("/mock/podcasts/episode-1.mp3".to_string()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl FileStorage for MockSubsonicStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self
            .file_data
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    async fn read_file_range(&self, path: &str, offset: u64, size: u64) -> Result<Vec<u8>> {
        let data = self.read_file(path).await?;
        let start = offset as usize;
        let end = std::cmp::min(start + size as usize, data.len());
        if start >= data.len() {
            return Err(StorageError::NotFound(format!(
                "Range start {} exceeds file size {}: {}",
                offset,
                data.len(),
                path
            )));
        }
        Ok(data[start..end].to_vec())
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let mut files = self
            .file_data
            .write()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        let files = self
            .file_data
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        Ok(files.contains_key(path))
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let mut files = self
            .file_data
            .write()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        files.remove(path);
        Ok(())
    }

    async fn list_files(&self, _path: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let files = self
            .file_data
            .read()
            .map_err(|e| StorageError::Unavailable(e.to_string()))?;
        let data = files
            .get(path)
            .ok_or_else(|| StorageError::NotFound(path.to_string()))?;
        Ok(FileMetadata {
            size: data.len() as u64,
            modified: std::time::SystemTime::now(),
            is_file: true,
            is_dir: false,
        })
    }
}
