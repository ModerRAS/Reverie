//! 核心响应类型
//!
//! 包含 SubsonicResponse、SubsonicResponseInner 和 ResponseData 枚举

use reverie_core::SUBSONIC_API_VERSION;
use serde::Serialize;

/// JSON 格式的 Subsonic 响应主包装器
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponse {
    #[serde(rename = "subsonic-response")]
    pub inner: SubsonicResponseInner,
}

impl SubsonicResponse {
    pub fn ok() -> Self {
        Self {
            inner: SubsonicResponseInner::ok(),
        }
    }

    pub fn ok_with<T: Into<ResponseData>>(data: T) -> Self {
        Self {
            inner: SubsonicResponseInner::ok_with(data),
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            inner: SubsonicResponseInner::error(code, message),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubsonicResponseInner {
    pub status: String,
    pub version: String,
    #[serde(rename = "type")]
    pub server_type: String,
    pub server_version: String,
    pub open_subsonic: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorResponse>,
    #[serde(flatten)]
    pub data: Option<ResponseData>,
}

impl SubsonicResponseInner {
    pub fn ok() -> Self {
        Self {
            status: "ok".to_string(),
            version: SUBSONIC_API_VERSION.to_string(),
            server_type: "reverie".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            open_subsonic: true,
            error: None,
            data: None,
        }
    }

    pub fn ok_with<T: Into<ResponseData>>(data: T) -> Self {
        Self {
            data: Some(data.into()),
            ..Self::ok()
        }
    }

    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            status: "failed".to_string(),
            version: SUBSONIC_API_VERSION.to_string(),
            server_type: "reverie".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            open_subsonic: true,
            error: Some(ErrorResponse {
                code,
                message: message.into(),
            }),
            data: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
}

/// 响应数据 - 每个变体使用自己的键进行序列化
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ResponseData {
    License(LicenseData),
    MusicFolders(MusicFoldersData),
    Indexes(IndexesData),
    Artists(ArtistsData),
    Artist(ArtistData),
    Album(AlbumData),
    Song(SongData),
    Directory(DirectoryData),
    Genres(GenresData),
    AlbumList(AlbumListData),
    AlbumList2(AlbumList2Data),
    RandomSongs(RandomSongsData),
    SongsByGenre(SongsByGenreData),
    NowPlaying(NowPlayingData),
    Starred(StarredData),
    Starred2(Starred2Data),
    SearchResult2(SearchResult2Data),
    SearchResult3(SearchResult3Data),
    Playlists(PlaylistsData),
    Playlist(PlaylistData),
    User(UserData),
    Users(UsersData),
    Bookmarks(BookmarksData),
    PlayQueue(PlayQueueData),
    Shares(SharesData),
    InternetRadioStations(InternetRadioStationsData),
    Lyrics(LyricsData),
    LyricsList(LyricsListData),
    ScanStatus(ScanStatusData),
    ArtistInfo(ArtistInfoData),
    ArtistInfo2(ArtistInfo2Data),
    AlbumInfo(AlbumInfoData),
    SimilarSongs(SimilarSongsData),
    SimilarSongs2(SimilarSongs2Data),
    TopSongs(TopSongsData),
    OpenSubsonicExtensions(OpenSubsonicExtensionsData),
}

// Re-export types from other modules
pub use super::{
    AlbumData, AlbumID3Item, AlbumList2Data, AlbumListData, AlbumWithSongs, ArtistData,
    ArtistID3Item, ArtistInfo2Data, ArtistInfoData, ArtistItem, ArtistWithAlbums, ArtistsData,
    BookmarkItem, BookmarksData, Child, DirectoryData, DirectoryItem, GenreItem, GenresData,
    InternetRadioStationItem, InternetRadioStationsData, LicenseData, LyricsData, LyricsListData,
    MusicFolderItem, MusicFoldersData, NowPlayingData, OpenSubsonicExtensionItem,
    OpenSubsonicExtensionsData, PlayQueueData, PlaylistData, PlaylistItem, PlaylistWithEntries,
    PlaylistsData, ScanStatusData, SearchResult2Data, SearchResult3Data, ShareItem, SharesData,
    SimilarSongsData, SongData, SongsByGenreData, Starred2Data, StarredData, TopSongsData,
    UserData, UserItem, UsersData,
};
