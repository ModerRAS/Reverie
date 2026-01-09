//! TrackStorage 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::DatabaseStorage;
use reverie_core::Track;

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
        .fetch_optional(self.pool())
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
        .fetch_all(self.pool())
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_track(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM tracks WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool())
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
        .fetch_all(self.pool())
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
        .fetch_all(self.pool())
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
        .fetch_all(self.pool())
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
