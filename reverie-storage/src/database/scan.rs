//! 媒体库扫描功能实现
//!
//! 为 DatabaseStorage 提供媒体库扫描和数据持久化功能

use chrono::Utc;
use tracing::{error, info};

use crate::error::{Result, StorageError};
use crate::scanner::{MediaScanner, ScanResult};
use crate::DatabaseStorage;
use reverie_core::SubsonicScanStatus;

impl DatabaseStorage {
    /// 执行媒体库扫描
    ///
    /// 扫描指定路径下的所有音频文件，提取元数据并存储到数据库
    pub async fn perform_scan(&self, path: &str) -> Result<ScanResult> {
        let scanner = MediaScanner::new(self.vfs().clone());
        
        // 更新扫描状态为正在扫描
        self.set_scan_status(true, None).await?;

        // 执行扫描
        let result = scanner.scan(path).await;

        match &result {
            Ok(scan_result) => {
                // 将扫描结果保存到数据库
                self.save_scan_result(scan_result).await?;
                
                // 更新扫描状态
                let count = scan_result.tracks.len() as i64;
                self.set_scan_status(false, Some(count)).await?;
            }
            Err(e) => {
                // 记录错误
                self.set_scan_error(&e.to_string()).await?;
            }
        }

        result
    }

    /// 将扫描结果保存到数据库
    async fn save_scan_result(&self, result: &ScanResult) -> Result<()> {
        let now = Utc::now().to_rfc3339();

        // 保存艺术家
        for artist in result.artists.values() {
            sqlx::query(
                r#"INSERT OR REPLACE INTO artists (id, name, created_at, updated_at)
                   VALUES (?, ?, ?, ?)"#,
            )
            .bind(&artist.id)
            .bind(&artist.name)
            .bind(&now)
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        info!("Saved {} artists", result.artists.len());

        // 保存专辑
        for album in result.albums.values() {
            // 计算专辑总时长
            let duration: f32 = result
                .tracks
                .iter()
                .filter(|t| album.tracks.contains(&t.id))
                .map(|t| t.duration)
                .sum();

            sqlx::query(
                r#"INSERT OR REPLACE INTO albums 
                   (id, name, artist_id, year, genre, song_count, duration, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&album.id)
            .bind(&album.name)
            .bind(&album.artist_id)
            .bind(album.year)
            .bind(&album.genre)
            .bind(album.tracks.len() as i32)
            .bind(duration)
            .bind(&now)
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        info!("Saved {} albums", result.albums.len());

        // 保存曲目
        for track in &result.tracks {
            // 查找对应的专辑 ID
            let album_id = result
                .albums
                .values()
                .find(|a| a.tracks.contains(&track.id))
                .map(|a| a.id.clone());

            // 查找对应的艺术家 ID
            let artist_key = track
                .album_artist
                .as_ref()
                .or(track.artist.as_ref())
                .map(|s| s.to_lowercase());
            let artist_id = artist_key
                .and_then(|k| result.artists.get(&k))
                .map(|a| a.id.clone());

            sqlx::query(
                r#"INSERT OR REPLACE INTO tracks 
                   (id, title, album_id, artist_id, duration, file_path, file_size, 
                    bitrate, sample_rate, channels, format, track_number, disc_number, 
                    year, genre, created_at, updated_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&track.id)
            .bind(&track.title)
            .bind(&album_id)
            .bind(&artist_id)
            .bind(track.duration as i64)
            .bind(&track.file_path)
            .bind(track.file_size)
            .bind(track.bitrate)
            .bind(track.sample_rate)
            .bind(track.channels)
            .bind(&track.format)
            .bind(track.track_number)
            .bind(track.disc_number)
            .bind(track.year)
            .bind(&track.genre)
            .bind(&now)
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

            // 如果有封面图片，保存到专辑
            if let (Some(album_id), Some(cover_data)) = (&album_id, &track.cover_data) {
                // 生成封面路径
                let cover_path = format!(".covers/{}.jpg", album_id);
                
                // 通过 VFS 保存封面
                if let Err(e) = self
                    .vfs()
                    .write(&cover_path, bytes::Bytes::from(cover_data.clone()))
                    .await
                {
                    error!("Failed to save cover art for album {}: {}", album_id, e);
                } else {
                    // 更新专辑封面路径
                    let _ = sqlx::query("UPDATE albums SET cover_art_path = ? WHERE id = ?")
                        .bind(&cover_path)
                        .bind(album_id)
                        .execute(self.pool())
                        .await;
                }
            }
        }

        info!("Saved {} tracks", result.tracks.len());

        // 更新流派表
        self.update_genres().await?;

        Ok(())
    }

    /// 更新流派表
    async fn update_genres(&self) -> Result<()> {
        // 从曲目中提取所有不重复的流派
        sqlx::query(
            r#"INSERT OR IGNORE INTO genres (name)
               SELECT DISTINCT genre FROM tracks WHERE genre IS NOT NULL"#,
        )
        .execute(self.pool())
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 设置扫描状态
    async fn set_scan_status(&self, scanning: bool, count: Option<i64>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        if scanning {
            sqlx::query(
                "UPDATE scan_status SET scanning = 1, count = 0, error = NULL WHERE id = 1",
            )
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        } else {
            sqlx::query(
                "UPDATE scan_status SET scanning = 0, count = ?, last_scan = ?, error = NULL WHERE id = 1",
            )
            .bind(count.unwrap_or(0))
            .bind(&now)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// 设置扫描错误
    async fn set_scan_error(&self, error: &str) -> Result<()> {
        sqlx::query("UPDATE scan_status SET scanning = 0, error = ? WHERE id = 1")
            .bind(error)
            .execute(self.pool())
            .await
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取扫描状态 (异步版本，用于 API)
    pub async fn get_current_scan_status(&self) -> Result<SubsonicScanStatus> {
        use crate::SubsonicStorage;
        self.get_scan_status().await
    }
}
