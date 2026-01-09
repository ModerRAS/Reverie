//! 音频元数据提取
//!
//! 使用 lofty 库从音频文件中提取 ID3 标签和元数据

use std::io::Cursor;
use std::path::Path;

use crate::error::{Result, StorageError};

/// 从音频文件提取的元数据
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// 标题
    pub title: Option<String>,
    /// 艺术家
    pub artist: Option<String>,
    /// 专辑
    pub album: Option<String>,
    /// 专辑艺术家
    pub album_artist: Option<String>,
    /// 年份
    pub year: Option<i32>,
    /// 流派
    pub genre: Option<String>,
    /// 音轨号
    pub track_number: Option<i32>,
    /// 音轨总数
    pub track_total: Option<i32>,
    /// 碟片号
    pub disc_number: Option<i32>,
    /// 碟片总数
    pub disc_total: Option<i32>,
    /// 时长（秒）
    pub duration: f32,
    /// 比特率 (kbps)
    pub bitrate: i32,
    /// 采样率 (Hz)
    pub sample_rate: i32,
    /// 声道数
    pub channels: i32,
    /// 是否包含封面图片
    pub has_cover: bool,
    /// 封面图片数据（如果有）
    pub cover_data: Option<Vec<u8>>,
    /// 封面 MIME 类型
    pub cover_mime: Option<String>,
}

impl AudioMetadata {
    /// 从文件路径提取元数据
    #[cfg(feature = "scanner")]
    pub fn from_path(path: &Path) -> Result<Self> {
        use lofty::prelude::*;
        use lofty::probe::Probe;

        let tagged_file = Probe::open(path)
            .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
            .read()
            .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Self::extract_metadata(&tagged_file)
    }

    /// 从内存数据提取元数据
    #[cfg(feature = "scanner")]
    pub fn from_bytes(data: &[u8], file_type_hint: Option<&str>) -> Result<Self> {
        use lofty::prelude::*;
        use lofty::probe::Probe;

        let cursor = Cursor::new(data);
        let mut probe = Probe::new(cursor);
        
        // 根据文件扩展名提示设置文件类型
        if let Some(hint) = file_type_hint {
            if let Some(ft) = lofty::file::FileType::from_ext(hint) {
                probe = probe.set_file_type(ft);
            }
        }

        let tagged_file = probe
            .read()
            .map_err(|e| StorageError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        Self::extract_metadata(&tagged_file)
    }

    #[cfg(feature = "scanner")]
    fn extract_metadata(tagged_file: &lofty::file::TaggedFile) -> Result<Self> {
        use lofty::prelude::*;
        use lofty::picture::PictureType;

        let properties = tagged_file.properties();
        let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

        let mut metadata = AudioMetadata {
            duration: properties.duration().as_secs_f32(),
            bitrate: properties.audio_bitrate().unwrap_or(0) as i32,
            sample_rate: properties.sample_rate().unwrap_or(44100) as i32,
            channels: properties.channels().unwrap_or(2) as i32,
            ..Default::default()
        };

        if let Some(tag) = tag {
            metadata.title = tag.title().map(|s| s.to_string());
            metadata.artist = tag.artist().map(|s| s.to_string());
            metadata.album = tag.album().map(|s| s.to_string());
            metadata.year = tag.year().map(|y| y as i32);
            metadata.genre = tag.genre().map(|s| s.to_string());
            metadata.track_number = tag.track().map(|t| t as i32);
            metadata.track_total = tag.track_total().map(|t| t as i32);
            metadata.disc_number = tag.disk().map(|d| d as i32);
            metadata.disc_total = tag.disk_total().map(|d| d as i32);

            // 获取专辑艺术家（如果有）
            // 先尝试从 ItemKey 获取，如果没有则使用 artist
            metadata.album_artist = tag
                .get_string(&lofty::tag::ItemKey::AlbumArtist)
                .map(|s| s.to_string())
                .or_else(|| metadata.artist.clone());

            // 提取封面图片
            let cover = tag.pictures().iter().find(|p| {
                matches!(
                    p.pic_type(),
                    PictureType::CoverFront | PictureType::Other | PictureType::Media
                )
            }).or_else(|| tag.pictures().first());

            if let Some(picture) = cover {
                metadata.has_cover = true;
                metadata.cover_data = Some(picture.data().to_vec());
                metadata.cover_mime = picture.mime_type().map(|m| m.to_string());
            }
        }

        Ok(metadata)
    }

    /// 不使用 scanner feature 时的空实现
    #[cfg(not(feature = "scanner"))]
    pub fn from_path(_path: &Path) -> Result<Self> {
        Err(StorageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Scanner feature not enabled",
        )))
    }

    #[cfg(not(feature = "scanner"))]
    pub fn from_bytes(_data: &[u8], _file_type_hint: Option<&str>) -> Result<Self> {
        Err(StorageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Scanner feature not enabled",
        )))
    }
}

/// 判断文件是否为支持的音频格式
pub fn is_audio_file(path: &str) -> bool {
    let path = path.to_lowercase();
    SUPPORTED_AUDIO_EXTENSIONS
        .iter()
        .any(|ext| path.ends_with(ext))
}

/// 支持的音频文件扩展名
pub const SUPPORTED_AUDIO_EXTENSIONS: &[&str] = &[
    ".mp3", ".flac", ".ogg", ".opus", ".m4a", ".aac", ".wav", ".wma", ".aiff", ".ape", ".wv",
];

/// 获取文件扩展名（不含点）
pub fn get_extension(path: &str) -> Option<&str> {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file("/music/song.mp3"));
        assert!(is_audio_file("/music/album/track.flac"));
        assert!(is_audio_file("SONG.MP3")); // 大小写不敏感
        assert!(!is_audio_file("/music/cover.jpg"));
        assert!(!is_audio_file("/music/playlist.m3u"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("song.mp3"), Some("mp3"));
        assert_eq!(get_extension("/path/to/track.flac"), Some("flac"));
        assert_eq!(get_extension("no_extension"), None);
    }
}
