//! 媒体库扫描器实现
//!
//! 扫描音乐文件夹，提取元数据并存储到数据库

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::vfs::{SharedVfs, VfsEntry};
use super::metadata::{is_audio_file, AudioMetadata, get_extension};

/// 扫描进度状态
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    /// 是否正在扫描
    pub scanning: bool,
    /// 已扫描文件数
    pub count: i64,
    /// 已扫描文件夹数
    pub folder_count: i64,
    /// 上次扫描时间
    pub last_scan: Option<chrono::DateTime<Utc>>,
    /// 错误信息（如果有）
    pub error: Option<String>,
    /// 当前正在扫描的路径
    pub current_path: Option<String>,
}

/// 扫描到的音轨信息
#[derive(Debug, Clone)]
pub struct ScannedTrack {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration: f32,
    pub bitrate: i32,
    pub sample_rate: i32,
    pub channels: i32,
    pub file_path: String,
    pub file_size: i64,
    pub format: String,
    pub cover_data: Option<Vec<u8>>,
    pub cover_mime: Option<String>,
}

/// 扫描到的专辑信息
#[derive(Debug, Clone)]
pub struct ScannedAlbum {
    pub id: String,
    pub name: String,
    pub artist_id: Option<String>,
    pub artist_name: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub tracks: Vec<String>, // track ids
}

/// 扫描到的艺术家信息
#[derive(Debug, Clone)]
pub struct ScannedArtist {
    pub id: String,
    pub name: String,
}

/// 扫描结果
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub tracks: Vec<ScannedTrack>,
    pub albums: HashMap<String, ScannedAlbum>,
    pub artists: HashMap<String, ScannedArtist>,
}

/// 媒体库扫描器
pub struct MediaScanner {
    vfs: SharedVfs,
    scanning: Arc<AtomicBool>,
    count: Arc<AtomicI64>,
    folder_count: Arc<AtomicI64>,
    current_path: Arc<RwLock<Option<String>>>,
    last_error: Arc<RwLock<Option<String>>>,
}

impl MediaScanner {
    /// 创建新的扫描器
    pub fn new(vfs: SharedVfs) -> Self {
        Self {
            vfs,
            scanning: Arc::new(AtomicBool::new(false)),
            count: Arc::new(AtomicI64::new(0)),
            folder_count: Arc::new(AtomicI64::new(0)),
            current_path: Arc::new(RwLock::new(None)),
            last_error: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取当前扫描状态
    pub async fn get_progress(&self) -> ScanProgress {
        ScanProgress {
            scanning: self.scanning.load(Ordering::Relaxed),
            count: self.count.load(Ordering::Relaxed),
            folder_count: self.folder_count.load(Ordering::Relaxed),
            last_scan: None,
            error: self.last_error.read().await.clone(),
            current_path: self.current_path.read().await.clone(),
        }
    }

    /// 扫描指定路径
    pub async fn scan(&self, path: &str) -> Result<ScanResult> {
        // 检查是否已在扫描
        if self.scanning.swap(true, Ordering::SeqCst) {
            return Err(StorageError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Scan already in progress",
            )));
        }

        // 重置计数器
        self.count.store(0, Ordering::Relaxed);
        self.folder_count.store(0, Ordering::Relaxed);
        *self.last_error.write().await = None;

        info!("Starting media scan at path: {}", path);

        let result = self.scan_directory(path).await;

        // 扫描完成
        self.scanning.store(false, Ordering::Relaxed);
        *self.current_path.write().await = None;

        match &result {
            Ok(scan_result) => {
                info!(
                    "Scan completed: {} tracks, {} albums, {} artists",
                    scan_result.tracks.len(),
                    scan_result.albums.len(),
                    scan_result.artists.len()
                );
            }
            Err(e) => {
                error!("Scan failed: {}", e);
                *self.last_error.write().await = Some(e.to_string());
            }
        }

        result
    }

    /// 递归扫描目录
    async fn scan_directory(&self, path: &str) -> Result<ScanResult> {
        let mut result = ScanResult::default();

        // 获取目录列表
        let entries = match self.vfs.list_recursive(path).await {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to list directory {}: {}", path, e);
                return Ok(result);
            }
        };

