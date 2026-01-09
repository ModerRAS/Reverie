//! Mock Subsonic Storage 实现

use reverie_core::{SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndex, SubsonicArtistIndexes, SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre, SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying, SubsonicPlaylist, SubsonicPlaylistWithSongs, SubsonicPlayQueue, SubsonicScanStatus, SubsonicShare, SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser, MediaFile};
use reverie_storage::{error::StorageError, SubsonicStorage, FileStorage, FileMetadata};
use std::fmt;

type Result<T> = std::result::Result<T, StorageError>;

/// 用于测试的模拟存储
#[derive(Clone)]
pub struct MockSubsonicStorage;

impl MockSubsonicStorage {
    pub fn new() -> Self {
        MockSubsonicStorage
    }
}

impl fmt::Debug for MockSubsonicStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MockSubsonicStorage")
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

    async fn get_music_directory(
        &self,
        _id: &str,
    ) -> Result<Option<SubsonicDirectory>> {
        Ok(None)
    }

    async fn get_artists(
        &self,
        _music_folder_id: Option<i32>,
    ) -> Result<SubsonicArtistIndexes> {
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

    async fn get_similar_songs(
        &self,
        _id: &str,
        _count: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_similar_songs2(
        &self,
        _id: &str,
        _count: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        Ok(vec![])
    }

    async fn get_top_songs(
        &self,
        _artist: &str,
        _count: Option<i32>,
    ) -> Result<SubsonicTopSongs> {
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

    async fn get_starred(
        &self,
        _music_folder_id: Option<i32>,
    ) -> Result<SubsonicStarred> {
        Ok(SubsonicStarred {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    async fn get_starred2(
        &self,
        _music_folder_id: Option<i32>,
    ) -> Result<SubsonicStarred> {
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

    async fn get_playlists(
        &self,
        _username: Option<&str>,
    ) -> Result<Vec<SubsonicPlaylist>> {
        Ok(vec![])
    }

    async fn get_playlist(
        &self,
        _id: &str,
    ) -> Result<Option<SubsonicPlaylistWithSongs>> {
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

    async fn get_stream_path(&self, _id: &str) -> Result<Option<String>> {
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

    async fn get_lyrics_by_song_id(
        &self,
        _id: &str,
    ) -> Result<Vec<SubsonicStructuredLyrics>> {
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

    async fn get_internet_radio_stations(
        &self,
    ) -> Result<Vec<SubsonicInternetRadioStation>> {
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

    async fn get_user(&self, _username: &str) -> Result<Option<SubsonicUser>> {
        Ok(Some(SubsonicUser {
            username: "admin".to_string(),
            email: None,
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
            video_conversion_role: false,
            avatar_last_changed: None,
            folders: vec![1],
        }))
    }

    async fn get_users(&self) -> Result<Vec<SubsonicUser>> {
        Ok(vec![])
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
}

#[async_trait::async_trait]
impl FileStorage for MockSubsonicStorage {
    async fn read_file(&self, _path: &str) -> Result<Vec<u8>> {
        Ok(vec![0, 1, 2, 3]) // 返回一些虚拟数据
    }

    async fn write_file(&self, _path: &str, _data: &[u8]) -> Result<()> {
        Ok(())
    }

    async fn file_exists(&self, _path: &str) -> Result<bool> {
        Ok(true)
    }

    async fn delete_file(&self, _path: &str) -> Result<()> {
        Ok(())
    }

    async fn list_files(&self, _path: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_file_metadata(&self, _path: &str) -> Result<FileMetadata> {
        Ok(FileMetadata {
            size: 1024,
            modified: std::time::SystemTime::now(),
            is_file: true,
            is_dir: false,
        })
    }
}
