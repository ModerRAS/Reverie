//! FileStorage 和 SubsonicStorage 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::vfs::VfsConfig;
use crate::DatabaseStorage;
use reverie_core::{MediaFile, SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndexes, SubsonicArtistInfo, SubsonicBookmark, SubsonicDirectory, SubsonicGenre, SubsonicInternetRadioStation, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying, SubsonicOpenSubsonicExtension, SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs, SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare, SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser};

use super::config::DatabaseConfig;

#[async_trait]
impl FileStorage for DatabaseStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let data = self.vfs().read(path).await?;
        Ok(data.to_vec())
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        self.vfs()
            .write(path, bytes::Bytes::copy_from_slice(data))
            .await
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        self.vfs().exists(path).await
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        self.vfs().delete(path).await
    }

    async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let entries = self.vfs().list(path).await?;
        Ok(entries.into_iter().map(|e| e.path).collect())
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let meta = self.vfs().stat(path).await?;
        Ok(FileMetadata {
            size: meta.size,
            modified: meta
                .last_modified
                .map(|dt| dt.into())
                .unwrap_or_else(std::time::SystemTime::now),
            is_file: meta.is_file,
            is_dir: meta.is_dir,
        })
    }
}

impl DatabaseStorage {
    /// 将数据库行转换为 MediaFile
    fn row_to_media_file(&self, r: &Row) -> MediaFile {
        MediaFile {
            id: r.get("id"),
            parent: r.get::<Option<String>, _>("album_id"),
            is_dir: false,
            title: r.get("title"),
            album: r.get("album_name"),
            artist: r.get("artist_name"),
            album_artist: r.get("artist_name"),
            year: r.get::<Option<i32>, _>("year").map(|v| v as i32),
            genre: r.get("genre"),
            cover_art: r.get::<Option<String>, _>("cover_art_path"),
            duration: r.get::<Option<i64>, _>("duration").map(|v| v as f32),
            bit_rate: r.get::<Option<i64>, _>("bitrate").map(|v| v as i32),
            path: r.get("file_path"),
            size: r.get::<Option<i64>, _>("file_size").map(|v| v as i64),
            format: r.get("format"),
            track_number: r.get::<Option<i64>, _>("track_number").map(|v| v as i32),
            disc_number: r.get::<Option<i64>, _>("disc_number").map(|v| v as i32),
            ..Default::default()
        }
    }

    /// 内部方法：获取专辑的歌曲
    async fn get_songs_by_album_internal(&self, album_id: &str) -> Result<Vec<MediaFile>> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.album_id = ? ORDER BY t.disc_number, t.track_number"#,
        )
        .bind(album_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_media_file(r)).collect())
    }

    /// 内部方法：获取艺术家的专辑
    async fn get_albums_by_artist_internal(&self, artist_id: &str) -> Result<Vec<SubsonicAlbum>> {
        let rows = sqlx::query(
            r#"SELECT a.*, ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.artist_id = ? ORDER BY a.year DESC, a.name"#,
        )
        .bind(artist_id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicAlbum {
                id: r.get("id"),
                name: r.get("name"),
                album_artist: r.get("artist_name"),
                album_artist_id: r.get("artist_id"),
                artist: r.get("artist_name"),
                artist_id: r.get("artist_id"),
                year: r.get::<Option<i32>, _>("year"),
                genre: r.get("genre"),
                cover_art: r.get("cover_art_path"),
                song_count: r.get::<Option<i32>, _>("song_count").unwrap_or(0),
                duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
                play_count: r.get::<Option<i64>, _>("play_count"),
                created: r
                    .get::<Option<String>, _>("created_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                starred: None,
                user_rating: None,
            })
            .collect())
    }
}