        // 过滤出音频文件
        let audio_files: Vec<&VfsEntry> = entries
            .iter()
            .filter(|e| !e.metadata.is_dir && is_audio_file(&e.path))
            .collect();

        info!("Found {} audio files to scan", audio_files.len());

        for entry in audio_files {
            // 更新当前扫描路径
            *self.current_path.write().await = Some(entry.path.clone());

            match self.scan_file(&entry.path).await {
                Ok(track) => {
                    // 处理艺术家
                    let artist_id = if let Some(artist_name) = &track.album_artist {
                        let artist_key = artist_name.to_lowercase();
                        if !result.artists.contains_key(&artist_key) {
                            let artist = ScannedArtist {
                                id: Uuid::new_v4().to_string(),
                                name: artist_name.clone(),
                            };
                            result.artists.insert(artist_key.clone(), artist);
                        }
                        result.artists.get(&artist_key).map(|a| a.id.clone())
                    } else if let Some(artist_name) = &track.artist {
                        let artist_key = artist_name.to_lowercase();
                        if !result.artists.contains_key(&artist_key) {
                            let artist = ScannedArtist {
                                id: Uuid::new_v4().to_string(),
                                name: artist_name.clone(),
                            };
                            result.artists.insert(artist_key.clone(), artist);
                        }
                        result.artists.get(&artist_key).map(|a| a.id.clone())
                    } else {
                        None
                    };

                    // 处理专辑
                    if let Some(album_name) = &track.album {
                        let artist_name = track.album_artist.as_ref().or(track.artist.as_ref());
                        let album_key = format!(
                            "{}::{}",
                            artist_name.map(|s| s.to_lowercase()).unwrap_or_default(),
                            album_name.to_lowercase()
                        );

                        let album = result.albums.entry(album_key.clone()).or_insert_with(|| {
                            ScannedAlbum {
                                id: Uuid::new_v4().to_string(),
                                name: album_name.clone(),
                                artist_id: artist_id.clone(),
                                artist_name: artist_name.cloned(),
                                year: track.year,
                                genre: track.genre.clone(),
                                tracks: Vec::new(),
                            }
                        });

                        album.tracks.push(track.id.clone());

                        // 更新年份（如果缺失）
                        if album.year.is_none() && track.year.is_some() {
                            album.year = track.year;
                        }
                    }

                    result.tracks.push(track);
                    self.count.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    debug!("Failed to scan file {}: {}", entry.path, e);
                }
            }
        }

        // 统计文件夹数
        let folder_count = entries.iter().filter(|e| e.metadata.is_dir).count() as i64;
        self.folder_count.store(folder_count, Ordering::Relaxed);

        Ok(result)
    }

    /// 扫描单个文件
    async fn scan_file(&self, path: &str) -> Result<ScannedTrack> {
        // 读取文件元数据
        let file_meta = self.vfs.stat(path).await?;
        
        // 读取文件内容
        let file_data = self.vfs.read(path).await?;
        
        // 获取文件扩展名
        let extension = get_extension(path).unwrap_or("mp3");
        
        // 提取音频元数据
        let metadata = AudioMetadata::from_bytes(&file_data, Some(extension))?;

        // 生成 track ID
        let track_id = Uuid::new_v4().to_string();

        // 使用文件名作为默认标题
        let default_title = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Ok(ScannedTrack {
            id: track_id,
            title: metadata.title.unwrap_or(default_title),
            artist: metadata.artist,
            album: metadata.album,
            album_artist: metadata.album_artist,
            year: metadata.year,
            genre: metadata.genre,
            track_number: metadata.track_number,
            disc_number: metadata.disc_number,
            duration: metadata.duration,
            bitrate: metadata.bitrate,
            sample_rate: metadata.sample_rate,
            channels: metadata.channels,
            file_path: path.to_string(),
            file_size: file_meta.size as i64,
            format: extension.to_string(),
            cover_data: metadata.cover_data,
            cover_mime: metadata.cover_mime,
        })
    }

    /// 检查是否正在扫描
    pub fn is_scanning(&self) -> bool {
        self.scanning.load(Ordering::Relaxed)
    }

    /// 停止扫描
    pub fn stop(&self) {
        self.scanning.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_default() {
        let result = ScanResult::default();
        assert!(result.tracks.is_empty());
        assert!(result.albums.is_empty());
        assert!(result.artists.is_empty());
    }
}
