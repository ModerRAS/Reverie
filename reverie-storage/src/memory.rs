//! In-memory storage implementation for testing and development

use crate::error::{Result, StorageError};
use crate::traits::*;

use async_trait::async_trait;
use chrono::Utc;
use reverie_core::{
    Album, Artist, MediaFile, Playlist, PlaylistTrack, SubsonicAlbum, SubsonicAlbumInfo,
    SubsonicArtist, SubsonicArtistIndexes, SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory,
    SubsonicGenre, SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder,
    SubsonicNowPlaying, SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs,
    SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare,
    SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser, Track, User,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// In-memory storage implementation using HashMaps
#[derive(Clone)]
pub struct MemoryStorage {
    tracks: Arc<RwLock<HashMap<Uuid, Track>>>,
    albums: Arc<RwLock<HashMap<Uuid, Album>>>,
    artists: Arc<RwLock<HashMap<Uuid, Artist>>>,
    users: Arc<RwLock<HashMap<Uuid, User>>>,
    playlists: Arc<RwLock<HashMap<Uuid, Playlist>>>,
    playlist_tracks: Arc<RwLock<HashMap<Uuid, Vec<PlaylistTrack>>>>,
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(HashMap::new())),
            albums: Arc::new(RwLock::new(HashMap::new())),
            artists: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            playlists: Arc::new(RwLock::new(HashMap::new())),
            playlist_tracks: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TrackStorage for MemoryStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.get(&id).cloned())
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks.values().skip(offset).take(limit).cloned().collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        let mut tracks = self.tracks.write().await;
        tracks.insert(track.id, track.clone());
        Ok(())
    }

    async fn delete_track(&self, id: Uuid) -> Result<()> {
        let mut tracks = self.tracks.write().await;
        tracks.remove(&id);
        Ok(())
    }

    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        let query_lower = query.to_lowercase();
        Ok(tracks
            .values()
            .filter(|t| t.title.to_lowercase().contains(&query_lower))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks
            .values()
            .filter(|t| t.album_id == Some(album_id))
            .cloned()
            .collect())
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let tracks = self.tracks.read().await;
        Ok(tracks
            .values()
            .filter(|t| t.artist_id == Some(artist_id))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl AlbumStorage for MemoryStorage {
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.get(&id).cloned())
    }

    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        let albums = self.albums.read().await;
        Ok(albums.values().skip(offset).take(limit).cloned().collect())
    }

    async fn save_album(&self, album: &Album) -> Result<()> {
        let mut albums = self.albums.write().await;
        albums.insert(album.id, album.clone());
        Ok(())
    }

    async fn delete_album(&self, id: Uuid) -> Result<()> {
        let mut albums = self.albums.write().await;
        albums.remove(&id);
        Ok(())
    }

    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>> {
        let albums = self.albums.read().await;
        Ok(albums
            .values()
            .filter(|a| a.artist_id == Some(artist_id))
            .cloned()
            .collect())
    }
}

#[async_trait]
impl ArtistStorage for MemoryStorage {
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>> {
        let artists = self.artists.read().await;
        Ok(artists.get(&id).cloned())
    }

    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>> {
        let artists = self.artists.read().await;
        Ok(artists.values().skip(offset).take(limit).cloned().collect())
    }

    async fn save_artist(&self, artist: &Artist) -> Result<()> {
        let mut artists = self.artists.write().await;
        artists.insert(artist.id, artist.clone());
        Ok(())
    }

    async fn delete_artist(&self, id: Uuid) -> Result<()> {
        let mut artists = self.artists.write().await;
        artists.remove(&id);
        Ok(())
    }
}

#[async_trait]
impl UserStorage for MemoryStorage {
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.get(&id).cloned())
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let users = self.users.read().await;
        Ok(users.values().find(|u| u.username == username).cloned())
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>> {
        let users = self.users.read().await;
        Ok(users.values().skip(offset).take(limit).cloned().collect())
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        let mut users = self.users.write().await;
        users.insert(user.id, user.clone());
        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        let mut users = self.users.write().await;
        users.remove(&id);
        Ok(())
    }
}

