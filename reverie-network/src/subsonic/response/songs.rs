//! 歌曲/媒体文件相关 DTO 类型

use reverie_core::MediaFile;
use serde::Serialize;

// === 子项 (歌曲/媒体文件) ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Child {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub is_dir: bool,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bit_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disc_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    pub is_video: bool,
}

impl From<&MediaFile> for Child {
    fn from(m: &MediaFile) -> Self {
        Self {
            id: m.id.clone(),
            parent: m.parent.clone(),
            is_dir: m.is_dir,
            title: m.title.clone(),
            album: m.album.clone(),
            artist: m.artist.clone(),
            track: m.track_number,
            year: m.year,
            genre: m.genre.clone(),
            cover_art: m.cover_art.clone(),
            size: Some(m.size),
            content_type: Some(m.content_type.clone()),
            suffix: Some(m.suffix.clone()),
            duration: Some(m.duration as i32),
            bit_rate: Some(m.bit_rate),
            path: Some(m.path.clone()),
            play_count: m.play_count,
            disc_number: m.disc_number,
            created: m.created.map(|d| d.to_rfc3339()),
            album_id: m.album_id.clone(),
            artist_id: m.artist_id.clone(),
            starred: m.starred.map(|d| d.to_rfc3339()),
            user_rating: m.user_rating,
            media_type: if m.r#type.is_empty() {
                Some("music".to_string())
            } else {
                Some(m.r#type.clone())
            },
            is_video: false,
        }
    }
}

// === 单个歌曲 ===
#[derive(Debug, Clone, Serialize)]
pub struct SongData {
    pub song: Child,
}

impl From<SongData> for super::ResponseData {
    fn from(v: SongData) -> Self {
        super::ResponseData::Song(v)
    }
}

// === 目录 ===
#[derive(Debug, Clone, Serialize)]
pub struct DirectoryData {
    pub directory: DirectoryInner,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryInner {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Child>,
}

impl From<&reverie_core::SubsonicDirectory> for DirectoryInner {
    fn from(d: &reverie_core::SubsonicDirectory) -> Self {
        Self {
            id: d.id.clone(),
            parent: d.parent.clone(),
            name: d.name.clone(),
            starred: d.starred.map(|dt| dt.to_rfc3339()),
            user_rating: d.user_rating,
            play_count: d.play_count,
            child: d.children.iter().map(Child::from).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryItem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Child>,
}

impl From<DirectoryData> for super::ResponseData {
    fn from(v: DirectoryData) -> Self {
        super::ResponseData::Directory(v)
    }
}

// === 随机歌曲 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomSongsData {
    pub random_songs: RandomSongsInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct RandomSongsInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<RandomSongsData> for super::ResponseData {
    fn from(v: RandomSongsData) -> Self {
        super::ResponseData::RandomSongs(v)
    }
}

// === 按流派歌曲 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongsByGenreData {
    pub songs_by_genre: SongsByGenreInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongsByGenreInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<SongsByGenreData> for super::ResponseData {
    fn from(v: SongsByGenreData) -> Self {
        super::ResponseData::SongsByGenre(v)
    }
}

// === 正在播放 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingData {
    pub now_playing: NowPlayingInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct NowPlayingInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<NowPlayingEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NowPlayingEntry {
    pub username: String,
    pub minutes_ago: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_name: Option<String>,
    #[serde(flatten)]
    pub song: Child,
}

impl From<&reverie_core::SubsonicNowPlaying> for NowPlayingEntry {
    fn from(n: &reverie_core::SubsonicNowPlaying) -> Self {
        Self {
            username: n.username.clone(),
            minutes_ago: n.minutes_ago,
            player_id: n.player_id.clone(),
            player_name: n.player_name.clone(),
            song: Child::from(&n.entry),
        }
    }
}

impl From<NowPlayingData> for super::ResponseData {
    fn from(v: NowPlayingData) -> Self {
        super::ResponseData::NowPlaying(v)
    }
}

// === 收藏 ===
#[derive(Debug, Clone, Serialize)]
pub struct StarredData {
    pub starred: StarredInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct StarredInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<super::ArtistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<StarredData> for super::ResponseData {
    fn from(v: StarredData) -> Self {
        super::ResponseData::Starred(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Starred2Data {
    pub starred2: Starred2Inner,
}

#[derive(Debug, Clone, Serialize)]
pub struct Starred2Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<super::ArtistID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<super::AlbumID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<Starred2Data> for super::ResponseData {
    fn from(v: Starred2Data) -> Self {
        super::ResponseData::Starred2(v)
    }
}

// === 搜索结果 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult2Data {
    pub search_result2: SearchResult2Inner,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult2Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<super::ArtistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<SearchResult2Data> for super::ResponseData {
    fn from(v: SearchResult2Data) -> Self {
        super::ResponseData::SearchResult2(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult3Data {
    pub search_result3: SearchResult3Inner,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult3Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<super::ArtistID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<super::AlbumID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<SearchResult3Data> for super::ResponseData {
    fn from(v: SearchResult3Data) -> Self {
        super::ResponseData::SearchResult3(v)
    }
}
