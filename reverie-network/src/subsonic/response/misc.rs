//! 其他 DTO 类型

use reverie_core::{SubsonicBookmark, SubsonicGenre, SubsonicInternetRadioStation, SubsonicLyrics, SubsonicPlayQueue, SubsonicScanStatus, SubsonicStructuredLyrics};
use serde::Serialize;

// === 许可证 ===
#[derive(Debug, Clone, Serialize)]
pub struct LicenseData {
    pub license: License,
}

#[derive(Debug, Clone, Serialize)]
pub struct License {
    pub valid: bool,
}

impl From<LicenseData> for super::ResponseData {
    fn from(v: LicenseData) -> Self {
        super::ResponseData::License(v)
    }
}

// === 流派 ===
#[derive(Debug, Clone, Serialize)]
pub struct GenresData {
    pub genres: GenresList,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenresList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genre: Vec<GenreItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenreItem {
    pub value: String,
    pub song_count: i32,
    pub album_count: i32,
}

impl From<&SubsonicGenre> for GenreItem {
    fn from(g: &SubsonicGenre) -> Self {
        Self {
            value: g.name.clone(),
            song_count: g.song_count,
            album_count: g.album_count,
        }
    }
}

impl From<GenresData> for super::ResponseData {
    fn from(v: GenresData) -> Self {
        super::ResponseData::Genres(v)
    }
}

// === 书签 ===
#[derive(Debug, Clone, Serialize)]
pub struct BookmarksData {
    pub bookmarks: BookmarksList,
}

#[derive(Debug, Clone, Serialize)]
pub struct BookmarksList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bookmark: Vec<BookmarkItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkItem {
    pub position: i64,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub created: String,
    pub changed: String,
    pub entry: super::Child,
}

impl From<&SubsonicBookmark> for BookmarkItem {
    fn from(b: &SubsonicBookmark) -> Self {
        Self {
            position: b.position,
            username: b.username.clone(),
            comment: b.comment.clone(),
            created: b.created.to_rfc3339(),
            changed: b.changed.to_rfc3339(),
            entry: super::Child::from(&b.entry),
        }
    }
}

impl From<BookmarksData> for super::ResponseData {
    fn from(v: BookmarksData) -> Self {
        super::ResponseData::Bookmarks(v)
    }
}

// === 播放队列 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayQueueData {
    pub play_queue: PlayQueueInner,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayQueueInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    pub position: i64,
    pub username: String,
    pub changed: String,
    pub changed_by: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<super::Child>,
}

impl From<&SubsonicPlayQueue> for PlayQueueInner {
    fn from(p: &SubsonicPlayQueue) -> Self {
        Self {
            current: p.current.clone(),
            position: p.position,
            username: p.username.clone(),
            changed: p.changed.to_rfc3339(),
            changed_by: p.changed_by.clone(),
            entry: p.entries.iter().map(super::Child::from).collect(),
        }
    }
}

impl From<PlayQueueData> for super::ResponseData {
    fn from(v: PlayQueueData) -> Self {
        super::ResponseData::PlayQueue(v)
    }
}

// === 互联网广播 ===
#[derive(Debug, Clone, Serialize)]
pub struct InternetRadioStationsData {
    pub internet_radio_stations: InternetRadioStationsList,
}

#[derive(Debug, Clone, Serialize)]
pub struct InternetRadioStationsList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub internet_radio_station: Vec<InternetRadioStationItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternetRadioStationItem {
    pub id: String,
    pub name: String,
    pub stream_url: String,
    pub home_page_url: Option<String>,
}

impl From<&SubsonicInternetRadioStation> for InternetRadioStationItem {
    fn from(s: &SubsonicInternetRadioStation) -> Self {
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            stream_url: s.stream_url.clone(),
            home_page_url: s.home_page_url.clone(),
        }
    }
}

impl From<InternetRadioStationsData> for super::ResponseData {
    fn from(v: InternetRadioStationsData) -> Self {
        super::ResponseData::InternetRadioStations(v)
    }
}

// === 歌词 ===
#[derive(Debug, Clone, Serialize)]
pub struct LyricsData {
    pub lyrics: LyricsItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsItem {
    pub artist: Option<String>,
    pub title: Option<String>,
    pub value: String,
}

impl From<&SubsonicLyrics> for LyricsItem {
    fn from(l: &SubsonicLyrics) -> Self {
        Self {
            artist: l.artist.clone(),
            title: l.title.clone(),
            value: l.value.clone(),
        }
    }
}

impl From<LyricsData> for super::ResponseData {
    fn from(v: LyricsData) -> Self {
        super::ResponseData::Lyrics(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LyricsListData {
    pub lyrics_list: LyricsListInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct LyricsListInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lyrics: Vec<StructuredLyricsItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredLyricsItem {
    pub artist: String,
    pub title: String,
    pub content: String,
}

impl From<&SubsonicStructuredLyrics> for StructuredLyricsItem {
    fn from(l: &SubsonicStructuredLyrics) -> Self {
        Self {
            artist: l.artist.clone(),
            title: l.title.clone(),
            content: l.content.clone(),
        }
    }
}

impl From<LyricsListData> for super::ResponseData {
    fn from(v: LyricsListData) -> Self {
        super::ResponseData::LyricsList(v)
    }
}

// === 扫描状态 ===
#[derive(Debug, Clone, Serialize)]
pub struct ScanStatusData {
    pub scan_status: ScanStatusItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusItem {
    pub scanning: bool,
    pub count: i64,
}

impl From<&SubsonicScanStatus> for ScanStatusItem {
    fn from(s: &SubsonicScanStatus) -> Self {
        Self {
            scanning: s.scanning,
            count: s.count,
        }
    }
}

impl From<ScanStatusData> for super::ResponseData {
    fn from(v: ScanStatusData) -> Self {
        super::ResponseData::ScanStatus(v)
    }
}

// === OpenSubsonic 扩展 ===
#[derive(Debug, Clone, Serialize)]
pub struct OpenSubsonicExtensionsData {
    pub open_subsonic_extensions: OpenSubsonicExtensionsList,
}

#[derive(Debug, Clone, Serialize)]
pub struct OpenSubsonicExtensionsList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extension: Vec<OpenSubsonicExtensionItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSubsonicExtensionItem {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

impl From<&SubsonicOpenSubsonicExtension> for OpenSubsonicExtensionItem {
    fn from(e: &SubsonicOpenSubsonicExtension) -> Self {
        Self {
            name: e.name.clone(),
            version: e.version.clone(),
            description: e.description.clone(),
        }
    }
}

impl From<OpenSubsonicExtensionsData> for super::ResponseData {
    fn from(v: OpenSubsonicExtensionsData) -> Self {
        super::ResponseData::OpenSubsonicExtensions(v)
    }
}
