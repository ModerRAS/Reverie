//! 专辑相关 DTO 类型

use reverie_core::SubsonicAlbum;
use serde::Serialize;

// === 专辑 ID3 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumID3Item {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub song_count: i32,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
}

impl From<&SubsonicAlbum> for AlbumID3Item {
    fn from(a: &SubsonicAlbum) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            artist: a.artist.clone().or_else(|| a.album_artist.clone()),
            artist_id: a.artist_id.clone().or_else(|| a.album_artist_id.clone()),
            cover_art: a.cover_art.clone(),
            song_count: a.song_count,
            duration: a.duration as i32,
            play_count: a.play_count,
            created: a.created.map(|d| d.to_rfc3339()),
            starred: a.starred.map(|d| d.to_rfc3339()),
            year: a.year,
            genre: a.genre.clone(),
        }
    }
}

// === 单个专辑及其歌曲 ===
#[derive(Debug, Clone, Serialize)]
pub struct AlbumData {
    pub album: AlbumWithSongs,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumWithSongs {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub song_count: i32,
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub play_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genre: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}

impl From<AlbumData> for super::ResponseData {
    fn from(v: AlbumData) -> Self {
        super::ResponseData::Album(v)
    }
}

impl From<&SubsonicAlbum> for AlbumWithSongs {
    fn from(a: &SubsonicAlbum) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            artist: a.artist.clone(),
            artist_id: a.artist_id.clone(),
            cover_art: a.cover_art.clone(),
            song_count: a.song_count,
            duration: a.duration as i32,
            play_count: a.play_count,
            created: a.created.map(|d| d.to_rfc3339()),
            starred: a.starred.map(|d| d.to_rfc3339()),
            year: a.year,
            genre: a.genre.clone(),
            // Note: songs need to be populated separately via get_album storage call
            song: Vec::new(),
        }
    }
}

// === 专辑列表 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumListData {
    pub album_list: AlbumListInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlbumListInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
}

impl From<AlbumListData> for super::ResponseData {
    fn from(v: AlbumListData) -> Self {
        super::ResponseData::AlbumList(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumList2Data {
    pub album_list2: AlbumList2Inner,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlbumList2Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3Item>,
}

impl From<AlbumList2Data> for super::ResponseData {
    fn from(v: AlbumList2Data) -> Self {
        super::ResponseData::AlbumList2(v)
    }
}

// === 专辑信息 ===
#[derive(Debug, Clone, Serialize)]
pub struct AlbumInfoData {
    pub album_info: AlbumInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfo {
    pub notes: Option<String>,
    pub music_brainz_id: Option<String>,
    pub last_fm_url: Option<String>,
    pub small_url: Option<String>,
    pub medium_url: Option<String>,
    pub large_url: Option<String>,
}

impl From<AlbumInfoData> for super::ResponseData {
    fn from(v: AlbumInfoData) -> Self {
        super::ResponseData::AlbumInfo(v)
    }
}

impl From<&reverie_core::SubsonicAlbumInfo> for AlbumInfo {
    fn from(a: &reverie_core::SubsonicAlbumInfo) -> Self {
        Self {
            notes: a.notes.clone(),
            music_brainz_id: a.music_brainz_id.clone(),
            last_fm_url: a.last_fm_url.clone(),
            small_url: a.small_url.clone(),
            medium_url: a.medium_url.clone(),
            large_url: a.large_url.clone(),
        }
    }
}

// === 相似歌曲 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongsData {
    pub similar_songs: SimilarSongsInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimilarSongsInner {
    pub song: Vec<Child>,
}

impl From<SimilarSongsData> for super::ResponseData {
    fn from(v: SimilarSongsData) -> Self {
        super::ResponseData::SimilarSongs(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongs2Data {
    pub similar_songs2: SimilarSongs2Inner,
}

#[derive(Debug, Clone, Serialize)]
pub struct SimilarSongs2Inner {
    pub song: Vec<Child>,
}

impl From<SimilarSongs2Data> for super::ResponseData {
    fn from(v: SimilarSongs2Data) -> Self {
        super::ResponseData::SimilarSongs2(v)
    }
}

// === 热门歌曲 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongsData {
    pub top_songs: TopSongsInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopSongsInner {
    pub song: Vec<Child>,
}

impl From<TopSongsData> for super::ResponseData {
    fn from(v: TopSongsData) -> Self {
        super::ResponseData::TopSongs(v)
    }
}
