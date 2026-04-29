//! TrackStorage 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::DatabaseStorage;
use reverie_core::Track;

/// Helper struct for reading track rows from SQLite
#[derive(Debug, FromRow)]
struct TrackRow {
    id: String,
    title: String,
    album_id: Option<String>,
    artist_id: Option<String>,
    duration: i64,
    file_path: String,
    file_size: i64,
    bitrate: i64,
    format: String,
    track_number: Option<i64>,
    disc_number: Option<i64>,
    year: Option<i64>,
    genre: Option<String>,
    created_at: String,
    updated_at: String,
    // CUE columns
    source_file: Option<String>,
    byte_offset_start: Option<i64>,
    byte_offset_end: Option<i64>,
    cue_path: Option<String>,
    is_cue_virtual: Option<i64>,
}

/// Convert a TrackRow to a Track domain model
fn row_to_track(r: TrackRow) -> Track {
    Track {
        id: Uuid::parse_str(r.id.as_str()).unwrap(),
        title: r.title,
        album_id: r.album_id.and_then(|s| Uuid::parse_str(&s).ok()),
        artist_id: r.artist_id.and_then(|s| Uuid::parse_str(&s).ok()),
        duration: r.duration as u32,
        file_path: r.file_path,
        file_size: r.file_size as u64,
        bitrate: r.bitrate as u32,
        format: r.format,
        track_number: r.track_number.map(|n| n as u32),
        disc_number: r.disc_number.map(|n| n as u32),
        year: r.year.map(|n| n as u32),
        genre: r.genre,
        created_at: DateTime::parse_from_rfc3339(r.created_at.as_str())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(r.updated_at.as_str())
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        // CUE fields
        source_file: r.source_file,
        byte_offset_start: r.byte_offset_start.map(|n| n as u64),
        byte_offset_end: r.byte_offset_end.map(|n| n as u64),
        cue_path: r.cue_path,
        is_cue_virtual: r.is_cue_virtual.map(|n| n != 0),
    }
}

/// Common SELECT columns for track queries
const TRACK_SELECT_COLUMNS: &str = r#"
    id, title, album_id, artist_id, duration, file_path, file_size,
    bitrate, format, track_number, disc_number, year, genre, created_at, updated_at,
    source_file, byte_offset_start, byte_offset_end, cue_path, is_cue_virtual
"#;

#[async_trait]
impl TrackStorage for DatabaseStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        let row = sqlx::query_as::<_, TrackRow>(&format!(
            "SELECT {} FROM tracks WHERE id = ?",
            TRACK_SELECT_COLUMNS
        ))
        .bind(id.to_string())
        .fetch_optional(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(row.map(row_to_track))
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        let rows = sqlx::query_as::<_, TrackRow>(&format!(
            "SELECT {} FROM tracks ORDER BY title LIMIT ? OFFSET ?",
            TRACK_SELECT_COLUMNS
        ))
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(row_to_track).collect())
    }

    async fn save_track(&self, track: &Track) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tracks (id, title, album_id, artist_id, duration, file_path, file_size,
                               bitrate, format, track_number, disc_number, year, genre, created_at, updated_at,
                               source_file, byte_offset_start, byte_offset_end, cue_path, is_cue_virtual)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
                updated_at = excluded.updated_at,
                source_file = excluded.source_file,
                byte_offset_start = excluded.byte_offset_start,
                byte_offset_end = excluded.byte_offset_end,
                cue_path = excluded.cue_path,
                is_cue_virtual = excluded.is_cue_virtual
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
        .bind(&track.source_file)
        .bind(track.byte_offset_start.map(|n| n as i64))
        .bind(track.byte_offset_end.map(|n| n as i64))
        .bind(&track.cue_path)
        .bind(track.is_cue_virtual.map(|v| if v { 1i64 } else { 0i64 }))
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
        let rows = sqlx::query_as::<_, TrackRow>(&format!(
            "SELECT {} FROM tracks WHERE title LIKE ? ORDER BY title LIMIT 100",
            TRACK_SELECT_COLUMNS
        ))
        .bind(&pattern)
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(row_to_track).collect())
    }

    async fn get_tracks_by_album(&self, album_id: Uuid) -> Result<Vec<Track>> {
        let rows = sqlx::query_as::<_, TrackRow>(&format!(
            "SELECT {} FROM tracks WHERE album_id = ? ORDER BY disc_number, track_number",
            TRACK_SELECT_COLUMNS
        ))
        .bind(album_id.to_string())
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(row_to_track).collect())
    }

    async fn get_tracks_by_artist(&self, artist_id: Uuid) -> Result<Vec<Track>> {
        let rows = sqlx::query_as::<_, TrackRow>(&format!(
            "SELECT {} FROM tracks WHERE artist_id = ? ORDER BY title",
            TRACK_SELECT_COLUMNS
        ))
        .bind(artist_id.to_string())
        .fetch_all(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(row_to_track).collect())
    }
}

#[cfg(test)]
mod cue_columns_tests {
    use super::*;
    use crate::database::config::DatabaseConfig;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_cue_columns_save_and_retrieve() {
        let storage = DatabaseStorage::new(DatabaseConfig::memory())
            .await
            .unwrap();
        storage.initialize().await.unwrap();

        let track_id = Uuid::new_v4();
        let now = Utc::now();

        let track = Track {
            id: track_id,
            title: "Virtual Track 1".to_string(),
            album_id: None,
            artist_id: None,
            duration: 225,
            file_path: "/music/album.flac".to_string(),
            file_size: 45_000_000,
            bitrate: 1411,
            format: "flac".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2024),
            genre: Some("Classical".to_string()),
            created_at: now,
            updated_at: now,
            source_file: Some("/music/album.flac".to_string()),
            byte_offset_start: Some(0),
            byte_offset_end: Some(39_656_250),
            cue_path: Some("/music/album.cue".to_string()),
            is_cue_virtual: Some(true),
        };

        storage.save_track(&track).await.unwrap();

        let retrieved = storage.get_track(track_id).await.unwrap();
        assert!(retrieved.is_some());
        let t = retrieved.unwrap();
        assert_eq!(t.title, "Virtual Track 1");
        assert_eq!(t.source_file, Some("/music/album.flac".to_string()));
        assert_eq!(t.is_cue_virtual, Some(true));
    }

    #[tokio::test]
    async fn test_track_without_cue_fields() {
        let storage = DatabaseStorage::new(DatabaseConfig::memory())
            .await
            .unwrap();
        storage.initialize().await.unwrap();

        let track_id = Uuid::new_v4();
        let now = Utc::now();

        let track = Track {
            id: track_id,
            title: "Regular Track".to_string(),
            album_id: None,
            artist_id: None,
            duration: 180,
            file_path: "/music/song.mp3".to_string(),
            file_size: 5_000_000,
            bitrate: 320,
            format: "mp3".to_string(),
            track_number: Some(1),
            disc_number: Some(1),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            created_at: now,
            updated_at: now,
            source_file: None,
            byte_offset_start: None,
            byte_offset_end: None,
            cue_path: None,
            is_cue_virtual: None,
        };

        storage.save_track(&track).await.unwrap();

        let retrieved = storage.get_track(track_id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Regular Track");
        assert!(retrieved.source_file.is_none());
        assert!(retrieved.is_cue_virtual.is_none());
    }
}
