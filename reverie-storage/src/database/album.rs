//! AlbumStorage 和 ArtistStorage 实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;
use crate::DatabaseStorage;
use reverie_core::{Album, Artist};

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
        .fetch_optional(self.pool())
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
        .fetch_all(self.pool())
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_album(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM albums WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool())
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
        .fetch_all(self.pool())
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
        .fetch_optional(self.pool())
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
        .fetch_all(self.pool())
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
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_artist(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM artists WHERE id = ?")
            .bind(id.to_string())
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
