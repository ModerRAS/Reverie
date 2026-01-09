//! 播放列表相关 DTO 类型

use reverie_core::{SubsonicPlaylist, SubsonicPlaylistWithSongs};
use serde::Serialize;

// === 播放列表 ===
#[derive(Debug, Clone, Serialize)]
pub struct PlaylistsData {
    pub playlists: PlaylistsList,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaylistsList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub playlist: Vec<PlaylistItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub owner: String,
    pub public: bool,
    pub song_count: i32,
    pub duration: i32,
    pub created: String,
    pub changed: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
}

impl From<&SubsonicPlaylist> for PlaylistItem {
    fn from(p: &SubsonicPlaylist) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            comment: p.comment.clone(),
            owner: p.owner.clone(),
            public: p.public,
            song_count: p.song_count,
            duration: p.duration,
            created: p.created.to_rfc3339(),
            changed: p.changed.to_rfc3339(),
            cover_art: p.cover_art.clone(),
        }
    }
}

impl From<PlaylistsData> for super::ResponseData {
    fn from(v: PlaylistsData) -> Self {
        super::ResponseData::Playlists(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaylistData {
    pub playlist: PlaylistWithEntries,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistWithEntries {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub owner: String,
    pub public: bool,
    pub song_count: i32,
    pub duration: i32,
    pub created: String,
    pub changed: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<super::Child>,
}

impl From<&SubsonicPlaylistWithSongs> for PlaylistWithEntries {
    fn from(p: &SubsonicPlaylistWithSongs) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            comment: p.comment.clone(),
            owner: p.owner.clone(),
            public: p.public,
            song_count: p.song_count,
            duration: p.duration,
            created: p.created.to_rfc3339(),
            changed: p.changed.to_rfc3339(),
            cover_art: p.cover_art.clone(),
            entry: p.entries.iter().map(super::Child::from).collect(),
        }
    }
}

impl From<PlaylistData> for super::ResponseData {
    fn from(v: PlaylistData) -> Self {
        super::ResponseData::Playlist(v)
    }
}
