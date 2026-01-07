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
    SubsonicLyricLine, SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying,
    SubsonicOpenSubsonicExtension, SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs,
    SubsonicScanStatus, SubsonicSearchResult2, SubsonicSearchResult3, SubsonicShare,
    SubsonicStarred, SubsonicStructuredLyrics, SubsonicTopSongs, SubsonicUser, Track, User,
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

#[async_trait]
impl SubsonicStorage for DatabaseStorage {
    // === Browsing ===
    async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>> {
        let rows = sqlx::query("SELECT id, name FROM music_folders ORDER BY name")
            .fetch_all(&self.pool)
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
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut indexes: std::collections::HashMap<String, Vec<SubsonicArtist>> =
            std::collections::HashMap::new();

        for row in rows {
            let name: String = row.get("name");
            let first_char = name.chars().next().unwrap_or('#').to_uppercase().to_string();
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
        .fetch_all(&self.pool)
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
        // Try to find as album first
        if let Some(album) = SubsonicStorage::get_album(self, id).await? {
            let songs = self.get_songs_by_album_internal(id).await?;
            return Ok(Some(SubsonicDirectory::from_album(&album, songs)));
        }
        
        // Try as artist
        if let Some(artist) = SubsonicStorage::get_artist(self, id).await? {
            let albums = self.get_albums_by_artist_internal(id).await?;
            let children: Vec<MediaFile> = albums.into_iter().map(|a| MediaFile {
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
            }).collect();
            
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
        .fetch_optional(&self.pool)
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
        .fetch_optional(&self.pool)
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
        .fetch_optional(&self.pool)
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
        self.get_random_songs(Some(limit), None, None, None, None).await
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
        .fetch_all(&self.pool)
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

        let mut query = format!(
            r#"SELECT a.id, a.name, a.artist_id, a.year, a.genre, a.cover_art_path,
                      a.song_count, a.duration, a.play_count, a.starred_at, a.created_at,
                      ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE 1=1"#
        );

        if let Some(fy) = from_year {
            query.push_str(&format!(" AND a.year >= {}", fy));
        }
        if let Some(ty) = to_year {
            query.push_str(&format!(" AND a.year <= {}", ty));
        }
        if let Some(g) = genre {
            query.push_str(&format!(" AND a.genre = '{}'", g.replace('\'', "''")));
        }

        query.push_str(&format!(" ORDER BY {} LIMIT {} OFFSET {}", order_clause, limit, off));

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
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
        self.get_album_list(list_type, size, offset, from_year, to_year, genre, music_folder_id)
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
            .fetch_all(&self.pool)
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
        .fetch_all(&self.pool)
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
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let album_rows = sqlx::query(
            r#"SELECT a.*, ar.name as artist_name
               FROM albums a
               LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.starred_at IS NOT NULL ORDER BY a.starred_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let song_rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.starred_at IS NOT NULL ORDER BY t.starred_at DESC"#,
        )
        .fetch_all(&self.pool)
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
            songs: song_rows.iter().map(|r| self.row_to_media_file(r)).collect(),
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
        .fetch_all(&self.pool)
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
        .fetch_all(&self.pool)
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
        .fetch_all(&self.pool)
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
            .search2(query, artist_count, artist_offset, album_count, album_offset, song_count, song_offset)
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
                          (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as entry_count,
                          (SELECT SUM(t.duration) FROM playlist_tracks pt JOIN tracks t ON pt.track_id = t.id WHERE pt.playlist_id = p.id) as total_duration
                   FROM playlists p LEFT JOIN users u ON p.owner_id = u.id
                   WHERE u.username = ? OR p.public = 1 ORDER BY p.name"#,
            )
            .bind(user)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"SELECT p.*, u.username as owner_name,
                          (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as entry_count,
                          (SELECT SUM(t.duration) FROM playlist_tracks pt JOIN tracks t ON pt.track_id = t.id WHERE pt.playlist_id = p.id) as total_duration
                   FROM playlists p LEFT JOIN users u ON p.owner_id = u.id
                   ORDER BY p.name"#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicPlaylist {
                id: r.get("id"),
                name: r.get("name"),
                comment: r.get("comment"),
                owner: r.get::<Option<String>, _>("owner_name").unwrap_or_default(),
                public: r.get::<Option<i32>, _>("public").map(|v| v == 1).unwrap_or(false),
                song_count: r.get::<i32, _>("entry_count"),
                duration: r.get::<Option<f32>, _>("total_duration").unwrap_or(0.0) as i32,
                created: r
                    .get::<Option<String>, _>("created_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                changed: r
                    .get::<Option<String>, _>("updated_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                cover_art: r.get("cover_art_path"),
            })
            .collect())
    }

    async fn get_playlist(&self, id: &str) -> Result<Option<SubsonicPlaylistWithSongs>> {
        let row = sqlx::query(
            r#"SELECT p.*, u.username as owner_name,
                      (SELECT COUNT(*) FROM playlist_tracks WHERE playlist_id = p.id) as entry_count,
                      (SELECT SUM(t.duration) FROM playlist_tracks pt JOIN tracks t ON pt.track_id = t.id WHERE pt.playlist_id = p.id) as total_duration
               FROM playlists p LEFT JOIN users u ON p.owner_id = u.id WHERE p.id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
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
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Some(SubsonicPlaylistWithSongs {
            id: row.get("id"),
            name: row.get("name"),
            comment: row.get("comment"),
            owner: row.get::<Option<String>, _>("owner_name").unwrap_or_default(),
            public: row.get::<Option<i32>, _>("public").map(|v| v == 1).unwrap_or(false),
            song_count: row.get::<i32, _>("entry_count"),
            duration: row.get::<Option<f32>, _>("total_duration").unwrap_or(0.0) as i32,
            created: row
                .get::<Option<String>, _>("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
            changed: row
                .get::<Option<String>, _>("updated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(Utc::now),
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
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let playlist_name = name.unwrap_or("New Playlist");

        sqlx::query("INSERT INTO playlists (id, name, created_at, updated_at) VALUES (?, ?, ?, ?)")
            .bind(&id)
            .bind(playlist_name)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        for (pos, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)",
            )
            .bind(&id)
            .bind(*song_id)
            .bind(pos as i32)
            .execute(&self.pool)
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
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(c) = comment {
            sqlx::query("UPDATE playlists SET comment = ?, updated_at = ? WHERE id = ?")
                .bind(c)
                .bind(&now)
                .bind(playlist_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(p) = public {
            sqlx::query("UPDATE playlists SET public = ?, updated_at = ? WHERE id = ?")
                .bind(if p { 1 } else { 0 })
                .bind(&now)
                .bind(playlist_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        for idx in song_indexes_to_remove.iter().rev() {
            sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND position = ?")
                .bind(playlist_id)
                .bind(idx)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if !song_ids_to_add.is_empty() {
            let max_pos: i32 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(position), -1) FROM playlist_tracks WHERE playlist_id = ?",
            )
            .bind(playlist_id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or(-1);

            for (i, song_id) in song_ids_to_add.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO playlist_tracks (playlist_id, track_id, position) VALUES (?, ?, ?)",
                )
                .bind(playlist_id)
                .bind(*song_id)
                .bind(max_pos + 1 + i as i32)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn delete_playlist(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === Media Retrieval ===
    async fn get_stream_path(&self, id: &str) -> Result<Option<String>> {
        let path: Option<String> =
            sqlx::query_scalar("SELECT file_path FROM tracks WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(path)
    }

    async fn get_cover_art_path(&self, id: &str) -> Result<Option<String>> {
        // 尝试从 albums 表获取
        let album_art: Option<String> =
            sqlx::query_scalar("SELECT cover_art_path FROM albums WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if album_art.is_some() {
            return Ok(album_art);
        }

        // 尝试从 artists 表获取
        let artist_art: Option<String> =
            sqlx::query_scalar("SELECT image_url FROM artists WHERE id = ?")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if artist_art.is_some() {
            return Ok(artist_art);
        }

        // 尝试从 track 的 album 获取
        let track_album_art: Option<String> = sqlx::query_scalar(
            "SELECT a.cover_art_path FROM tracks t JOIN albums a ON t.album_id = a.id WHERE t.id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(track_album_art)
    }

    async fn get_lyrics(&self, artist: Option<&str>, title: Option<&str>) -> Result<Option<SubsonicLyrics>> {
        let row = sqlx::query(
            r#"SELECT t.lyrics, t.title, ar.name as artist_name
               FROM tracks t LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE (? IS NULL OR ar.name LIKE ?) AND (? IS NULL OR t.title LIKE ?)
               AND t.lyrics IS NOT NULL LIMIT 1"#,
        )
        .bind(artist)
        .bind(artist.map(|a| format!("%{}%", a)))
        .bind(title)
        .bind(title.map(|t| format!("%{}%", t)))
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(r) = row {
            Ok(Some(SubsonicLyrics {
                artist: r.get("artist_name"),
                title: r.get("title"),
                value: r.get::<Option<String>, _>("lyrics").unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_lyrics_by_song_id(&self, id: &str) -> Result<Vec<SubsonicStructuredLyrics>> {
        let row = sqlx::query(
            r#"SELECT t.lyrics, t.title, ar.name as artist_name
               FROM tracks t LEFT JOIN artists ar ON t.artist_id = ar.id WHERE t.id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let structured = if let Some(r) = row {
            if let Some(lyrics_text) = r.get::<Option<String>, _>("lyrics") {
                vec![SubsonicStructuredLyrics {
                    display_artist: r.get("artist_name"),
                    display_title: r.get("title"),
                    lang: "eng".to_string(),
                    synced: false,
                    offset: None,
                    lines: lyrics_text
                        .lines()
                        .map(|line| SubsonicLyricLine {
                            start: None,
                            value: line.to_string(),
                        })
                        .collect(),
                }]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        Ok(structured)
    }

    async fn get_avatar_path(&self, username: &str) -> Result<Option<String>> {
        let path: Option<String> =
            sqlx::query_scalar("SELECT avatar_path FROM users WHERE username = ?")
                .bind(username)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(path)
    }

    // === Annotation ===
    async fn star(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        for id in ids {
            sqlx::query("UPDATE tracks SET starred_at = ? WHERE id = ?")
                .bind(&now)
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        for id in album_ids {
            sqlx::query("UPDATE albums SET starred_at = ? WHERE id = ?")
                .bind(&now)
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        for id in artist_ids {
            sqlx::query("UPDATE artists SET starred_at = ? WHERE id = ?")
                .bind(&now)
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        Ok(())
    }

    async fn unstar(&self, ids: &[&str], album_ids: &[&str], artist_ids: &[&str]) -> Result<()> {
        for id in ids {
            sqlx::query("UPDATE tracks SET starred_at = NULL WHERE id = ?")
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        for id in album_ids {
            sqlx::query("UPDATE albums SET starred_at = NULL WHERE id = ?")
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        for id in artist_ids {
            sqlx::query("UPDATE artists SET starred_at = NULL WHERE id = ?")
                .bind(*id)
                .execute(&self.pool)
                .await
                .ok();
        }

        Ok(())
    }

    async fn set_rating(&self, id: &str, rating: i32) -> Result<()> {
        // 尝试更新 track
        let track_result = sqlx::query("UPDATE tracks SET rating = ? WHERE id = ?")
            .bind(rating)
            .bind(id)
            .execute(&self.pool)
            .await;

        if let Ok(r) = track_result {
            if r.rows_affected() > 0 {
                return Ok(());
            }
        }

        // 尝试更新 album
        let album_result = sqlx::query("UPDATE albums SET rating = ? WHERE id = ?")
            .bind(rating)
            .bind(id)
            .execute(&self.pool)
            .await;

        if let Ok(r) = album_result {
            if r.rows_affected() > 0 {
                return Ok(());
            }
        }

        Err(StorageError::NotFound(format!("Item {} not found", id)))
    }

    async fn scrobble(&self, id: &str, _time: Option<i64>, submission: bool) -> Result<()> {
        if submission {
            let now = Utc::now().to_rfc3339();

            sqlx::query("UPDATE tracks SET play_count = COALESCE(play_count, 0) + 1, last_played_at = ? WHERE id = ?")
                .bind(&now)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    // === Bookmarks ===
    async fn get_bookmarks(&self) -> Result<Vec<SubsonicBookmark>> {
        let rows = sqlx::query(
            r#"SELECT b.*, t.title, t.album_id, a.name as album_name, ar.name as artist_name,
                      u.username
               FROM bookmarks b
               JOIN tracks t ON b.track_id = t.id
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               LEFT JOIN users u ON b.user_id = u.id
               ORDER BY b.changed_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut bookmarks = Vec::new();
        for r in rows {
            let track_row = sqlx::query(
                r#"SELECT t.*, a.name as album_name, ar.name as artist_name
                   FROM tracks t
                   LEFT JOIN albums a ON t.album_id = a.id
                   LEFT JOIN artists ar ON t.artist_id = ar.id
                   WHERE t.id = ?"#,
            )
            .bind(r.get::<String, _>("track_id"))
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            if let Some(tr) = track_row {
                bookmarks.push(SubsonicBookmark {
                    position: r.get::<i64, _>("position"),
                    username: r.get::<Option<String>, _>("username").unwrap_or_default(),
                    comment: r.get("comment"),
                    created: r
                        .get::<Option<String>, _>("created_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    changed: r
                        .get::<Option<String>, _>("changed_at")
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                    entry: self.row_to_media_file(&tr),
                });
            }
        }

        Ok(bookmarks)
    }

    async fn create_bookmark(&self, id: &str, position: i64, comment: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"INSERT INTO bookmarks (track_id, position, comment, created_at, changed_at)
               VALUES (?, ?, ?, ?, ?)
               ON CONFLICT(track_id) DO UPDATE SET position = ?, comment = ?, changed_at = ?"#,
        )
        .bind(id)
        .bind(position)
        .bind(comment)
        .bind(&now)
        .bind(&now)
        .bind(position)
        .bind(comment)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_bookmark(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM bookmarks WHERE track_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_play_queue(&self) -> Result<Option<SubsonicPlayQueue>> {
        let row = sqlx::query(
            r#"SELECT pq.*, u.username
               FROM play_queues pq
               LEFT JOIN users u ON pq.user_id = u.id
               LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(r) = row {
            let entries = sqlx::query(
                r#"SELECT t.*, a.name as album_name, ar.name as artist_name
                   FROM play_queue_entries pqe
                   JOIN tracks t ON pqe.track_id = t.id
                   LEFT JOIN albums a ON t.album_id = a.id
                   LEFT JOIN artists ar ON t.artist_id = ar.id
                   WHERE pqe.queue_id = ?
                   ORDER BY pqe.position"#,
            )
            .bind(r.get::<String, _>("id"))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            // 获取当前索引对应的 track id
            let current_index = r.get::<Option<i32>, _>("current_index");
            let current_id = if let Some(idx) = current_index {
                entries.get(idx as usize).map(|e| e.get::<String, _>("id"))
            } else {
                None
            };

            Ok(Some(SubsonicPlayQueue {
                entries: entries.iter().map(|tr| self.row_to_media_file(tr)).collect(),
                current: current_id,
                position: r.get::<Option<i64>, _>("position").unwrap_or(0),
                username: r.get::<Option<String>, _>("username").unwrap_or_default(),
                changed: r
                    .get::<Option<String>, _>("changed_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                changed_by: r.get::<Option<String>, _>("changed_by").unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn save_play_queue(
        &self,
        song_ids: &[&str],
        current: Option<&str>,
        position: Option<i64>,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let queue_id = uuid::Uuid::new_v4().to_string();

        // 删除旧队列
        sqlx::query("DELETE FROM play_queue_entries")
            .execute(&self.pool)
            .await
            .ok();
        sqlx::query("DELETE FROM play_queues")
            .execute(&self.pool)
            .await
            .ok();

        let current_index = current.and_then(|c| song_ids.iter().position(|s| *s == c).map(|i| i as i32));

        sqlx::query(
            "INSERT INTO play_queues (id, current_index, position, changed_at) VALUES (?, ?, ?, ?)",
        )
        .bind(&queue_id)
        .bind(current_index)
        .bind(position)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        for (pos, song_id) in song_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO play_queue_entries (queue_id, track_id, position) VALUES (?, ?, ?)",
            )
            .bind(&queue_id)
            .bind(*song_id)
            .bind(pos as i32)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    // === Sharing ===
    async fn get_shares(&self) -> Result<Vec<SubsonicShare>> {
        let rows = sqlx::query(
            r#"SELECT s.*, u.username
               FROM shares s LEFT JOIN users u ON s.user_id = u.id ORDER BY s.created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let mut shares = Vec::new();
        for r in rows {
            let entries = sqlx::query(
                r#"SELECT t.*, a.name as album_name, ar.name as artist_name
                   FROM share_entries se
                   JOIN tracks t ON se.track_id = t.id
                   LEFT JOIN albums a ON t.album_id = a.id
                   LEFT JOIN artists ar ON t.artist_id = ar.id
                   WHERE se.share_id = ?"#,
            )
            .bind(r.get::<String, _>("id"))
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            shares.push(SubsonicShare {
                id: r.get("id"),
                url: r.get("url"),
                description: r.get("description"),
                username: r.get::<Option<String>, _>("username").unwrap_or_default(),
                created: r
                    .get::<Option<String>, _>("created_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now),
                expires: r
                    .get::<Option<String>, _>("expires_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                last_visited: r
                    .get::<Option<String>, _>("last_visited_at")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                visit_count: r.get::<Option<i32>, _>("visit_count").unwrap_or(0) as i64,
                entries: entries.iter().map(|tr| self.row_to_media_file(tr)).collect(),
            });
        }

        Ok(shares)
    }

    async fn create_share(
        &self,
        song_ids: &[&str],
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<SubsonicShare> {
        let now = Utc::now();
        let id = uuid::Uuid::new_v4().to_string();
        let url = format!("/share/{}", id);
        let expires_at = expires.map(|e| {
            DateTime::from_timestamp_millis(e)
                .unwrap_or(now)
                .to_rfc3339()
        });

        sqlx::query(
            "INSERT INTO shares (id, url, description, created_at, expires_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&url)
        .bind(description)
        .bind(now.to_rfc3339())
        .bind(&expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        for song_id in song_ids {
            sqlx::query("INSERT INTO share_entries (share_id, track_id) VALUES (?, ?)")
                .bind(&id)
                .bind(*song_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        let entries = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM share_entries se
               JOIN tracks t ON se.track_id = t.id
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE se.share_id = ?"#,
        )
        .bind(&id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(SubsonicShare {
            id,
            url,
            description: description.map(|s| s.to_string()),
            username: String::new(),
            created: now,
            expires: expires.and_then(DateTime::from_timestamp_millis),
            last_visited: None,
            visit_count: 0,
            entries: entries.iter().map(|tr| self.row_to_media_file(tr)).collect(),
        })
    }

    async fn update_share(
        &self,
        id: &str,
        description: Option<&str>,
        expires: Option<i64>,
    ) -> Result<()> {
        if let Some(desc) = description {
            sqlx::query("UPDATE shares SET description = ? WHERE id = ?")
                .bind(desc)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(exp) = expires {
            let expires_at = DateTime::from_timestamp_millis(exp).map(|d| d.to_rfc3339());
            sqlx::query("UPDATE shares SET expires_at = ? WHERE id = ?")
                .bind(expires_at)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    async fn delete_share(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM share_entries WHERE share_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        sqlx::query("DELETE FROM shares WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === Internet Radio ===
    async fn get_internet_radio_stations(&self) -> Result<Vec<SubsonicInternetRadioStation>> {
        let rows = sqlx::query("SELECT * FROM internet_radio_stations ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SubsonicInternetRadioStation {
                id: r.get("id"),
                name: r.get("name"),
                stream_url: r.get("stream_url"),
                homepage_url: r.get("homepage_url"),
            })
            .collect())
    }

    async fn create_internet_radio_station(
        &self,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO internet_radio_stations (id, name, stream_url, homepage_url) VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(stream_url)
        .bind(homepage_url)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_internet_radio_station(
        &self,
        id: &str,
        stream_url: &str,
        name: &str,
        homepage_url: Option<&str>,
    ) -> Result<()> {
        sqlx::query("UPDATE internet_radio_stations SET stream_url = ?, name = ?, homepage_url = ? WHERE id = ?")
            .bind(stream_url)
            .bind(name)
            .bind(homepage_url)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_internet_radio_station(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM internet_radio_stations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === User Management ===
    async fn get_user(&self, username: &str) -> Result<Option<SubsonicUser>> {
        let row = sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(SubsonicUser {
            username: row.get("username"),
            email: row.get("email"),
            scrobbling_enabled: row.get::<Option<i32>, _>("scrobbling_enabled").map(|v| v == 1).unwrap_or(true),
            max_bit_rate: row.get::<Option<i32>, _>("max_bit_rate"),
            admin_role: row.get::<Option<i32>, _>("admin_role").map(|v| v == 1).unwrap_or(false),
            settings_role: row.get::<Option<i32>, _>("settings_role").map(|v| v == 1).unwrap_or(false),
            download_role: row.get::<Option<i32>, _>("download_role").map(|v| v == 1).unwrap_or(true),
            upload_role: row.get::<Option<i32>, _>("upload_role").map(|v| v == 1).unwrap_or(false),
            playlist_role: row.get::<Option<i32>, _>("playlist_role").map(|v| v == 1).unwrap_or(true),
            cover_art_role: row.get::<Option<i32>, _>("cover_art_role").map(|v| v == 1).unwrap_or(true),
            comment_role: row.get::<Option<i32>, _>("comment_role").map(|v| v == 1).unwrap_or(false),
            podcast_role: row.get::<Option<i32>, _>("podcast_role").map(|v| v == 1).unwrap_or(false),
            stream_role: row.get::<Option<i32>, _>("stream_role").map(|v| v == 1).unwrap_or(true),
            jukebox_role: row.get::<Option<i32>, _>("jukebox_role").map(|v| v == 1).unwrap_or(false),
            share_role: row.get::<Option<i32>, _>("share_role").map(|v| v == 1).unwrap_or(false),
            video_conversion_role: row.get::<Option<i32>, _>("video_conversion_role").map(|v| v == 1).unwrap_or(false),
            avatar_last_changed: None,
            folders: vec![],
        }))
    }

    async fn get_users(&self) -> Result<Vec<SubsonicUser>> {
        let rows = sqlx::query("SELECT * FROM users ORDER BY username")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|row| SubsonicUser {
                username: row.get("username"),
                email: row.get("email"),
                scrobbling_enabled: row.get::<Option<i32>, _>("scrobbling_enabled").map(|v| v == 1).unwrap_or(true),
                max_bit_rate: row.get::<Option<i32>, _>("max_bit_rate"),
                admin_role: row.get::<Option<i32>, _>("admin_role").map(|v| v == 1).unwrap_or(false),
                settings_role: row.get::<Option<i32>, _>("settings_role").map(|v| v == 1).unwrap_or(false),
                download_role: row.get::<Option<i32>, _>("download_role").map(|v| v == 1).unwrap_or(true),
                upload_role: row.get::<Option<i32>, _>("upload_role").map(|v| v == 1).unwrap_or(false),
                playlist_role: row.get::<Option<i32>, _>("playlist_role").map(|v| v == 1).unwrap_or(true),
                cover_art_role: row.get::<Option<i32>, _>("cover_art_role").map(|v| v == 1).unwrap_or(true),
                comment_role: row.get::<Option<i32>, _>("comment_role").map(|v| v == 1).unwrap_or(false),
                podcast_role: row.get::<Option<i32>, _>("podcast_role").map(|v| v == 1).unwrap_or(false),
                stream_role: row.get::<Option<i32>, _>("stream_role").map(|v| v == 1).unwrap_or(true),
                jukebox_role: row.get::<Option<i32>, _>("jukebox_role").map(|v| v == 1).unwrap_or(false),
                share_role: row.get::<Option<i32>, _>("share_role").map(|v| v == 1).unwrap_or(false),
                video_conversion_role: row.get::<Option<i32>, _>("video_conversion_role").map(|v| v == 1).unwrap_or(false),
                avatar_last_changed: None,
                folders: vec![],
            })
            .collect())
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
        _music_folder_ids: &[i32],
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let password_hash = format!("{:x}", md5::compute(password)); // Simple hash, should use bcrypt in production

        sqlx::query(
            r#"INSERT INTO users (id, username, password_hash, email, admin_role, settings_role,
                stream_role, jukebox_role, download_role, upload_role, playlist_role, cover_art_role,
                comment_role, podcast_role, share_role, video_conversion_role, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(username)
        .bind(&password_hash)
        .bind(email)
        .bind(if admin_role { 1 } else { 0 })
        .bind(if settings_role { 1 } else { 0 })
        .bind(if stream_role { 1 } else { 0 })
        .bind(if jukebox_role { 1 } else { 0 })
        .bind(if download_role { 1 } else { 0 })
        .bind(if upload_role { 1 } else { 0 })
        .bind(if playlist_role { 1 } else { 0 })
        .bind(if cover_art_role { 1 } else { 0 })
        .bind(if comment_role { 1 } else { 0 })
        .bind(if podcast_role { 1 } else { 0 })
        .bind(if share_role { 1 } else { 0 })
        .bind(if video_conversion_role { 1 } else { 0 })
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

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
        _music_folder_ids: Option<&[i32]>,
        _max_bit_rate: Option<i32>,
    ) -> Result<()> {
        if let Some(pwd) = password {
            let password_hash = format!("{:x}", md5::compute(pwd));
            sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
                .bind(&password_hash)
                .bind(username)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        if let Some(e) = email {
            sqlx::query("UPDATE users SET email = ? WHERE username = ?")
                .bind(e)
                .bind(username)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        macro_rules! update_role {
            ($field:literal, $val:expr) => {
                if let Some(v) = $val {
                    sqlx::query(concat!("UPDATE users SET ", $field, " = ? WHERE username = ?"))
                        .bind(if v { 1 } else { 0 })
                        .bind(username)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
                }
            };
        }

        update_role!("admin_role", admin_role);
        update_role!("settings_role", settings_role);
        update_role!("stream_role", stream_role);
        update_role!("jukebox_role", jukebox_role);
        update_role!("download_role", download_role);
        update_role!("upload_role", upload_role);
        update_role!("playlist_role", playlist_role);
        update_role!("cover_art_role", cover_art_role);
        update_role!("comment_role", comment_role);
        update_role!("podcast_role", podcast_role);
        update_role!("share_role", share_role);
        update_role!("video_conversion_role", video_conversion_role);

        Ok(())
    }

    async fn delete_user(&self, username: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE username = ?")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn change_password(&self, username: &str, password: &str) -> Result<()> {
        let password_hash = format!("{:x}", md5::compute(password));

        sqlx::query("UPDATE users SET password_hash = ? WHERE username = ?")
            .bind(&password_hash)
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // === Scanning ===
    async fn get_scan_status(&self) -> Result<SubsonicScanStatus> {
        let row = sqlx::query(
            "SELECT * FROM scan_status ORDER BY started_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if let Some(r) = row {
            Ok(SubsonicScanStatus {
                scanning: r.get::<Option<i32>, _>("scanning").map(|v| v == 1).unwrap_or(false),
                count: r.get::<Option<i64>, _>("count").unwrap_or(0),
                folder_count: r.get::<Option<i64>, _>("folder_count").unwrap_or(0),
                last_scan: r
                    .get::<Option<String>, _>("last_scan")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|d| d.with_timezone(&Utc)),
                error: r.get("error"),
                scan_type: r.get("scan_type"),
                elapsed_time: r.get::<Option<i64>, _>("elapsed_time"),
            })
        } else {
            Ok(SubsonicScanStatus {
                scanning: false,
                count: 0,
                folder_count: 0,
                last_scan: None,
                error: None,
                scan_type: None,
                elapsed_time: None,
            })
        }
    }

    async fn start_scan(&self) -> Result<SubsonicScanStatus> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"INSERT INTO scan_status (id, scanning, started_at, count, folder_count)
               VALUES (?, 1, ?, 0, 0)
               ON CONFLICT(id) DO UPDATE SET scanning = 1, started_at = ?, count = 0"#,
        )
        .bind("current")
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // 实际扫描逻辑应该在后台进行
        // 这里只是标记扫描开始

        Ok(SubsonicScanStatus {
            scanning: true,
            count: 0,
            folder_count: 0,
            last_scan: None,
            error: None,
            scan_type: Some("fullScan".to_string()),
            elapsed_time: Some(0),
        })
    }

    // === OpenSubsonic Extensions ===
    async fn get_open_subsonic_extensions(&self) -> Result<Vec<SubsonicOpenSubsonicExtension>> {
        Ok(vec![
            SubsonicOpenSubsonicExtension {
                name: "songLyrics".to_string(),
                versions: vec![1],
            },
            SubsonicOpenSubsonicExtension {
                name: "transcodeOffset".to_string(),
                versions: vec![1],
            },
        ])
    }
}

// === Helper Methods ===
impl DatabaseStorage {
    fn row_to_media_file(&self, r: &sqlx::sqlite::SqliteRow) -> MediaFile {
        use sqlx::Row;

        MediaFile {
            id: r.get("id"),
            parent: r.get("album_id"),
            is_dir: false,
            title: r.get("title"),
            album: r.get("album_name"),
            artist: r.get("artist_name"),
            album_artist: r.get("artist_name"),
            track_number: r.get::<Option<i32>, _>("track_number"),
            year: r.get::<Option<i32>, _>("year"),
            genre: r.get("genre"),
            cover_art: r.get::<Option<String>, _>("album_id"),
            size: r.get::<Option<i64>, _>("file_size").unwrap_or(0),
            content_type: r.get::<Option<String>, _>("content_type").unwrap_or_default(),
            suffix: r.get::<Option<String>, _>("format").unwrap_or_default(),
            duration: r.get::<Option<f32>, _>("duration").unwrap_or(0.0),
            bit_rate: r.get::<Option<i32>, _>("bit_rate").unwrap_or(0),
            sample_rate: r.get::<Option<i32>, _>("sample_rate").unwrap_or(0),
            bit_depth: r.get::<Option<i32>, _>("bit_depth"),
            channels: r.get::<Option<i32>, _>("channels"),
            path: r.get::<Option<String>, _>("file_path").unwrap_or_default(),
            user_rating: r.get::<Option<i32>, _>("rating"),
            play_count: r.get::<Option<i64>, _>("play_count"),
            disc_number: r.get::<Option<i32>, _>("disc_number"),
            created: r
                .get::<Option<String>, _>("created_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            starred: r
                .get::<Option<String>, _>("starred_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.with_timezone(&Utc)),
            album_id: r.get("album_id"),
            artist_id: r.get("artist_id"),
            r#type: "music".to_string(),
            library_id: 1,
            missing: false,
        }
    }

    async fn get_songs_by_album_internal(&self, album_id: &str) -> Result<Vec<MediaFile>> {
        let rows = sqlx::query(
            r#"SELECT t.*, a.name as album_name, ar.name as artist_name
               FROM tracks t
               LEFT JOIN albums a ON t.album_id = a.id
               LEFT JOIN artists ar ON t.artist_id = ar.id
               WHERE t.album_id = ? ORDER BY t.disc_number, t.track_number"#,
        )
        .bind(album_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.iter().map(|r| self.row_to_media_file(r)).collect())
    }

    async fn get_albums_by_artist_internal(&self, artist_id: &str) -> Result<Vec<SubsonicAlbum>> {
        let rows = sqlx::query(
            r#"SELECT a.*, ar.name as artist_name
               FROM albums a LEFT JOIN artists ar ON a.artist_id = ar.id
               WHERE a.artist_id = ? ORDER BY a.year, a.name"#,
        )
        .bind(artist_id)
        .fetch_all(&self.pool)
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
                user_rating: r.get::<Option<i32>, _>("rating"),
            })
            .collect())
    }
}
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
