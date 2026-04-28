//! Video-related response DTOs

use reverie_core::{Caption, MediaFile, VideoInfo};
use serde::Serialize;

use super::Child;

/// Wrapper for videos response (Subsonic API: <videos><video>...</video></videos>)
#[derive(Debug, Clone, Serialize)]
pub struct VideosData {
    pub video: Vec<Child>,
}

impl From<Vec<MediaFile>> for VideosData {
    fn from(videos: Vec<MediaFile>) -> Self {
        Self {
            video: videos.iter().map(Child::from).collect(),
        }
    }
}

impl From<VideosData> for super::ResponseData {
    fn from(v: VideosData) -> Self {
        super::ResponseData::Videos(v)
    }
}

/// VideoInfo response DTO
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfoData {
    pub video_info: VideoInfoItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfoItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_track_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

impl From<&VideoInfo> for VideoInfoItem {
    fn from(info: &VideoInfo) -> Self {
        Self {
            id: info.id.clone(),
            title: info.title.clone(),
            path: info.path.clone(),
            cover_art: info.cover_art.clone(),
            original_width: info.original_width,
            original_height: info.original_height,
            audio_track_id: info.audio_track_id.clone(),
            duration: info.duration,
            bit_rate: info.bit_rate,
            created: info.created.map(|dt| dt.to_rfc3339()),
        }
    }
}

impl From<VideoInfoData> for super::ResponseData {
    fn from(v: VideoInfoData) -> Self {
        super::ResponseData::VideoInfo(v)
    }
}

/// Captions response wrapper (Subsonic API: <captions><caption>...</caption></captions>)
#[derive(Debug, Clone, Serialize)]
pub struct CaptionsData {
    #[serde(rename = "captions")]
    pub captions: CaptionsInner,
}

/// Inner wrapper for caption items
#[derive(Debug, Clone, Serialize)]
pub struct CaptionsInner {
    #[serde(rename = "caption")]
    pub caption: Vec<CaptionItem>,
}

/// Individual caption track item
#[derive(Debug, Clone, Serialize)]
pub struct CaptionItem {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub format: String,
}

impl From<&Caption> for CaptionItem {
    fn from(c: &Caption) -> Self {
        Self {
            id: c.id.clone(),
            name: c.name.clone(),
            language: c.language.clone(),
            format: c.format.clone(),
        }
    }
}

impl From<CaptionsData> for super::ResponseData {
    fn from(v: CaptionsData) -> Self {
        super::ResponseData::Captions(v)
    }
}
