//! Video-related response DTOs

use reverie_core::MediaFile;
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
