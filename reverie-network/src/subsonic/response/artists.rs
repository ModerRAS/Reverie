//! 艺术家相关 DTO 类型

use reverie_core::{SubsonicArtist, SubsonicArtistInfo, SubsonicArtistIndex};
use serde::Serialize;

// === 音乐文件夹 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFoldersData {
    pub music_folders: MusicFoldersList,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicFoldersList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub music_folder: Vec<MusicFolderItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MusicFolderItem {
    pub id: i32,
    pub name: String,
}

impl From<&reverie_core::SubsonicMusicFolder> for MusicFolderItem {
    fn from(f: &reverie_core::SubsonicMusicFolder) -> Self {
        Self {
            id: f.id,
            name: f.name.clone(),
        }
    }
}

impl From<MusicFoldersData> for super::ResponseData {
    fn from(v: MusicFoldersData) -> Self {
        super::ResponseData::MusicFolders(v)
    }
}

// === 索引 ===
#[derive(Debug, Clone, Serialize)]
pub struct IndexesData {
    pub indexes: IndexesList,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexesList {
    pub last_modified: i64,
    pub ignored_articles: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index: Vec<IndexItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistItem>,
}

impl From<IndexesData> for super::ResponseData {
    fn from(v: IndexesData) -> Self {
        super::ResponseData::Indexes(v)
    }
}

// === 艺术家 (ID3) ===
#[derive(Debug, Clone, Serialize)]
pub struct ArtistsData {
    pub artists: ArtistsList,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistsList {
    pub last_modified: i64,
    pub ignored_articles: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub index: Vec<ArtistIndexItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtistIndexItem {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistID3Item>,
}

impl From<ArtistsData> for super::ResponseData {
    fn from(v: ArtistsData) -> Self {
        super::ResponseData::Artists(v)
    }
}

// === 艺术家 (基于文件夹) ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistItem {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
}

impl From<&SubsonicArtist> for ArtistItem {
    fn from(a: &SubsonicArtist) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            cover_art: a.cover_art.clone(),
            artist_image_url: None,
            starred: a.starred.map(|d| d.to_rfc3339()),
            user_rating: a.user_rating,
        }
    }
}

// === 艺术家 ID3 ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistID3Item {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub album_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_rating: Option<i32>,
}

impl From<&SubsonicArtist> for ArtistID3Item {
    fn from(a: &SubsonicArtist) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            cover_art: a.cover_art.clone(),
            album_count: a.album_count,
            artist_image_url: None,
            starred: a.starred.map(|d| d.to_rfc3339()),
            user_rating: a.user_rating,
        }
    }
}

// === 单个艺术家及其专辑 ===
#[derive(Debug, Clone, Serialize)]
pub struct ArtistData {
    pub artist: ArtistWithAlbums,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistWithAlbums {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_art: Option<String>,
    pub album_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starred: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3Item>,
}

impl From<ArtistData> for super::ResponseData {
    fn from(v: ArtistData) -> Self {
        super::ResponseData::Artist(v)
    }
}

impl From<&SubsonicArtist> for ArtistWithAlbums {
    fn from(a: &SubsonicArtist) -> Self {
        Self {
            id: a.id.clone(),
            name: a.name.clone(),
            cover_art: a.cover_art.clone(),
            album_count: a.album_count,
            starred: a.starred.map(|d| d.to_rfc3339()),
            // Note: albums need to be populated separately via get_artist storage call
            album: Vec::new(),
        }
    }
}

// === 艺术家信息 ===
#[derive(Debug, Clone, Serialize)]
pub struct ArtistInfoData {
    pub artist_info: ArtistInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo {
    pub biography: Option<String>,
    pub music_brainz_id: Option<String>,
    pub last_fm_url: Option<String>,
    pub small_url: Option<String>,
    pub medium_url: Option<String>,
    pub large_url: Option<String>,
    pub similar_artist: Vec<ArtistID3Item>,
}

impl From<ArtistInfoData> for super::ResponseData {
    fn from(v: ArtistInfoData) -> Self {
        super::ResponseData::ArtistInfo(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtistInfo2Data {
    pub artist_info2: ArtistInfo2,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo2 {
    pub biography: Option<String>,
    pub links: Vec<LinkItem>,
    pub image: Vec<ImageItem>,
    pub similar_artist: Vec<ArtistID3Item>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkItem {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageItem {
    pub url: String,
}

impl From<ArtistInfo2Data> for super::ResponseData {
    fn from(v: ArtistInfo2Data) -> Self {
        super::ResponseData::ArtistInfo2(v)
    }
}

impl From<&SubsonicArtistInfo> for ArtistInfo {
    fn from(a: &SubsonicArtistInfo) -> Self {
        Self {
            biography: a.biography.clone(),
            music_brainz_id: a.music_brainz_id.clone(),
            last_fm_url: a.last_fm_url.clone(),
            small_url: a.small_url.clone(),
            medium_url: a.medium_url.clone(),
            large_url: a.large_url.clone(),
            similar_artist: a
                .similar_artist
                .iter()
                .map(ArtistID3Item::from)
                .collect(),
        }
    }
}
