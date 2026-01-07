//! Database storage implementation using SQLite
//!
//! This module provides a SQLite-based storage implementation for metadata,
//! while media files are stored in a VFS backend (local filesystem, S3, etc.)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Row, Sqlite};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::vfs::{create_vfs, SharedVfs, VfsConfig};
use reverie_core::{
    Album, Artist, MediaFile, Playlist, PlaylistTrack, SubsonicAlbum, SubsonicAlbumInfo,
    SubsonicArtist, SubsonicArtistIndex, SubsonicArtistIndexes, SubsonicArtistInfo,
    SubsonicBookmark, SubsonicDirectory, SubsonicGenre, SubsonicInternetRadioStation,
    SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying, SubsonicPlayQueue, SubsonicPlaylist,
    SubsonicPlaylistWithSongs, SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3,
    SubsonicShare, SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser,
    Track, User,
};

/// Database storage configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// SQLite database path (e.g., "reverie.db" or ":memory:")
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// VFS configuration for media file storage
    pub vfs_config: VfsConfig,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "reverie.db".to_string(),
            max_connections: 5,
            vfs_config: VfsConfig::local("./music"),
        }
    }
}

impl DatabaseConfig {
    /// Create a new database configuration
    pub fn new(database_url: impl Into<String>, vfs_config: VfsConfig) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 5,
            vfs_config,
        }
    }

    /// Create an in-memory database configuration (for testing)
    pub fn memory() -> Self {
        Self {
            database_url: ":memory:".to_string(),
            max_connections: 1,
            vfs_config: VfsConfig::memory(),
        }
    }
}

/// SQLite-based storage implementation
#[derive(Clone)]
pub struct DatabaseStorage {
    pool: Pool<Sqlite>,
    vfs: SharedVfs,
    config: DatabaseConfig,
}

impl DatabaseStorage {
    /// Create a new database storage
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        let database_url = if config.database_url == ":memory:" {
            "sqlite::memory:".to_string()
        } else {
            format!("sqlite://{}?mode=rwc", config.database_url)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&database_url)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let vfs = create_vfs(config.vfs_config.clone())?;

        let storage = Self { pool, vfs, config };
        storage.run_migrations().await?;

        Ok(storage)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                bio TEXT,
                image_url TEXT,
                starred_at TEXT,
                play_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS albums (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                artist_id TEXT,
                year INTEGER,
                genre TEXT,
                cover_art_path TEXT,
                song_count INTEGER DEFAULT 0,
                duration REAL DEFAULT 0,
                play_count INTEGER DEFAULT 0,
                starred_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (artist_id) REFERENCES artists(id)
            );

