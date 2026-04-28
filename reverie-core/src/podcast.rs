//! Podcast domain models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 播客频道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastChannel {
    /// Unique identifier
    pub id: String,
    /// RSS feed URL
    pub url: String,
    /// Channel title
    pub title: String,
    /// Channel description
    pub description: Option<String>,
    /// Cover art ID
    pub cover_art: Option<String>,
    /// Channel status (completed/error)
    pub status: String,
    /// Error message if status is error
    pub error_message: Option<String>,
    /// Creation timestamp
    pub created: DateTime<Utc>,
    /// Last refresh timestamp
    pub last_refresh: Option<DateTime<Utc>>,
}

impl PodcastChannel {
    pub fn new(id: String, url: String, title: String) -> Self {
        Self {
            id,
            url,
            title,
            description: None,
            cover_art: None,
            status: "completed".to_string(),
            error_message: None,
            created: Utc::now(),
            last_refresh: None,
        }
    }
}

/// 播客单集
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodcastEpisode {
    /// Unique identifier
    pub id: String,
    /// Parent channel ID
    pub channel_id: String,
    /// Episode title
    pub title: String,
    /// Episode description
    pub description: Option<String>,
    /// Publish date
    pub publish_date: Option<DateTime<Utc>>,
    /// Episode status (completed/downloading/skipped/error/deleted)
    pub status: String,
    /// Audio stream ID
    pub stream_id: Option<String>,
    /// Duration in seconds
    pub duration: Option<i32>,
    /// File size in bytes
    pub size: Option<i64>,
    /// Episode file URL
    pub url: Option<String>,
    /// Cover art ID
    pub cover_art: Option<String>,
}

impl PodcastEpisode {
    pub fn new(id: String, channel_id: String, title: String) -> Self {
        Self {
            id,
            channel_id,
            title,
            description: None,
            publish_date: None,
            status: "completed".to_string(),
            stream_id: None,
            duration: None,
            size: None,
            url: None,
            cover_art: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_podcast_channel_creation() {
        let channel = PodcastChannel::new(
            "channel-1".to_string(),
            "https://example.com/feed.xml".to_string(),
            "My Podcast".to_string(),
        );
        assert_eq!(channel.id, "channel-1");
        assert_eq!(channel.url, "https://example.com/feed.xml");
        assert_eq!(channel.title, "My Podcast");
        assert_eq!(channel.status, "completed");
        assert!(channel.error_message.is_none());
    }

    #[test]
    fn test_podcast_episode_creation() {
        let episode = PodcastEpisode::new(
            "episode-1".to_string(),
            "channel-1".to_string(),
            "Episode 1".to_string(),
        );
        assert_eq!(episode.id, "episode-1");
        assert_eq!(episode.channel_id, "channel-1");
        assert_eq!(episode.title, "Episode 1");
        assert_eq!(episode.status, "completed");
    }
}
