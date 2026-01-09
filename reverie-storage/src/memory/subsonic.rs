//! FileStorage + SubsonicStorage 实现

use crate::error::Result;
use crate::traits::*;

use async_trait::async_trait;
use chrono::Utc;
use reverie_core::{
    MediaFile, SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndexes,
    SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre,
    SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying,
    SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs, SubsonicScanStatus,
    SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare, SubsonicStarred,
    SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser,
};

use super::core::MemoryStorage;

#[async_trait]
impl FileStorage for MemoryStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self.files.read().await;
        files
            .get(path)
            .cloned()
            .ok_or_else(|| crate::error::StorageError::NotFound(path.to_string()))
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
            .ok_or_else(|| crate::error::StorageError::NotFound(path.to_string()))
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
        self.get_album_list(list_type, size, offset, from_year, to_year, genre, music_folder_id)
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

    // === Searching ===
    async fn search2(
        &self,
        query: &str,
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
        query: &str,
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

    // === User ===
    async fn get_user(&self, _username: &str) -> Result<Option<SubsonicUser>> {
        Ok(Some(SubsonicUser {
            username: "test".to_string(),
            email: None,
            stream_role: true,
            jukebox_role: true,
            download_role: true,
            upload_role: true,
            playlist_role: true,
            cover_art_role: true,
            comment_role: true,
            podcast_role: true,
            share_role: true,
            video_conversion_role: true,
            avatar_last_changed: None,
            folders: vec![],
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

    // === Playlists ===
    async fn get_playlists(&self, _username: Option<&str>) -> Result<Vec<SubsonicPlaylist>> {
        Ok(vec![])
    }

    async fn get_playlist(&self, id: &str) -> Result<Option<SubsonicPlaylistWithSongs>> {
        Ok(Some(SubsonicPlaylistWithSongs {
            id: id.to_string(),
            name: "Test Playlist".to_string(),
            comment: None,
            owner: "test".to_string(),
            public: false,
            song_count: 0,
            duration: 0,
            cover_art: None,
            created: Utc::now(),
            changed: Utc::now(),
            entries: vec![],
        }))
    }

    async fn create_playlist(
        &self,
        _name: Option<&str>,
        _playlist_id: Option<&str>,
        _song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs> {
        Ok(SubsonicPlaylistWithSongs {
            id: "new".to_string(),
            name: "New Playlist".to_string(),
            comment: None,
            owner: "test".to_string(),
            public: false,
            song_count: 0,
            duration: 0,
            cover_art: None,
            created: Utc::now(),
            changed: Utc::now(),
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

    // === Media ===
    async fn get_stream_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(None)
    }

    async fn get_cover_art_path(&self, _id: &str) -> Result<Option<String>> {
        Ok(None)
    }

    // === Lyrics ===
    async fn get_lyrics(
        &self,
        _artist: Option<&str>,
        _title: Option<&str>,
    ) -> Result<Option<SubsonicLyrics>> {
        Ok(Some(SubsonicLyrics {
            value: "".to_string(),
            artist: None,
            title: None,
        }))
    }

    async fn get_lyrics_by_song_id(&self, _id: &str) -> Result<Vec<SubsonicStructuredLyrics>> {
        Ok(vec![])
    }

    async fn get_avatar_path(&self, _username: &str) -> Result<Option<String>> {
        Ok(None)
    }

    // === Annotation ===
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

    // === Play Queue ===
    async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>> {
        Ok(Some(SubsonicPlayQueue {
            current: None,
            position: 0,
            username: "test".to_string(),
            changed: Utc::now(),
            changed_by: "test".to_string(),
            entries: vec![],
        }))
    }

    async fn save_play_queue(
        &self,
        _ids: &[&str],
        _current: Option<&str>,
        _position: Option<i64>,
    ) -> Result<()> {
        Ok(())
    }

    // === Shares ===
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
            url: "http://localhost:8080/share/share-1".to_string(),
            description: None,
            username: "test".to_string(),
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

    // === Now Playing ===
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
        Ok(SubsonicStarred {
            artists: vec![],
            albums: vec![],
            songs: vec![],
        })
    }

    // === Scanning ===
    async fn get_scan_status(&self) -> Result<SubsonicScanStatus> {
        Ok(SubsonicScanStatus {
            scanning: false,
            count: 0,
            last_scan: None,
            elapsed_time: Some(0),
            error: None,
            folder_count: 0,
            scan_type: None,
        })
    }

    async fn start_scan(&self) -> Result<SubsonicScanStatus> {
        Ok(SubsonicScanStatus {
            scanning: true,
            count: 0,
            last_scan: None,
            elapsed_time: Some(0),
            error: None,
            folder_count: 0,
            scan_type: None,
        })
    }
}