            CREATE TABLE IF NOT EXISTS tracks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                album_id TEXT,
                artist_id TEXT,
                duration INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                bitrate INTEGER NOT NULL,
                sample_rate INTEGER DEFAULT 44100,
                channels INTEGER DEFAULT 2,
                format TEXT NOT NULL,
                track_number INTEGER,
                disc_number INTEGER,
                year INTEGER,
                genre TEXT,
                cover_art_path TEXT,
                play_count INTEGER DEFAULT 0,
                starred_at TEXT,
                rating INTEGER,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (album_id) REFERENCES albums(id),
                FOREIGN KEY (artist_id) REFERENCES artists(id)
            );

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                email TEXT,
                is_admin INTEGER NOT NULL DEFAULT 0,
                last_login_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS playlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                user_id TEXT NOT NULL,
                is_public INTEGER NOT NULL DEFAULT 0,
                cover_art_path TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS playlist_tracks (
                playlist_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (playlist_id, track_id),
                FOREIGN KEY (playlist_id) REFERENCES playlists(id),
                FOREIGN KEY (track_id) REFERENCES tracks(id)
            );

            CREATE TABLE IF NOT EXISTS music_folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS genres (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE
            );

            CREATE TABLE IF NOT EXISTS scrobbles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                played_at TEXT NOT NULL,
                FOREIGN KEY (track_id) REFERENCES tracks(id),
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS play_queue (
                user_id TEXT PRIMARY KEY,
                track_ids TEXT NOT NULL,
                current_track_id TEXT,
                position INTEGER DEFAULT 0,
                changed_at TEXT NOT NULL,
                changed_by TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                track_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                comment TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id),
                FOREIGN KEY (track_id) REFERENCES tracks(id),
                UNIQUE(user_id, track_id)
            );

            CREATE TABLE IF NOT EXISTS internet_radio_stations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                stream_url TEXT NOT NULL,
                homepage_url TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS shares (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                description TEXT,
                expires_at TEXT,
                visit_count INTEGER DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );

            CREATE TABLE IF NOT EXISTS share_items (
                share_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                item_id TEXT NOT NULL,
                PRIMARY KEY (share_id, item_type, item_id),
                FOREIGN KEY (share_id) REFERENCES shares(id)
            );

            CREATE TABLE IF NOT EXISTS scan_status (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                scanning INTEGER NOT NULL DEFAULT 0,
                count INTEGER DEFAULT 0,
                folder_count INTEGER DEFAULT 0,
                last_scan TEXT,
                error TEXT,
                scan_type TEXT,
                elapsed_time INTEGER
            );

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_tracks_album ON tracks(album_id);
            CREATE INDEX IF NOT EXISTS idx_tracks_artist ON tracks(artist_id);
            CREATE INDEX IF NOT EXISTS idx_tracks_title ON tracks(title);
            CREATE INDEX IF NOT EXISTS idx_albums_artist ON albums(artist_id);
            CREATE INDEX IF NOT EXISTS idx_albums_name ON albums(name);
            CREATE INDEX IF NOT EXISTS idx_artists_name ON artists(name);
            CREATE INDEX IF NOT EXISTS idx_playlist_tracks ON playlist_tracks(playlist_id, position);

            -- Insert default scan status row
            INSERT OR IGNORE INTO scan_status (id, scanning, count, folder_count) VALUES (1, 0, 0, 0);
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get the VFS instance
    pub fn vfs(&self) -> &SharedVfs {
        &self.vfs
    }

    /// Get the database pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

