//! UserStorage 和 PlaylistStorage 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::DatabaseStorage;
use reverie_core::{Playlist, PlaylistTrack, User};

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
        .fetch_optional(self.pool())
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
        .fetch_optional(self.pool())
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
        .fetch_all(self.pool())
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool())
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
        .fetch_optional(self.pool())
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
        .fetch_all(self.pool())
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
                user_id = excluded.user_id,
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn add_track_to_playlist(&self, playlist_track: &PlaylistTrack) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO playlist_tracks (playlist_id, track_id, position, added_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(playlist_track.playlist_id.to_string())
        .bind(playlist_track.track_id.to_string())
        .bind(playlist_track.position as i64)
        .bind(playlist_track.added_at.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn remove_track_from_playlist(&self, playlist_id: Uuid, track_id: Uuid) -> Result<()> {
        sqlx::query(
            "DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?",
        )
        .bind(playlist_id.to_string())
        .bind(track_id.to_string())
        .execute(self.pool())
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
        .fetch_all(self.pool())
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
