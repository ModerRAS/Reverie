//! 媒体库扫描器实现
//!
//! 扫描音乐文件夹，提取元数据并存储到数据库

use std::collections::HashMap;
use std::io::{BufReader, Cursor};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::vfs::{SharedVfs, VfsEntry};
use super::cue::{CueSheet, CueTrack, stable_uuid};
#[cfg(feature = "flac")]
use super::cue::extract_embedded_cue_from_flac;
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
    /// CUE: path to the .cue file (None for non-CUE tracks)
    pub cue_path: Option<String>,
    /// CUE: parent audio file for virtual tracks
    pub source_file: Option<String>,
    /// CUE: byte offset within source file (lossless only)
    pub byte_offset_start: Option<u64>,
    /// CUE: byte offset within source file (lossless only)
    pub byte_offset_end: Option<u64>,
    /// CUE: true if this track was created from CUE data
    pub is_cue_virtual: bool,
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
                Ok(tracks) => {
                    for track in tracks {
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

    /// 扫描单个文件，返回该文件的所有音轨（含 CUE 虚拟音轨）
    async fn scan_file(&self, path: &str) -> Result<Vec<ScannedTrack>> {
        // 读取文件元数据
        let file_meta = self.vfs.stat(path).await?;

        // 读取文件内容
        let file_data = self.vfs.read(path).await?;

        // 获取文件扩展名
        let extension = get_extension(path).unwrap_or("mp3");

        // 提取音频元数据
        let metadata = AudioMetadata::from_bytes(&file_data, Some(extension))?;

        // 使用文件名作为默认标题
        let default_title = std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // 构建 whole-disc 音轨（track_index 0 = whole disc）
        let whole_disc = ScannedTrack {
            id: stable_uuid(path, 0),
            title: metadata.title.clone().unwrap_or_else(|| default_title.clone()),
            artist: metadata.artist.clone(),
            album: metadata.album.clone(),
            album_artist: metadata.album_artist.clone(),
            year: metadata.year,
            genre: metadata.genre.clone(),
            track_number: metadata.track_number,
            disc_number: metadata.disc_number,
            duration: metadata.duration,
            bitrate: metadata.bitrate,
            sample_rate: metadata.sample_rate,
            channels: metadata.channels,
            file_path: path.to_string(),
            file_size: file_meta.size as i64,
            format: extension.to_string(),
            cover_data: metadata.cover_data.clone(),
            cover_mime: metadata.cover_mime.clone(),
            cue_path: None,
            source_file: Some(path.to_string()),
            byte_offset_start: None,
            byte_offset_end: None,
            is_cue_virtual: false,
        };

        let mut tracks = vec![whole_disc];

        // CUE 检测
        if let Some((cue_sheet, cue_path_str)) = self.detect_cue(path, &file_data, &metadata).await? {
            // Determine if format is lossless for byte offset calculation
            let is_lossless = matches!(extension, "flac" | "wav" | "ape" | "wv");
            let file_duration = metadata.duration as f64;

            for (i, cue_track) in cue_sheet.tracks.iter().enumerate() {
                let start_secs = cue_track.index_start;

                // Duration: next track start - this track start, or file end for last
                let end_secs = if i + 1 < cue_sheet.tracks.len() {
                    cue_sheet.tracks[i + 1].index_start
                } else {
                    file_duration
                };
                let duration = ((end_secs - start_secs).max(1.0)) as f32;

                // Byte offsets (lossless only)
                let (byte_start, byte_end) = if is_lossless {
                    let off_start = compute_byte_offset(start_secs, &metadata, file_meta.size);
                    let off_end = compute_byte_offset(end_secs, &metadata, file_meta.size);
                    (Some(off_start), Some(off_end))
                } else {
                    (None, None)
                };

                let track_num: Option<i32> = cue_track.track_number.parse().ok();

                // Virtual track: track_index = i + 1 (0 is reserved for whole-disc)
                let virtual_track = ScannedTrack {
                    id: stable_uuid(path, i as u32 + 1),
                    title: cue_track
                        .title
                        .clone()
                        .unwrap_or_else(|| format!("Track {}", cue_track.track_number)),
                    artist: cue_track
                        .performer
                        .clone()
                        .or_else(|| cue_sheet.album_performer.clone())
                        .or_else(|| metadata.artist.clone()),
                    album: cue_sheet
                        .album_title
                        .clone()
                        .or_else(|| metadata.album.clone()),
                    album_artist: cue_sheet
                        .album_performer
                        .clone()
                        .or_else(|| metadata.album_artist.clone()),
                    year: metadata.year,
                    genre: metadata.genre.clone(),
                    track_number: track_num,
                    disc_number: metadata.disc_number.or(Some(1)),
                    duration,
                    bitrate: metadata.bitrate,
                    sample_rate: metadata.sample_rate,
                    channels: metadata.channels,
                    file_path: path.to_string(),
                    file_size: file_meta.size as i64,
                    format: extension.to_string(),
                    cover_data: None, // virtual tracks share parent cover
                    cover_mime: None,
                    cue_path: Some(cue_path_str.clone()),
                    source_file: Some(path.to_string()),
                    byte_offset_start: byte_start,
                    byte_offset_end: byte_end,
                    is_cue_virtual: true,
                };
                tracks.push(virtual_track);
            }
        }

        Ok(tracks)
    }

    /// Detect CUE sheet for an audio file.
    ///
    /// Priority: external .cue sidecar > embedded FLAC CUESHEET.
    /// Returns `(CueSheet, cue_path_string)` if found.
    async fn detect_cue(
        &self,
        path: &str,
        file_data: &[u8],
        metadata: &AudioMetadata,
    ) -> Result<Option<(CueSheet, String)>> {
        let extension = get_extension(path).unwrap_or("");

        // 1. Check for external .cue sidecar via VFS
        let cue_vfs_path = std::path::Path::new(path).with_extension("cue");
        if let Some(cue_path_str) = cue_vfs_path.to_str() {
            if self.vfs.exists(cue_path_str).await.unwrap_or(false) {
                let cue_bytes = self.vfs.read(cue_path_str).await?;
                match parse_cue_from_bytes(&cue_bytes) {
                    Ok(cue_sheet) => return Ok(Some((cue_sheet, cue_path_str.to_string()))),
                    Err(e) => {
                        warn!("Failed to parse CUE file {}, falling back to whole-disc only: {}", cue_path_str, e);
                        return Ok(None);
                    }
                }
            }
        }

        // 2. Check for embedded FLAC CUESHEET (feature-gated)
        #[cfg(feature = "flac")]
        if extension == "flac" {
            let _ = metadata; // metadata not needed for FLAC extraction path
            let tmp_dir = std::env::temp_dir();
            let tmp_name = format!("reverie_flac_{}.tmp", Uuid::new_v4());
            let tmp_path = tmp_dir.join(&tmp_name);
            if let Err(e) = std::fs::write(&tmp_path, file_data) {
                debug!("Failed to write FLAC tempfile for CUESHEET extraction: {}", e);
                return Ok(None);
            }
            let result = extract_embedded_cue_from_flac(&tmp_path);
            let _ = std::fs::remove_file(&tmp_path);
            match result {
                Ok(Some(cue_sheet)) => {
                    return Ok(Some((cue_sheet, path.to_string())));
                }
                Ok(None) => {}
                Err(e) => {
                    debug!("Failed to extract embedded FLAC CUESHEET: {}", e);
                }
            }
        }

        let _ = (path, file_data, metadata);
        Ok(None)
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

/// Parse a CUE sheet from in-memory bytes (VFS-compatible).
///
/// Strips UTF-8 BOM, filters blank lines, then delegates to rcue strict parser.
fn parse_cue_from_bytes(data: &[u8]) -> Result<CueSheet> {
    let raw = std::str::from_utf8(data).map_err(|e| {
        StorageError::Unavailable(format!("CUE file is not valid UTF-8: {}", e))
    })?;

    // Strip UTF-8 BOM
    let content = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);

    // Filter blank/whitespace-only lines
    let filtered: String = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<&str>>()
        .join("\n");

    let cursor = Cursor::new(filtered.as_bytes());
    let mut buf_reader = BufReader::new(cursor);

    let cue = rcue::parser::parse(&mut buf_reader, true).map_err(|e| {
        StorageError::Unavailable(format!("Failed to parse CUE file: {}", e))
    })?;

    let album_title = cue.title;
    let album_performer = cue.performer;
    let file_path = cue
        .files
        .first()
        .map(|f| f.file.clone())
        .unwrap_or_default();

    let mut tracks = Vec::new();
    for file in &cue.files {
        let file_name = file.file.clone();
        for track in &file.tracks {
            let mut index_start = 0.0f64;
            let mut index_pregap: Option<f64> = None;
            for (index_num, duration) in &track.indices {
                let seconds = duration.as_secs_f64();
                match index_num.as_str() {
                    "00" => index_pregap = Some(seconds),
                    "01" => index_start = seconds,
                    _ => {}
                }
            }
            tracks.push(CueTrack {
                track_number: track.no.clone(),
                title: track.title.clone(),
                performer: track.performer.clone(),
                index_start,
                index_pregap,
                file_name: file_name.clone(),
            });
        }
    }

    Ok(CueSheet {
        album_title,
        album_performer,
        tracks,
        file_path,
    })
}

/// Compute byte offset within an audio file for a given timestamp.
///
/// For lossless formats (FLAC/WAV/APE/WV), the byte position is approximated
/// using the bitrate. Falls back to file_size/duration estimate if bitrate is 0.
fn compute_byte_offset(index_start_seconds: f64, metadata: &AudioMetadata, file_size: u64) -> u64 {
    let bytes_per_second = if metadata.bitrate > 0 {
        metadata.bitrate as f64 * 1000.0 / 8.0
    } else if metadata.duration > 0.0 {
        file_size as f64 / metadata.duration as f64
    } else {
        0.0
    };
    (index_start_seconds * bytes_per_second) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vfs::{create_vfs, VfsConfig};

    /// Helper to create a VFS rooted at a temp directory, write files into it,
    /// and return (temp_dir, vfs). The temp dir is automatically cleaned up.
    async fn setup_temp_vfs(files: &[(&str, &[u8])]) -> (tempfile::TempDir, SharedVfs) {
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, data) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::write(&path, data).expect("write test file");
        }
        let root = dir.path().to_string_lossy().to_string();
        let vfs = create_vfs(VfsConfig::local(root)).expect("create VFS");
        (dir, vfs)
    }

    /// Generate a minimal valid WAV file via hound.
    fn make_test_wav(
        sample_rate: u32,
        channels: u16,
        duration_secs: f32,
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(std::io::Cursor::new(&mut buf), spec)
            .expect("WAV writer");
        let num_samples = (sample_rate as f32 * duration_secs) as u32;
        for i in 0..num_samples {
            let sample = ((i as f64 * 440.0 * 2.0 * std::f64::consts::PI / sample_rate as f64).sin()
                * 16000.0) as i16;
            for _ in 0..channels {
                writer.write_sample(sample).expect("write sample");
            }
        }
        writer.finalize().expect("finalize WAV");
        buf
    }

    #[test]
    fn test_scan_result_default() {
        let result = ScanResult::default();
        assert!(result.tracks.is_empty());
        assert!(result.albums.is_empty());
        assert!(result.artists.is_empty());
    }

    #[tokio::test]
    async fn test_scan_cue_sidecar_creates_virtual_tracks() {
        // 10-second WAV at 44100 Hz, stereo, 1411 kbps
        let wav_data = make_test_wav(44100, 2, 10.0);

        // CUE with 2 tracks: Track 1 at 0:00, Track 2 at 0:05
        let cue_content = r#"PERFORMER "Test Artist"
TITLE "Test Album"
FILE "sample.wav" WAVE
  TRACK 01 AUDIO
    TITLE "First Track"
    PERFORMER "Test Artist"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Second Track"
    PERFORMER "Test Artist"
    INDEX 01 00:05:00
"#;

        let (_dir, vfs) = setup_temp_vfs(&[
            ("sample.wav", &wav_data),
            ("sample.cue", cue_content.as_bytes()),
        ]).await;

        let scanner = MediaScanner::new(vfs);
        let result = scanner.scan("").await.expect("scan");

        // Expected: 1 whole-disc + 2 virtual = 3 tracks
        assert_eq!(result.tracks.len(), 3, "should have 3 tracks (1 whole + 2 virtual)");

        // Whole-disc track
        let whole = result.tracks.iter().find(|t| !t.is_cue_virtual).expect("whole-disc track");
        assert!(!whole.is_cue_virtual);
        assert!(whole.cue_path.is_none());
        assert_eq!(whole.source_file.as_deref(), Some("sample.wav"));
        assert!(whole.byte_offset_start.is_none());
        assert!(whole.byte_offset_end.is_none());
        assert!(whole.duration > 0.0);

        // Virtual tracks
        let virt: Vec<&ScannedTrack> = result.tracks.iter().filter(|t| t.is_cue_virtual).collect();
        assert_eq!(virt.len(), 2);

        assert_eq!(virt[0].title, "First Track");
        assert_eq!(virt[0].track_number, Some(1));
        assert_eq!(virt[0].cue_path.as_deref(), Some("sample.cue"));
        assert_eq!(virt[0].source_file.as_deref(), Some("sample.wav"));
        assert!(virt[0].is_cue_virtual);
        // Duration ≈ 5s for first track (00:05:00 - 00:00:00)
        assert!((virt[0].duration - 5.0).abs() < 0.5);

        assert_eq!(virt[1].title, "Second Track");
        assert_eq!(virt[1].track_number, Some(2));
        // Duration ≈ 5s (10s file - 5s start) for last track
        assert!((virt[1].duration - 5.0).abs() < 0.5);
    }

    #[tokio::test]
    async fn test_scan_no_cue_unchanged() {
        let wav_data = make_test_wav(44100, 2, 3.0);
        let (_dir, vfs) = setup_temp_vfs(&[
            ("sample.wav", &wav_data),
        ]).await;

        let scanner = MediaScanner::new(vfs);
        let result = scanner.scan("").await.expect("scan");

        // Should only have 1 whole-disc track
        assert_eq!(result.tracks.len(), 1);
        let track = &result.tracks[0];
        assert!(!track.is_cue_virtual);
        assert!(track.cue_path.is_none());
        assert_eq!(track.source_file.as_deref(), Some("sample.wav"));
        assert!(track.byte_offset_start.is_none());
        assert!(track.byte_offset_end.is_none());
    }

    #[tokio::test]
    async fn test_virtual_track_byte_offsets() {
        // WAV: 44100 Hz, stereo, 16-bit → 1411 kbps, 176400 bytes/sec
        let wav_data = make_test_wav(44100, 2, 10.0);

        let cue_content = r#"PERFORMER "Test"
TITLE "Offset Test"
FILE "sample.wav" WAVE
  TRACK 01 AUDIO
    TITLE "T1"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "T2"
    INDEX 01 00:05:00
  TRACK 03 AUDIO
    TITLE "T3"
    INDEX 01 00:08:00
"#;

        let (_dir, vfs) = setup_temp_vfs(&[
            ("sample.wav", &wav_data),
            ("sample.cue", cue_content.as_bytes()),
        ]).await;

        let scanner = MediaScanner::new(vfs);
        let result = scanner.scan("").await.expect("scan");

        let virt: Vec<&ScannedTrack> = result.tracks.iter().filter(|t| t.is_cue_virtual).collect();
        assert_eq!(virt.len(), 3);

        // WAV byte rate: 44100 * 2 * 2 = 176400 bytes/sec (or 1411*125 = 176375)
        // Track 1: 0s → 5s → offset_start=0, offset_end ≈ 5*176375
        let t1 = virt[0];
        assert_eq!(t1.byte_offset_start, Some(0));
        let expected_end_1 = (5.0 * 44100.0 * 2.0 * 2.0) as u64; // 882000
        assert!(
            (t1.byte_offset_end.unwrap() as i64 - expected_end_1 as i64).abs() < 5000,
            "T1 byte_offset_end was {}, expected ~{}",
            t1.byte_offset_end.unwrap(),
            expected_end_1
        );

        // Track 2: 5s → 8s → offset_start ≈ 5*176375
        let t2 = virt[1];
        let expected_start_2 = (5.0 * 44100.0 * 2.0 * 2.0) as u64;
        assert!(
            (t2.byte_offset_start.unwrap() as i64 - expected_start_2 as i64).abs() < 5000
        );

        // Track 3: last → duration = 10 - 8 = 2s
        let t3 = virt[2];
        assert!((t3.duration - 2.0).abs() < 0.5);
    }

    #[cfg(feature = "flac")]
    #[tokio::test]
    async fn test_scan_flac_with_embedded_cuesheet() {
        use crate::scanner::cue::tests::make_test_flac;

        let sample_rate: u32 = 48000;

        // Build CUESHEET with 2 tracks
        let flac_cue = flac::metadata::CueSheet {
            media_catalog_number: String::new(),
            lead_in: 0,
            is_cd: false,
            tracks: vec![
                flac::metadata::CueSheetTrack {
                    offset: 0,
                    number: 1,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
                flac::metadata::CueSheetTrack {
                    offset: sample_rate as u64, // 1 second in
                    number: 2,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
            ],
        };

        let flac_data = make_test_flac(sample_rate, Some(&flac_cue));
        let (_dir, vfs) = setup_temp_vfs(&[
            ("sample.flac", &flac_data),
        ]).await;

        let scanner = MediaScanner::new(vfs);
        let result = scanner.scan("").await.expect("scan");

        // Expected: 1 whole-disc + 2 virtual = 3 tracks
        assert_eq!(result.tracks.len(), 3, "should have 3 tracks (1 whole + 2 virtual)");

        // Whole-disc track
        let whole = result.tracks.iter().find(|t| !t.is_cue_virtual).expect("whole-disc");
        assert!(!whole.is_cue_virtual);

        // Virtual tracks
        let virt: Vec<&ScannedTrack> = result.tracks.iter().filter(|t| t.is_cue_virtual).collect();
        assert_eq!(virt.len(), 2);

        assert_eq!(virt[0].track_number, Some(1));
        assert!(virt[0].is_cue_virtual);
        assert_eq!(virt[0].source_file.as_deref(), Some("sample.flac"));

        assert_eq!(virt[1].track_number, Some(2));
    }

    #[cfg(feature = "flac")]
    #[tokio::test]
    async fn test_cue_priority_over_embedded() {
        use crate::scanner::cue::tests::make_test_flac;

        let sample_rate: u32 = 48000;

        // FLAC with 2 embedded tracks (Track 1=0s, Track 2=1s)
        let flac_cue = flac::metadata::CueSheet {
            media_catalog_number: String::new(),
            lead_in: 0,
            is_cd: false,
            tracks: vec![
                flac::metadata::CueSheetTrack {
                    offset: 0,
                    number: 1,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
                flac::metadata::CueSheetTrack {
                    offset: sample_rate as u64,
                    number: 2,
                    isrc: String::new(),
                    is_audio: true,
                    is_pre_emphasis: false,
                    indices: vec![flac::metadata::CueSheetTrackIndex {
                        offset: 0,
                        number: 1,
                    }],
                },
            ],
        };
        let flac_data = make_test_flac(sample_rate, Some(&flac_cue));

        // External .cue with 3 tracks (should win over embedded)
        let cue_content = r#"PERFORMER "External Artist"
TITLE "External Album"
FILE "sample.flac" WAVE
  TRACK 01 AUDIO
    TITLE "Overridden T1"
    INDEX 01 00:00:00
  TRACK 02 AUDIO
    TITLE "Overridden T2"
    INDEX 01 00:01:00
  TRACK 03 AUDIO
    TITLE "Overridden T3"
    INDEX 01 00:02:00
"#;

        let (_dir, vfs) = setup_temp_vfs(&[
            ("sample.flac", &flac_data),
            ("sample.cue", cue_content.as_bytes()),
        ]).await;

        let scanner = MediaScanner::new(vfs);
        let result = scanner.scan("").await.expect("scan");

        // External CUE wins: 1 whole + 3 virtual = 4 tracks
        assert_eq!(result.tracks.len(), 4, "should have 4 tracks (1 whole + 3 virtual from external CUE)");

        let virt: Vec<&ScannedTrack> = result.tracks.iter().filter(|t| t.is_cue_virtual).collect();
        assert_eq!(virt.len(), 3, "external CUE should take priority (3 tracks) not embedded (2 tracks)");

        assert_eq!(virt[0].title, "Overridden T1");
        assert_eq!(virt[0].album.as_deref(), Some("External Album"));
        assert_eq!(virt[0].artist.as_deref(), Some("External Artist"));
    }

    #[test]
    fn test_compute_byte_offset() {
        let metadata = AudioMetadata {
            bitrate: 1411,
            duration: 10.0,
            sample_rate: 44100,
            channels: 2,
            ..Default::default()
        };

        // At t=0, offset should be 0
        assert_eq!(compute_byte_offset(0.0, &metadata, 1764000), 0);

        // At t=1s: 1411 * 1000 / 8 = 176375 bytes
        let offset_1s = compute_byte_offset(1.0, &metadata, 1764000);
        assert!((offset_1s as i64 - 176375).abs() < 1000);

        // At t=5s
        let offset_5s = compute_byte_offset(5.0, &metadata, 1764000);
        assert!((offset_5s as i64 - 881875).abs() < 5000);

        // Fallback when bitrate=0: use file_size/duration
        let meta_zero = AudioMetadata {
            bitrate: 0,
            duration: 10.0,
            ..Default::default()
        };
        let fallback = compute_byte_offset(5.0, &meta_zero, 1000000);
        // 5/10 * 1000000 = 500000
        assert!((fallback as i64 - 500000).abs() < 1000);
    }
}