#[async_trait]
impl Storage for DatabaseStorage {
    async fn initialize(&self) -> Result<()> {
        // Migrations already run in new()
        // Ensure default admin user exists
        let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if user_count.0 == 0 {
            let admin_user = User {
                id: Uuid::new_v4(),
                username: "admin".to_string(),
                password_hash: "admin".to_string(), // TODO: proper hashing
                email: Some("admin@reverie.local".to_string()),
                is_admin: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            self.save_user(&admin_user).await?;
        }

        // Insert default music folder if none exists
        let folder_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM music_folders")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if folder_count.0 == 0 {
            sqlx::query("INSERT INTO music_folders (name, path) VALUES (?, ?)")
                .bind("Music")
                .bind("/music")
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn close(&self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(true)
    }
}

#[async_trait]
impl TrackStorage for DatabaseStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, album_id, artist_id, duration, file_path, file_size,
                   bitrate, format, track_number, disc_number, year, genre, created_at, updated_at
            FROM tracks WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Track {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            title: r.get("title"),
            album_id: r
                .get::<Option<String>, _>("album_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            artist_id: r
                .get::<Option<String>, _>("artist_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            duration: r.get::<i64, _>("duration") as u32,
            file_path: r.get("file_path"),
            file_size: r.get::<i64, _>("file_size") as u64,
            bitrate: r.get::<i64, _>("bitrate") as u32,
            format: r.get("format"),
            track_number: r.get::<Option<i64>, _>("track_number").map(|n| n as u32),
            disc_number: r.get::<Option<i64>, _>("disc_number").map(|n| n as u32),
            year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
            genre: r.get("genre"),
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, album_id, artist_id, duration, file_path, file_size,
                   bitrate, format, track_number, disc_number, year, genre, created_at, updated_at
            FROM tracks ORDER BY title LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Track {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                title: r.get("title"),
                album_id: r
                    .get::<Option<String>, _>("album_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                duration: r.get::<i64, _>("duration") as u32,
                file_path: r.get("file_path"),
                file_size: r.get::<i64, _>("file_size") as u64,
                bitrate: r.get::<i64, _>("bitrate") as u32,
                format: r.get("format"),
                track_number: r.get::<Option<i64>, _>("track_number").map(|n| n as u32),
                disc_number: r.get::<Option<i64>, _>("disc_number").map(|n| n as u32),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tracks (id, title, album_id, artist_id, duration, file_path, file_size,
                               bitrate, format, track_number, disc_number, year, genre, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                album_id = excluded.album_id,
                artist_id = excluded.artist_id,
                duration = excluded.duration,
                file_path = excluded.file_path,
                file_size = excluded.file_size,
                bitrate = excluded.bitrate,
                format = excluded.format,
                track_number = excluded.track_number,
                disc_number = excluded.disc_number,
                year = excluded.year,
                genre = excluded.genre,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(track.id.to_string())
        .bind(&track.title)
        .bind(track.album_id.map(|id| id.to_string()))
        .bind(track.artist_id.map(|id| id.to_string()))
        .bind(track.duration as i64)
        .bind(&track.file_path)
        .bind(track.file_size as i64)
        .bind(track.bitrate as i64)
        .bind(&track.format)
        .bind(track.track_number.map(|n| n as i64))
        .bind(track.disc_number.map(|n| n as i64))
        .bind(track.year.map(|n| n as i64))
        .bind(&track.genre)
        .bind(track.created_at.to_rfc3339())
        .bind(track.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_track(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tracks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query(
            r#"
            SELECT id, title, album_id, artist_id, duration, file_path, file_size,
                   bitrate, format, track_number, disc_number, year, genre, created_at, updated_at
            FROM tracks WHERE title LIKE ? ORDER BY title LIMIT 100
            "#,
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Track {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                title: r.get("title"),
                album_id: r
                    .get::<Option<String>, _>("album_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                duration: r.get::<i64, _>("duration") as u32,
                file_path: r.get("file_path"),
                file_size: r.get::<i64, _>("file_size") as u64,
                bitrate: r.get::<i64, _>("bitrate") as u32,
                format: r.get("format"),
                track_number: r.get::<Option<i64>, _>("track_number").map(|n| n as u32),
                disc_number: r.get::<Option<i64>, _>("disc_number").map(|n| n as u32),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, album_id, artist_id, duration, file_path, file_size,
                   bitrate, format, track_number, disc_number, year, genre, created_at, updated_at
            FROM tracks WHERE album_id = ? ORDER BY disc_number, track_number
            "#,
        )
        .bind(album_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Track {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                title: r.get("title"),
                album_id: r
                    .get::<Option<String>, _>("album_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                duration: r.get::<i64, _>("duration") as u32,
                file_path: r.get("file_path"),
                file_size: r.get::<i64, _>("file_size") as u64,
                bitrate: r.get::<i64, _>("bitrate") as u32,
                format: r.get("format"),
                track_number: r.get::<Option<i64>, _>("track_number").map(|n| n as u32),
                disc_number: r.get::<Option<i64>, _>("disc_number").map(|n| n as u32),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, album_id, artist_id, duration, file_path, file_size,
                   bitrate, format, track_number, disc_number, year, genre, created_at, updated_at
            FROM tracks WHERE artist_id = ? ORDER BY title
            "#,
        )
        .bind(artist_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Track {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                title: r.get("title"),
                album_id: r
                    .get::<Option<String>, _>("album_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                duration: r.get::<i64, _>("duration") as u32,
                file_path: r.get("file_path"),
                file_size: r.get::<i64, _>("file_size") as u64,
                bitrate: r.get::<i64, _>("bitrate") as u32,
                format: r.get("format"),
                track_number: r.get::<Option<i64>, _>("track_number").map(|n| n as u32),
                disc_number: r.get::<Option<i64>, _>("disc_number").map(|n| n as u32),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }
}

#[async_trait]
impl AlbumStorage for DatabaseStorage {
    async fn get_album(&self, id: Uuid) -> Result<Option<Album>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, artist_id, year, genre, cover_art_path, created_at, updated_at
            FROM albums WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Album {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            name: r.get("name"),
            artist_id: r
                .get::<Option<String>, _>("artist_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
            genre: r.get("genre"),
            cover_art_path: r.get("cover_art_path"),
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn list_albums(&self, limit: usize, offset: usize) -> Result<Vec<Album>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, artist_id, year, genre, cover_art_path, created_at, updated_at
            FROM albums ORDER BY name LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Album {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                name: r.get("name"),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                cover_art_path: r.get("cover_art_path"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn save_album(&self, album: &Album) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO albums (id, name, artist_id, year, genre, cover_art_path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                artist_id = excluded.artist_id,
                year = excluded.year,
                genre = excluded.genre,
                cover_art_path = excluded.cover_art_path,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(album.id.to_string())
        .bind(&album.name)
        .bind(album.artist_id.map(|id| id.to_string()))
        .bind(album.year.map(|n| n as i64))
        .bind(&album.genre)
        .bind(&album.cover_art_path)
        .bind(album.created_at.to_rfc3339())
        .bind(album.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_album(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM albums WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_albums_by_artist(&self, artist_id: Uuid) -> Result<Vec<Album>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, artist_id, year, genre, cover_art_path, created_at, updated_at
            FROM albums WHERE artist_id = ? ORDER BY year DESC, name
            "#,
        )
        .bind(artist_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Album {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                name: r.get("name"),
                artist_id: r
                    .get::<Option<String>, _>("artist_id")
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                year: r.get::<Option<i64>, _>("year").map(|n| n as u32),
                genre: r.get("genre"),
                cover_art_path: r.get("cover_art_path"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }
}

#[async_trait]
impl ArtistStorage for DatabaseStorage {
    async fn get_artist(&self, id: Uuid) -> Result<Option<Artist>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, bio, created_at, updated_at
            FROM artists WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Artist {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            name: r.get("name"),
            bio: r.get("bio"),
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn list_artists(&self, limit: usize, offset: usize) -> Result<Vec<Artist>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, bio, created_at, updated_at
            FROM artists ORDER BY name LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Artist {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                name: r.get("name"),
                bio: r.get("bio"),
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn save_artist(&self, artist: &Artist) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO artists (id, name, bio, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                bio = excluded.bio,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(artist.id.to_string())
        .bind(&artist.name)
        .bind(&artist.bio)
        .bind(artist.created_at.to_rfc3339())
        .bind(artist.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_artist(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM artists WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl UserStorage for DatabaseStorage {
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, password_hash, email, is_admin, created_at, updated_at
            FROM users WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            email: r.get("email"),
            is_admin: r.get::<i64, _>("is_admin") != 0,
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, password_hash, email, is_admin, created_at, updated_at
            FROM users WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            username: r.get("username"),
            password_hash: r.get("password_hash"),
            email: r.get("email"),
            is_admin: r.get::<i64, _>("is_admin") != 0,
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn list_users(&self, limit: usize, offset: usize) -> Result<Vec<User>> {
        let rows = sqlx::query(
            r#"
            SELECT id, username, password_hash, email, is_admin, created_at, updated_at
            FROM users ORDER BY username LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| User {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                username: r.get("username"),
                password_hash: r.get("password_hash"),
                email: r.get("email"),
                is_admin: r.get::<i64, _>("is_admin") != 0,
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn save_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password_hash, email, is_admin, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                username = excluded.username,
                password_hash = excluded.password_hash,
                email = excluded.email,
                is_admin = excluded.is_admin,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(user.id.to_string())
        .bind(&user.username)
        .bind(&user.password_hash)
        .bind(&user.email)
        .bind(if user.is_admin { 1i64 } else { 0i64 })
        .bind(user.created_at.to_rfc3339())
        .bind(user.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl PlaylistStorage for DatabaseStorage {
    async fn get_playlist(&self, id: Uuid) -> Result<Option<Playlist>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, user_id, is_public, created_at, updated_at
            FROM playlists WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Playlist {
            id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
            name: r.get("name"),
            description: r.get("description"),
            user_id: Uuid::parse_str(r.get::<String, _>("user_id").as_str()).unwrap(),
            is_public: r.get::<i64, _>("is_public") != 0,
            created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        }))
    }

    async fn get_playlists_by_user(&self, user_id: Uuid) -> Result<Vec<Playlist>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, user_id, is_public, created_at, updated_at
            FROM playlists WHERE user_id = ? ORDER BY name
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Playlist {
                id: Uuid::parse_str(r.get::<String, _>("id").as_str()).unwrap(),
                name: r.get("name"),
                description: r.get("description"),
                user_id: Uuid::parse_str(r.get::<String, _>("user_id").as_str()).unwrap(),
                is_public: r.get::<i64, _>("is_public") != 0,
                created_at: DateTime::parse_from_rfc3339(r.get::<String, _>("created_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(r.get::<String, _>("updated_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }

    async fn save_playlist(&self, playlist: &Playlist) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO playlists (id, name, description, user_id, is_public, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                is_public = excluded.is_public,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(playlist.id.to_string())
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(playlist.user_id.to_string())
        .bind(if playlist.is_public { 1i64 } else { 0i64 })
        .bind(playlist.created_at.to_rfc3339())
        .bind(playlist.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<()> {
        // Delete playlist tracks first
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(playlist_id, track_id) DO UPDATE SET
                position = excluded.position
            "#,
        )
        .bind(playlist_track.playlist_id.to_string())
        .bind(playlist_track.track_id.to_string())
        .bind(playlist_track.position as i64)
        .bind(playlist_track.added_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
            .bind(playlist_id.to_string())
            .bind(track_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_playlist_tracks(&self, playlist_id: Uuid) -> Result<Vec<PlaylistTrack>> {
        let rows = sqlx::query(
            r#"
            SELECT playlist_id, track_id, position, added_at
            FROM playlist_tracks WHERE playlist_id = ? ORDER BY position
            "#,
        )
        .bind(playlist_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| PlaylistTrack {
                playlist_id: Uuid::parse_str(r.get::<String, _>("playlist_id").as_str()).unwrap(),
                track_id: Uuid::parse_str(r.get::<String, _>("track_id").as_str()).unwrap(),
                position: r.get::<i64, _>("position") as u32,
                added_at: DateTime::parse_from_rfc3339(r.get::<String, _>("added_at").as_str())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect())
    }
}

#[async_trait]
impl FileStorage for DatabaseStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let data = self.vfs.read(path).await?;
        Ok(data.to_vec())
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        self.vfs
            .write(path, bytes::Bytes::copy_from_slice(data))
            .await
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        self.vfs.exists(path).await
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        self.vfs.delete(path).await
    }

    async fn list_files(&self, path: &str) -> Result<Vec<String>> {
        let entries = self.vfs.list(path).await?;
        Ok(entries.into_iter().map(|e| e.path).collect())
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let meta = self.vfs.stat(path).await?;
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

// SubsonicStorage implementation will be added in a follow-up
// as it requires more complex queries and data transformation

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_storage_init() {
        let storage = DatabaseStorage::new(DatabaseConfig::memory())
            .await
            .unwrap();
        storage.initialize().await.unwrap();
        assert!(storage.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_track_crud() {
        let storage = DatabaseStorage::new(DatabaseConfig::memory())
            .await
            .unwrap();
        storage.initialize().await.unwrap();

        let track = Track {
            id: Uuid::new_v4(),
            title: "Test Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/test/path.mp3".to_string(),
            file_size: 5000000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Test".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Create
        storage.save_track(&track).await.unwrap();

        // Read
        let retrieved = storage.get_track(track.id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, track.title);

        // Delete
        storage.delete_track(track.id).await.unwrap();
        assert!(storage.get_track(track.id).await.unwrap().is_none());
    }
}
