//! Podcast response DTOs

use reverie_core::{PodcastChannel, PodcastEpisode};
use serde::Serialize;

/// Podcasts wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PodcastsData {
    pub podcasts: PodcastsInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct PodcastsInner {
    pub channel: Vec<PodcastChannelItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PodcastChannelItem {
    pub id: String,
    pub url: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "coverArt", skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub status: String,
    #[serde(rename = "errorMessage", skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episodes: Option<PodcastEpisodesInner>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PodcastEpisodesInner {
    #[serde(rename = "episode")]
    pub episodes: Vec<PodcastEpisodeItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PodcastEpisodeItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "publishDate", skip_serializing_if = "Option::is_none")]
    pub publish_date: Option<String>,
    pub status: String,
    #[serde(rename = "streamId", skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

impl From<&PodcastChannel> for PodcastChannelItem {
    fn from(c: &PodcastChannel) -> Self {
        Self {
            id: c.id.clone(),
            url: c.url.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            cover_art: c.cover_art.clone(),
            status: c.status.clone(),
            error_message: c.error_message.clone(),
            episodes: None,
        }
    }
}

impl From<&PodcastEpisode> for PodcastEpisodeItem {
    fn from(e: &PodcastEpisode) -> Self {
        Self {
            id: e.id.clone(),
            title: e.title.clone(),
            description: e.description.clone(),
            publish_date: e.publish_date.map(|dt| dt.to_rfc3339()),
            status: e.status.clone(),
            stream_id: e.stream_id.clone(),
            duration: e.duration,
            size: e.size,
        }
    }
}

impl From<PodcastsData> for super::ResponseData {
    fn from(v: PodcastsData) -> Self {
        super::ResponseData::Podcasts(v)
    }
}

/// Newest podcasts wrapper
#[derive(Debug, Clone, Serialize)]
pub struct NewestPodcastsData {
    #[serde(rename = "newestPodcasts")]
    pub newest_podcasts: NewestPodcastsInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct NewestPodcastsInner {
    #[serde(rename = "episode")]
    pub episodes: Vec<PodcastEpisodeItem>,
}

impl From<NewestPodcastsData> for super::ResponseData {
    fn from(v: NewestPodcastsData) -> Self {
        super::ResponseData::NewestPodcasts(v)
    }
}
