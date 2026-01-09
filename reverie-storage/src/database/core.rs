//! DatabaseStorage 核心实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::vfs::{create_vfs, SharedVfs, VfsConfig};
use reverie_core::{Album, Artist, Playlist, Track, User};

use super::config::DatabaseConfig;

/// 基于 SQLite 的存储实现
#[derive(Clone)]
pub struct DatabaseStorage {
    pool: Pool<Sqlite>,
    vfs: SharedVfs,
    #[allow(dead_code)]
    config: DatabaseConfig,
}

impl DatabaseStorage {
    /// 创建新的数据库存储
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

    /// 运行数据库迁移
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取 VFS 实例
    pub fn vfs(&self) -> &SharedVfs {
        &self.vfs
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}

#[async_trait]
impl Storage for DatabaseStorage {
    async fn initialize(&self) -> Result<()> {
        // 迁移已在 new() 中运行
        // 确保默认管理员用户存在
        let user_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(self.pool())
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
            .fetch_one(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        if folder_count.0 == 0 {
            sqlx::query("INSERT INTO music_folders (name, path) VALUES (?, ?)")
                .bind("Music")
                .bind("/music")
                .execute(self.pool())
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
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(true)
    }
}