#[async_trait]
impl PlaylistStorage for MemoryStorage {
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>> {
        let playlists = self.playlists.read().await;
        Ok(playlists.get(&id).cloned())
    }

    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>> {
        let playlists = self.playlists.read().await;
        Ok(playlists
            .values()
            .filter(|p| p.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        let mut playlists = self.playlists.write().await;
        playlists.insert(playlist.id, playlist.clone());
        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<()> {
        let mut playlists = self.playlists.write().await;
        playlists.remove(&id);
        Ok(())
    }

    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()> {
        let mut playlist_tracks = self.playlist_tracks.write().await;
        playlist_tracks
            .entry(playlist_track.playlist_id)
            .or_insert_with(Vec::new)
            .push(playlist_track.clone());
        Ok(())
    }

    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()> {
        let mut playlist_tracks = self.playlist_tracks.write().await;
        if let Some(tracks) = playlist_tracks.get_mut(&playlist_id) {
            tracks.retain(|pt| pt.track_id != track_id);
        }
        Ok(())
    }

    async fn get_playlist_tracks(&self, playlist_id: Uuid) -> Result<Vec<PlaylistTrack>> {
        let playlist_tracks = self.playlist_tracks.read().await;
        Ok(playlist_tracks
            .get(&playlist_id)
            .cloned()
            .unwrap_or_default())
    }
}

#[async_trait]
impl FileStorage for MemoryStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self.files.read().await;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let mut files = self.files.write().await;
        files.insert(path.to_string(), data.to_vec());
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        let files = self.files.read().await;
        Ok(files.contains_key(path))
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        let mut files = self.files.write().await;
        files.remove(path);
        Ok(())
    }

    async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let files = self.files.read().await;
        Ok(files
            .keys()
            .filter(|k| k.starts_with(path))
            .cloned()
            .collect())
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let files = self.files.read().await;
        files
            .get(path)
            .ok_or_else(|| StorageError::NotFound(path.to_string()))
            .map(|data| FileMetadata {
                size: data.len() as u64,
                modified: std::time::SystemTime::now(),
                is_file: true,
                is_dir: false,
            })
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }
}

