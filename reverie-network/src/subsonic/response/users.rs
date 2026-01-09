//! 用户相关 DTO 类型

use reverie_core::SubsonicUser;
use serde::Serialize;

// === 用户 ===
#[derive(Debug, Clone, Serialize)]
pub struct UserData {
    pub user: UserItem,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserItem {
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub scrobbling_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bit_rate: Option<i32>,
    pub admin_role: bool,
    pub settings_role: bool,
    pub download_role: bool,
    pub upload_role: bool,
    pub playlist_role: bool,
    pub cover_art_role: bool,
    pub comment_role: bool,
    pub podcast_role: bool,
    pub stream_role: bool,
    pub jukebox_role: bool,
    pub share_role: bool,
    pub video_conversion_role: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub folder: Vec<i32>,
}

impl From<&SubsonicUser> for UserItem {
    fn from(u: &SubsonicUser) -> Self {
        Self {
            username: u.username.clone(),
            email: u.email.clone(),
            scrobbling_enabled: u.scrobbling_enabled,
            max_bit_rate: u.max_bit_rate,
            admin_role: u.admin_role,
            settings_role: u.settings_role,
            download_role: u.download_role,
            upload_role: u.upload_role,
            playlist_role: u.playlist_role,
            cover_art_role: u.cover_art_role,
            comment_role: u.comment_role,
            podcast_role: u.podcast_role,
            stream_role: u.stream_role,
            jukebox_role: u.jukebox_role,
            share_role: u.share_role,
            video_conversion_role: u.video_conversion_role,
            folder: u.folders.clone(),
        }
    }
}

impl From<UserData> for super::ResponseData {
    fn from(v: UserData) -> Self {
        super::ResponseData::User(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersData {
    pub users: UsersInner,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user: Vec<UserItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user: Vec<UserItem>,
}

impl From<UsersData> for super::ResponseData {
    fn from(v: UsersData) -> Self {
        super::ResponseData::Users(v)
    }
}

// === 分享 ===
#[derive(Debug, Clone, Serialize)]
pub struct SharesData {
    pub shares: SharesList,
}

#[derive(Debug, Clone, Serialize)]
pub struct SharesList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub share: Vec<ShareItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareItem {
    pub id: String,
    pub url: String,
    pub username: String,
    pub created: String,
    pub expires: Option<String>,
    pub last_visited: Option<String>,
    pub visit_count: i32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<super::Child>,
}

impl From<&reverie_core::SubsonicShare> for ShareItem {
    fn from(s: &reverie_core::SubsonicShare) -> Self {
        Self {
            id: s.id.clone(),
            url: s.url.clone(),
            username: s.username.clone(),
            created: s.created.to_rfc3339(),
            expires: s.expires.map(|d| d.to_rfc3339()),
            last_visited: s.last_visited.map(|d| d.to_rfc3339()),
            visit_count: s.visit_count as i32,
            entry: s.entries.iter().map(super::Child::from).collect(),
        }
    }
}

impl From<SharesData> for super::ResponseData {
    fn from(v: SharesData) -> Self {
        super::ResponseData::Shares(v)
    }
}