#[async_trait]
impl SubsonicStorage for DatabaseStorage {
    // === System ===
    async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>> {
        let rows = sqlx::query("SELECT id, name FROM music_folders ORDER BY name")
            .fetch_all(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicMusicFolder {
                id: r.get::<i32, _>("id"),
                name: r.get("name"),
            })
            .collect())
    }

    async fn get_indexes(
        &self,
        _music_folder_id: Option<i32>,
        _if_modified_since: Option<i64>,
    ) -> Result<SubsonicArtistIndexes> {
        let rows = sqlx::query(
            r#"SELECT id, name, 
                      (SELECT COUNT(*) FROM albums WHERE albums.artist_id = artists.id) as album_count
               FROM artists ORDER BY name"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut indexes: std::collections::HashMap<String, Vec<SubsonicArtist>> =
            std::collections::HashMap::new();

        for row in rows {
            let name: String = row.get("name");
            let first_char = name
                .chars()
                .next()
                .unwrap_or('#')
                .to_uppercase()
                .to_string();
            let index_name = if first_char.chars().next().unwrap_or('#').is_alphabetic() {
                first_char
            } else {
                "#".to_string()
            };

            let artist = SubsonicArtist {
                id: row.get("id"),
                name,
                cover_art: None,
                album_count: row.get::<i32, _>("album_count"),
                starred: None,
                user_rating: None,
            };

            indexes.entry(index_name).or_default().push(artist);
        }

        let mut result: SubsonicArtistIndexes = indexes
            .into_iter()
            .map(|(id, artists)| SubsonicArtistIndex { id, artists })
            .collect();
        result.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(result)
    }

    async fn get_genres(&self) -> Result<Vec<SubsonicGenre>> {
        let rows = sqlx::query(
            r#"SELECT genre, COUNT(DISTINCT id) as song_count, 
                      COUNT(DISTINCT album_id) as album_count
               FROM tracks WHERE genre IS NOT NULL 
               GROUP BY genre ORDER BY genre"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicGenre {
                name: r.get("genre"),
                song_count: r.get::<i32, _>("song_count"),
                album_count: r.get::<i32, _>("album_count"),
            })
            .collect())
    }

    async fn get_music_directory(&self, id: &str) -> Result<Option<SubsonicDirectory>> {
        // 首先尝试作为专辑查找
        if let Some(album) = SubsonicStorage::get_album(self, id).await? {
            let songs = self.get_songs_by_album_internal(id).await?;
            return Ok(Some(SubsonicDirectory::from_album(&album, songs)));
        }

        // 尝试作为艺术家
        if let Some(artist) = SubsonicStorage::get_artist(self, id).await? {
            let albums = self.get_albums_by_artist_internal(id).await?;
            let children: Vec<MediaFile> = albums
                .into_iter()
                .map(|a| MediaFile {
                    id: a.id.clone(),
                    parent: Some(artist.id.clone()),
                    is_dir: true,
                    title: a.name.clone(),
                    album: Some(a.name),
                    artist: a.artist,
                    album_artist: a.album_artist,
                    year: a.year,
                    genre: a.genre,
                    cover_art: a.cover_art,
                    duration: a.duration,
                    ..Default::default()
                })
                .collect();

            return Ok(Some(SubsonicDirectory {
                id: artist.id.clone(),
                parent: None,
                name: artist.name,
                artist: None,
                artist_id: None,
                cover_art: artist.cover_art,
                child_count: Some(children.len() as i32),
                album_count: Some(artist.album_count),
                duration: None,
                play_count: None,
                starred: artist.starred,
                user_rating: artist.user_rating,
                children,
            }));
        }

        Ok(None)
    }

    async fn get_artists(&self, _music_folder_id: Option<i32>) -> Result<SubsonicArtistIndexes> {
        self.get_indexes(None, None).await
    }

    async fn get_artist(&self, id: &str) -> Result<Option<SubsonicArtist>> {
        let row = sqlx::query(
            r#"SELECT id, name, image_url, starred_at,
                      (SELECT COUNT(*) FROM albums WHERE albums.artist_id = artists.id) as album_count
               FROM artists WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| SubsonicArtist {
            id: r.get("id"),
            name: r.get("name"),
            cover_art: r.get("image_url"),
            album_count: r.get::<i32, _>("album_count"),
            starred: r
                .get::<Option<String>, _>("starred_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            user_rating: None,
        }))
    }

    async fn get_album(&self, id: &str) -> Result<Option<SubsonicAlbum>> {
        let row = sqlx::query(
            r#"SELECT a.id, a.name, a.artist_id, a.year, a.genre, a.cover_art_path,
                      a.song_count, a.duration, a.play_count, a.starred_at, a.created_at,
                      ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.id = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| SubsonicAlbum {
            id: r.get("id"),
            name: r.get("name"),
            album_artist: r.get("artist_name"),
            album_artist_id: r.get("artist_id"),
            artist: r.get("artist_name"),
            artist_id: r.get("artist_id"),
            year: r.get::<Option<i32>, _>("year"),
            genre: r.get("genre"),
            cover_art: r.get("cover_art_path"),
            song_count: r.get::<Option<i32>, _>("song_count").unwrap_or(0),
            duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
            play_count: r.get::<Option<i64>, _>("play_count"),
            created: r
                .get::<Option<String>, _>("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            starred: r
                .get::<Option<String>, _>("starred_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            user_rating: None,
        }))
    }

    async fn get_song(&self, id: &str) -> Result<Option<MediaFile>> {
        let row = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.id = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| self.row_to_media_file(&r)))
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

    async fn get_similar_songs(&self, _id: &str, count: Option<i32>) -> Result<Vec<MediaFile>> {
        let limit = count.unwrap_or(50);
        self.get_random_songs(Some(limit), None, None, None, None)
            .await
    }

    async fn get_similar_songs2(&self, id: &str, count: Option<i32>) -> Result<Vec<MediaFile>> {
        self.get_similar_songs(id, count).await
    }

    async fn get_top_songs(&self, artist: &str, count: Option<i32>) -> Result<SubsonicTopSongs> {
        let limit = count.unwrap_or(50);
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE ar.name LIKE ?
               ORDER BY t.play_count DESC
               LIMIT ?"#,
        )
        .bind(format!("%{}%", artist))
        .bind(limit)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(SubsonicTopSongs {
            songs: rows.iter().map(|r| self.row_to_media_file(r)).collect(),
        })
    }

    // === Album/Song Lists ===
    async fn get_album_list(
        &self,
        list_type: &str,
        size: Option<i32>,
        offset: Option<i32>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        genre: Option<&str>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<SubsonicAlbum>> {
        let limit = size.unwrap_or(10);
        let off = offset.unwrap_or(0);

        let order_clause = match list_type {
            "newest" => "a.created_at DESC",
            "recent" => "a.updated_at DESC",
            "frequent" => "a.play_count DESC",
            "highest" => "a.play_count DESC",
            "alphabeticalByName" => "a.name ASC",
            "alphabeticalByArtist" => "ar.name ASC, a.name ASC",
            "starred" => "a.starred_at DESC",
            "byYear" => "a.year ASC",
            "byGenre" => "a.genre ASC",
            "random" => "RANDOM()",
            _ => "a.name ASC",
        };

        let mut query = r#"SELECT a.id, a.name, a.artist_id, a.year, a.genre, a.cover_art_path,
                      a.song_count, a.duration, a.play_count, a.starred_at, a.created_at,
                      ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE 1=1"#
            .to_string();

        if let Some(fy) = from_year {
            query.push_str(&format!(" AND a.year >= {}", fy));
        }
        if let Some(ty) = to_year {
            query.push_str(&format!(" AND a.year <= {}", ty));
        }
        if let Some(g) = genre {
            query.push_str(&format!(" AND a.genre = '{}'", g.replace('\'', "''")));
        }

        query.push_str(&format!(
            " ORDER BY {} LIMIT {} OFFSET {}",
            order_clause, limit, off
        ));

        let rows = sqlx::query(&query)
            .fetch_all(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicAlbum {
                id: r.get("id"),
                name: r.get("name"),
                album_artist: r.get("artist_name"),
                album_artist_id: r.get("artist_id"),
                artist: r.get("artist_name"),
                artist_id: r.get("artist_id"),
                year: r.get::<Option<i32>, _>("year"),
                genre: r.get("genre"),
                cover_art: r.get("cover_art_path"),
                song_count: r.get::<Option<i32>, _>("song_count").unwrap_or(0),
                duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
                play_count: r.get::<Option<i64>, _>("play_count"),
                created: r
                    .get::<Option<String>, _>("created_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                starred: r
                    .get::<Option<String>, _>("starred_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                user_rating: None,
            })
            .collect())
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
        size: Option<i32>,
        genre: Option<&str>,
        from_year: Option<i32>,
        to_year: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        let limit = size.unwrap_or(10);

        let mut query = String::from(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE 1=1"#,
        );

        if let Some(g) = genre {
            query.push_str(&format!(" AND t.genre = '{}'", g.replace('\'', "''")));
        }
        if let Some(fy) = from_year {
            query.push_str(&format!(" AND t.year >= {}", fy));
        }
        if let Some(ty) = to_year {
            query.push_str(&format!(" AND t.year <= {}", ty));
        }

        query.push_str(&format!(" ORDER BY RANDOM() LIMIT {}", limit));

        let rows = sqlx::query(&query)
            .fetch_all(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_media_file(r)).collect())
    }

    async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: Option<i32>,
        offset: Option<i32>,
        _music_folder_id: Option<i32>,
    ) -> Result<Vec<MediaFile>> {
        let limit = count.unwrap_or(10);
        let off = offset.unwrap_or(0);

        let rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.genre = ?
               ORDER BY t.title
               LIMIT ? OFFSET ?"#,
        )
        .bind(genre)
        .bind(limit)
        .bind(off)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_media_file(r)).collect())
    }

    async fn get_now_playing(&self) -> Result<Vec<SubsonicNowPlaying>> {
        Ok(vec![])
    }

    // === Starred ===
    async fn get_starred(&self, _music_folder_id: Option<i32>) -> Result<SubsonicStarred> {
        let artist_rows = sqlx::query(
            r#"SELECT id, name, image_url, starred_at,
                      (SELECT COUNT(*) FROM albums WHERE albums.artist_id = artists.id) as album_count
               FROM artists WHERE starred_at IS NOT NULL ORDER BY starred_at DESC"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let album_rows = sqlx::query(
            r#"SELECT a.*, ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.starred_at IS NOT NULL ORDER BY a.starred_at DESC"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let song_rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.starred_at IS NOT NULL ORDER BY t.starred_at DESC"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(SubsonicStarred {
            artists: artist_rows
                .into_iter()
                .map(|r| SubsonicArtist {
                    id: r.get("id"),
                    name: r.get("name"),
                    cover_art: r.get("image_url"),
                    album_count: r.get::<i32, _>("album_count"),
                    starred: r
                        .get::<Option<String>, _>("starred_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                    user_rating: None,
                })
                .collect(),
            albums: album_rows
                .into_iter()
                .map(|r| SubsonicAlbum {
                    id: r.get("id"),
                    name: r.get("name"),
                    album_artist: r.get("artist_name"),
                    album_artist_id: r.get("artist_id"),
                    artist: r.get("artist_name"),
                    artist_id: r.get("artist_id"),
                    year: r.get::<Option<i32>, _>("year"),
                    genre: r.get("genre"),
                    cover_art: r.get("cover_art_path"),
                    song_count: r.get::<Option<i32>, _>("song_count").unwrap_or(0),
                    duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
                    play_count: r.get::<Option<i64>, _>("play_count"),
                    created: None,
                    starred: r
                        .get::<Option<String>, _>("starred_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc)),
                    user_rating: None,
                })
                .collect(),
            songs: song_rows
                .iter()
                .map(|r| self.row_to_media_file(r))
                .collect(),
        })
    }

    async fn get_starred2(&self, music_folder_id: Option<i32>) -> Result<SubsonicStarred> {
        self.get_starred(music_folder_id).await
    }

    // === Searching ===
    async fn search2(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult2> {
        let pattern = format!("%{}%", query);
        let ar_limit = artist_count.unwrap_or(20);
        let ar_off = artist_offset.unwrap_or(0);
        let al_limit = album_count.unwrap_or(20);
        let al_off = album_offset.unwrap_or(0);
        let s_limit = song_count.unwrap_or(20);
        let s_off = song_offset.unwrap_or(0);

        let artists = sqlx::query(
            r#"SELECT id, name, image_url, starred_at,
                      (SELECT COUNT(*) FROM albums WHERE albums.artist_id = artists.id) as album_count
               FROM artists WHERE name LIKE ? ORDER BY name LIMIT ? OFFSET ?"#,
        )
        .bind(&pattern)
        .bind(ar_limit)
        .bind(ar_off)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let albums = sqlx::query(
            r#"SELECT a.*, ar.name as artist_name
               FROM albums a LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.name LIKE ? ORDER BY a.name LIMIT ? OFFSET ?"#,
        )
        .bind(&pattern)
        .bind(al_limit)
        .bind(al_off)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let songs = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.title LIKE ? ORDER BY t.title LIMIT ? OFFSET ?"#,
        )
        .bind(&pattern)
        .bind(s_limit)
        .bind(s_off)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(SubsonicSearchResult2 {
            artists: artists
                .into_iter()
                .map(|r| SubsonicArtist {
                    id: r.get("id"),
                    name: r.get("name"),
                    cover_art: r.get("image_url"),
                    album_count: r.get::<i32, _>("album_count"),
                    starred: None,
                    user_rating: None,
                })
                .collect(),
            albums: albums
                .into_iter()
                .map(|r| SubsonicAlbum {
                    id: r.get("id"),
                    name: r.get("name"),
                    album_artist: r.get("artist_name"),
                    album_artist_id: r.get("artist_id"),
                    artist: r.get("artist_name"),
                    artist_id: r.get("artist_id"),
                    year: r.get::<Option<i32>, _>("year"),
                    genre: r.get("genre"),
                    cover_art: r.get("cover_art_path"),
                    song_count: r.get::<Option<i32>, _>("song_count").unwrap_or(0),
                    duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
                    play_count: None,
                    created: None,
                    starred: None,
                    user_rating: None,
                })
                .collect(),
            songs: songs.iter().map(|r| self.row_to_media_file(r)).collect(),
        })
    }

    async fn search3(
        &self,
        query: &str,
        artist_count: Option<i32>,
        artist_offset: Option<i32>,
        album_count: Option<i32>,
        album_offset: Option<i32>,
        song_count: Option<i32>,
        song_offset: Option<i32>,
    ) -> Result<SubsonicSearchResult3> {
        let result = self
            .search2(
                query,
                artist_count,
                artist_offset,
                album_count,
                album_offset,
                song_count,
                song_offset,
            )
            .await?;
        Ok(SubsonicSearchResult3 {
            artists: result.artists,
            albums: result.albums,
            songs: result.songs,
        })
    }

    // === Playlists ===
    async fn get_playlists(&self, username: Option<&str>) -> Result<Vec<SubsonicPlaylist>> {
        let rows = if let Some(user) = username {
            sqlx::query(
                r#"SELECT p.*, u.username as owner_name,
                          (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as entry_count
                   FROM playlists p LEFT JOIN users u ON p.user_id = u.id
                   WHERE u.username = ? OR p.is_public = 1 ORDER BY p.name"#,
            )
            .bind(user)
            .fetch_all(self.pool())
            .await
        } else {
            sqlx::query(
                r#"SELECT p.*, u.username as owner_name,
                          (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as entry_count
                   FROM playlists p LEFT JOIN users u ON p.user_id = u.id
                   ORDER BY p.name"#,
            )
            .fetch_all(self.pool())
            .await
        }
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicPlaylist {
                id: r.get("id"),
                name: r.get("name"),
                comment: r.get("description"),
                owner: r.get::<Option<String>, _>("owner_name").unwrap_or_default(),
                public: r
                    .get::<Option<i64>, _>("is_public")
                    .map(|v| v == 1)
                    .unwrap_or(false),
                song_count: r.get::<i32, _>("entry_count"),
                duration: 0,
                created: Utc::now(),
                changed: Utc::now(),
                cover_art: r.get("cover_art_path"),
            })
            .collect())
    }

    async fn get_playlist(&self, id: &str) -> Result<Option<SubsonicPlaylistWithSongs>> {
        let row = sqlx::query(
            r#"SELECT p.*, u.username as owner_name
               FROM playlists p LEFT JOIN users u ON p.user_id = u.id WHERE p.id = ?"#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let entries = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM playlist_tracks pt
               JOIN tracks t ON pt.track_id = t.id
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE pt.playlist_id = ? ORDER BY pt.position"#,
        )
        .bind(id)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Some(SubsonicPlaylistWithSongs {
            id: row.get("id"),
            name: row.get("name"),
            comment: row.get("description"),
            owner: row
                .get::<Option<String>, _>("owner_name")
                .unwrap_or_default(),
            public: row
                .get::<Option<i64>, _>("is_public")
                .map(|v| v == 1)
                .unwrap_or(false),
            song_count: entries.len() as i32,
            duration: 0,
            created: Utc::now(),
            changed: Utc::now(),
            cover_art: row.get("cover_art_path"),
            entries: entries.iter().map(|r| self.row_to_media_file(r)).collect(),
        }))
    }

    async fn create_playlist(
        &self,
        name: Option<&str>,
        playlist_id: Option<&str>,
        song_ids: &[&str],
    ) -> Result<SubsonicPlaylistWithSongs> {
        let now = Utc::now().to_rfc3339();
        let id = playlist_id
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let playlist_name = name.unwrap_or("New Playlist");

        sqlx::query(
            "INSERT INTO playlists (id, name, description, user_id, is_public, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(playlist_name)
        .bind("")
        .bind("system") // user_id
        .bind(0i64) // is_public
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        for (pos, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at) VALUES (?, ?, ?, ?)",
            )
            .bind(&id)
            .bind(*song_id)
            .bind(pos as i64)
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        SubsonicStorage::get_playlist(self, &id)
            .await?
            .ok_or_else(|| StorageError::NotFound(format!("Playlist {} not found", id)))
    }

    async fn update_playlist(
        &self,
        playlist_id: &str,
        name: Option<&str>,
        comment: Option<&str>,
        public: Option<bool>,
        song_ids_to_add: &[&str],
        song_indexes_to_remove: &[i32],
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        if let Some(n) = name {
            sqlx::query("UPDATE playlists SET name = ?, updated_at = ? WHERE id = ?")
                .bind(n)
                .bind(&now)
                .bind(playlist_id)
                .execute(self.pool())
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(c) = comment {
            sqlx::query("UPDATE playlists SET description = ?, updated_at = ? WHERE id = ?")
                .bind(c)
                .bind(&now)
                .bind(playlist_id)
                .execute(self.pool())
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(p) = public {
            sqlx::query("UPDATE playlists SET is_public = ?, updated_at = ? WHERE id = ?")
                .bind(if p { 1i64 } else { 0i64 })
                .bind(&now)
                .bind(playlist_id)
                .execute(self.pool())
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        // Add songs
        for song_id in song_ids_to_add {
            let pos: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_tracks WHERE playlist_id = ?",
            )
            .bind(playlist_id)
            .fetch_one(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            sqlx::query(
                "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position, added_at) VALUES (?, ?, ?, ?)",
            )
            .bind(playlist_id)
            .bind(song_id)
            .bind(pos)
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        // Remove songs by index
        for &idx in song_indexes_to_remove {
            sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND position = ?")
                .bind(playlist_id)
                .bind(idx)
                .execute(self.pool())
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn delete_playlist(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === Media Retrieval ===
    async fn get_stream_path(&self, id: &str) -> Result<Option<String>> {
        let row = sqlx::query("SELECT file_path FROM tracks WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.get("file_path")))
    }

    async fn get_cover_art_path(&self, id: &str) -> Result<Option<String>> {
        // First check if it's an album
        let row = sqlx::query("SELECT cover_art_path FROM albums WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(cover) = row {
            return Ok(cover.get("cover_art_path"));
        }

        // Then check if it's a track
        let row = sqlx::query("SELECT cover_art_path FROM tracks WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.get("cover_art_path")))
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

    async fn create_bookmark(&self, _id: &str, _position: i64, _comment: Option<&str>) -> Result<()> {
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
            id: Uuid::new_v4().to_string(),
            url: "".to_string(),
            description: None,
            username: "".to_string(),
            created: Utc::now(),
            expires: None,
            last_visited: None,
            visit_count: 0,
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
        let row = sqlx::query(
            r#"SELECT id, username, email, is_admin, created_at
               FROM users WHERE username = ?"#,
        )
        .bind(username)
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| SubsonicUser {
            scrobbling_enabled: true,
            admin_role: r.get::<i64, _>("is_admin") != 0,
            settings_role: true,
            download_role: false,
            upload_role: false,
            playlist_role: true,
            cover_art_role: false,
            comment_role: false,
            podcast_role: false,
            share_role: false,
            video_conversion_role: false,
            upload_avatar_role: false,
            change_password_role: false,
            max_bit_rate: 0,
            user_id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            avatar_url: None,
            enabled: true,
            folder_ids: vec![],
        }))
    }

    async fn get_users(&self) -> Result<Vec<SubsonicUser>> {
        let rows = sqlx::query(
            r#"SELECT id, username, email, is_admin, created_at
               FROM users ORDER BY username"#,
        )
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicUser {
                scrobbling_enabled: true,
                admin_role: r.get::<i64, _>("is_admin") != 0,
                settings_role: true,
                download_role: false,
                upload_role: false,
                playlist_role: true,
                cover_art_role: false,
                comment_role: false,
                podcast_role: false,
                share_role: false,
                video_conversion_role: false,
                upload_avatar_role: false,
                change_password_role: false,
                max_bit_rate: 0,
                user_id: r.get("id"),
                username: r.get("username"),
                email: r.get("email"),
                avatar_url: None,
                enabled: true,
                folder_ids: vec![],
            })
            .collect())
    }

    async fn create_user(
        &self,
        username: &str,
        password: &str,
        email: Option<&str>,
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
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(username)
        .bind(password)
        .bind(email)
        .bind(0i64)
        .bind(&now)
        .bind(&now)
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_user(
        &self,
        username: &str,
        email: Option<&str>,
        new_password: Option<&str>,
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
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        let mut query = String::from("UPDATE users SET updated_at = ?");
        let mut params: Vec<Box<dyn sqlx::Type<Sqlite> + Send>> = vec![Box::new(now)];

        if let Some(pwd) = new_password {
            query.push_str(", password_hash = ?");
            params.push(Box::new(pwd.to_string()));
        }

        if let Some(mail) = email {
            query.push_str(", email = ?");
            params.push(Box::new(mail.to_string()));
        }

        query.push_str(" WHERE username = ?");
        params.push(Box::new(username.to_string()));

        sqlx::query(&query)
            .bind(&params[0])
            .bind(&params[1])
            .bind(&params[2])
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_user(&self, username: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE username = ?")
            .bind(username)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === Scanning ===
    async fn get_scan_status(&self) -> Result<SubsonicScanStatus> {
        let row = sqlx::query("SELECT * FROM scan_status WHERE id = 1")
            .fetch_optional(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(match row {
            Some(r) => SubsonicScanStatus {
                scanning: r.get::<i32, _>("scanning") == 1,
                count: r.get::<i64, _>("count") as i32,
                folder_count: r.get::<i64, _>("folder_count") as i32,
                last_scan: r
                    .get::<Option<String>, _>("last_scan")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                error: r.get("error"),
            },
            None => SubsonicScanStatus {
                scanning: false,
                count: 0,
                folder_count: 0,
                last_scan: None,
                error: None,
            },
        })
    }

    async fn start_scan(&self, _folder: Option<&str>) -> Result<SubsonicScanStatus> {
        self.get_scan_status().await
    }
}