#[async_trait]
impl SubsonicStorage for MemoryStorage {
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
        Ok(vec![
            SubsonicGenre {
                name: "Rock".to_string(),
                song_count: 10,
                album_count: 5,
            },
            SubsonicGenre {
                name: "Pop".to_string(),
                song_count: 8,
                album_count: 4,
            },
        ])
    }

    async fn get_music_directory(&self, id: &str) -> Result<Option<SubsonicDirectory>> {
        Ok(Some(SubsonicDirectory {
            id: id.to_string(),
            parent: None,
            name: "Test Directory".to_string(),
            artist: None,
            artist_id: None,
            cover_art: None,
            child_count: Some(0),
            album_count: None,
            duration: None,
            play_count: None,
            starred: None,
            user_rating: None,
            children: vec![],
        }))
    }

    async fn get_artists(&self, _music_folder_id: Option<i32>) -> Result<SubsonicArtistIndexes> {
        Ok(vec![])
    }

    async fn get_artist(&self, id: &str) -> Result<Option<SubsonicArtist>> {
        Ok(Some(SubsonicArtist {
            id: id.to_string(),
            name: "Test Artist".to_string(),
            cover_art: Some("ar-123".to_string()),
            album_count: 5,
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_album(&self, id: &str) -> Result<Option<SubsonicAlbum>> {
        Ok(Some(SubsonicAlbum {
            id: id.to_string(),
            name: "Test Album".to_string(),
            album_artist: Some("Test Artist".to_string()),
            album_artist_id: Some("artist-1".to_string()),
            artist: Some("Test Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            cover_art: Some("al-123".to_string()),
            song_count: 10,
            duration: 2400.0,
            play_count: None,
            created: Some(Utc::now()),
            starred: None,
            user_rating: None,
        }))
    }

    async fn get_song(&self, id: &str) -> Result<Option<MediaFile>> {
        Ok(Some(MediaFile {
            id: id.to_string(),
            parent: Some("album-1".to_string()),
            is_dir: false,
            title: "Test Song".to_string(),
            album: Some("Test Album".to_string()),
            artist: Some("Test Artist".to_string()),
            album_artist: Some("Test Artist".to_string()),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            cover_art: Some("al-123".to_string()),
            size: 5000000,
            content_type: "audio/mpeg".to_string(),
            suffix: "mp3".to_string(),
            duration: 240.0,
            bit_rate: 320,
            sample_rate: 44100,
            bit_depth: None,
            channels: Some(2),
            path: "/music/test/song.mp3".to_string(),
            play_count: None,
            created: Some(Utc::now()),
            starred: None,
            album_id: Some("album-1".to_string()),
            artist_id: Some("artist-1".to_string()),
            r#type: "music".to_string(),
            user_rating: None,
            library_id: 1,
            missing: false,
        }))
    }

    async fn get_artist_info(
        &self,
        _id: &str,
        _count: Option<i32>,
        _include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo> {
        Ok(SubsonicArtistInfo {
            biography: Some("Test biography".to_string()),
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
        id: &str,
        count: Option<i32>,
        include_not_present: Option<bool>,
    ) -> Result<SubsonicArtistInfo> {
        self.get_artist_info(id, count, include_not_present).await
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

    async fn get_album_info2(&self, id: &str) -> Result<SubsonicAlbumInfo> {
        self.get_album_info(id).await
    }

    async fn get_similar_songs(&self, _id: &str, _count: Option<i32>) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_similar_songs2(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>> {
        self.get_similar_songs(id, count).await
    }

    async fn get_top_songs(&self, _artist: &str, _count: Option<i32>) -> Result<SubsonicTopSongs> {
        Ok(SubsonicTopSongs { songs: vec![] })
    }

    // === Album/Song Lists ===
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
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>> {
        self.get_album_list(
            list_type,
            size,
            offset,
            from_year,
            to_year,
            genre,
            music_folder_id,
        )
        .await
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

    async fn get_starred2(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred> {
        self.get_starred(music_folder_id).await
    }

    // === Searching ===
    async fn search2(
        &self,
        _query: &str,
        _artist_count: Option<i32>,
        _artist_offset: Option<i32>,
        _album_count: Option<i32>,
        _album_offset: Option<i32>,
        _song_count: Option<i32>,
        _song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult2> {
        Ok(SubsonicSearchResult2 {
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
    ) -> Result<SubsonicSearchResult3> {
        Ok(SubsonicSearchResult3 {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    // === Playlists ===
    async fn get_playlists(&self, _username: Option<&str>) -> Result<Vec<SubsonicPlaylist>> {
        Ok(vec![])
    }

    async fn get_playlist(&self, _id: &str) -> Result<Option<SubsonicPlaylistWithSongs>> {
        Ok(None)
    }

    async fn create_playlist(
        &self,
        name: Option<&str>,
        _playlist_id: Option<&str>,
        _song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs> {
        Ok(SubsonicPlaylistWithSongs {
            id: Uuid::new_v4().to_string(),
            name: name.unwrap_or("New Playlist").to_string(),
            comment: None,
            owner: "admin".to_string(),
            public: false,
            song_count: 0,
            duration: 0,
            created: Utc::now(),
            changed: Utc::now(),
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

    // === Media Retrieval ===
    async fn get_stream_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_cover_art_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(None)
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

    // === Media Annotation ===
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

    // === Bookmarks ===
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

    // === Sharing ===
    async fn get_shares(&self) -> Result<Vec<SubsonicShare>> {
        Ok(vec![])
    }

    async fn create_share(
        &self,
        _ids: &[&str],
        description: Option<&str>,
        _expires: Option<i64>,
    ) -> Result<SubsonicShare> {
        Ok(SubsonicShare {
            id: Uuid::new_v4().to_string(),
            url: format!("https://example.com/share/{}", Uuid::new_v4()),
            description: description.map(String::from),
            username: "admin".to_string(),
            created: Utc::now(),
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

    // === Internet Radio ===
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

    // === User Management ===
    async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>> {
        Ok(Some(SubsonicUser {
            username: username.to_string(),
            email: None,
            scrobbling_enabled: true,
            max_bit_rate: None,
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
            folders: vec![1],
        }))
    }

    async fn get_users(&self) -> Result<Vec<SubsonicUser>> {
        Ok(vec![SubsonicUser {
            username: "admin".to_string(),
            email: Some("admin@example.com".to_string()),
            scrobbling_enabled: true,
            max_bit_rate: None,
            admin_role: true,
            settings_role: true,
            download_role: true,
            upload_role: true,
            playlist_role: true,
            cover_art_role: true,
            comment_role: true,
            podcast_role: true,
            stream_role: true,
            jukebox_role: true,
            share_role: true,
            video_conversion_role: true,
            avatar_last_changed: None,
            folders: vec![1],
        }])
    }

    async fn create_user(
        &self,
        _username: &str,
        _password: &str,
        _email: Option<&str>,
        _admin_role: bool,
        _settings_role: bool,
        _stream_role: bool,
        _jukebox_role: bool,
        _download_role: bool,
        _upload_role: bool,
        _playlist_role: bool,
        _cover_art_role: bool,
        _comment_role: bool,
        _podcast_role: bool,
        _share_role: bool,
        _video_conversion_role: bool,
        _music_folder_ids: &[i32],
    ) -> Result<()> {
        Ok(())
    }

    async fn update_user(
        &self,
        _username: &str,
        _password: Option<&str>,
        _email: Option<&str>,
        _admin_role: Option<bool>,
        _settings_role: Option<bool>,
        _stream_role: Option<bool>,
        _jukebox_role: Option<bool>,
        _download_role: Option<bool>,
        _upload_role: Option<bool>,
        _playlist_role: Option<bool>,
        _cover_art_role: Option<bool>,
        _comment_role: Option<bool>,
        _podcast_role: Option<bool>,
        _share_role: Option<bool>,
        _video_conversion_role: Option<bool>,
        _music_folder_ids: Option<&[i32]>,
        _max_bit_rate: Option<i32>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_user(&self, _username: &str) -> Result<()> {
        Ok(())
    }

    async fn change_password(&self, _username: &str, _password: &str) -> Result<()> {
        Ok(())
    }

    // === Library Scanning ===
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
        Ok(SubsonicScanStatus {
            scanning: true,
            count: 0,
            folder_count: 1,
            last_scan: None,
            error: None,
            scan_type: Some("fullScan".to_string()),
            elapsed_time: None,
        })
    }
}
