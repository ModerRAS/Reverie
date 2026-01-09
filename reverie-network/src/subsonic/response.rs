//! Subsonic API response types and serialization
//!
//! Supports both JSON and XML output formats as per Subsonic API spec.

use reverie_core::{
    MediaFile, SubsonicAlbum, SubsonicAlbumInfo, SubsonicArtist, SubsonicArtistIndex,
    SubsonicArtistInfo, SubsonicBookmark, SubsonicGenre, SubsonicInternetRadioStation,
    SubsonicLyrics, SubsonicMusicFolder, SubsonicNowPlaying, SubsonicOpenSubsonicExtension,
    SubsonicPlayQueue, SubsonicPlaylist, SubsonicPlaylistWithSongs, SubsonicScanStatus,
    SubsonicShare, SubsonicStructuredLyrics, SubsonicUser, SUBSONIC_API_VERSION,
};
use serde::Serialize;

/// Main Subsonic response wrapper for JSON format
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

/// Response data - each variant serializes with its own key
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

// === License ===
#[derive(Debug, Clone, Serialize)]
pub struct LicenseData {
    pub license: License,
}
#[derive(Debug, Clone, Serialize)]
pub struct License {
    pub valid: bool,
}
impl From<LicenseData> for ResponseData {
    fn from(v: LicenseData) -> Self {
        ResponseData::License(v)
    }
}

// === Music Folders ===
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
impl From<&SubsonicMusicFolder> for MusicFolderItem {
    fn from(f: &SubsonicMusicFolder) -> Self {
        Self {
            id: f.id,
            name: f.name.clone(),
        }
    }
}
impl From<MusicFoldersData> for ResponseData {
    fn from(v: MusicFoldersData) -> Self {
        ResponseData::MusicFolders(v)
    }
}

// === Indexes ===
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
impl From<IndexesData> for ResponseData {
    fn from(v: IndexesData) -> Self {
        ResponseData::Indexes(v)
    }
}

// === Artists (ID3) ===
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
impl From<ArtistsData> for ResponseData {
    fn from(v: ArtistsData) -> Self {
        ResponseData::Artists(v)
    }
}

// === Artist (folder-based) ===
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

// === Artist ID3 ===
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

// === Single Artist with Albums ===
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
impl From<ArtistData> for ResponseData {
    fn from(v: ArtistData) -> Self {
        ResponseData::Artist(v)
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

// === Album ID3 ===
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

// === Single Album with Songs ===
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
impl From<AlbumData> for ResponseData {
    fn from(v: AlbumData) -> Self {
        ResponseData::Album(v)
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

// === Child (Song/MediaFile) ===
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

// === Single Song ===
#[derive(Debug, Clone, Serialize)]
pub struct SongData {
    pub song: Child,
}
impl From<SongData> for ResponseData {
    fn from(v: SongData) -> Self {
        ResponseData::Song(v)
    }
}

// === Directory ===
#[derive(Debug, Clone, Serialize)]
pub struct DirectoryData {
    pub directory: DirectoryItem,
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
impl From<DirectoryData> for ResponseData {
    fn from(v: DirectoryData) -> Self {
        ResponseData::Directory(v)
    }
}

// === Genres ===
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
impl From<GenresData> for ResponseData {
    fn from(v: GenresData) -> Self {
        ResponseData::Genres(v)
    }
}

// === Album Lists ===
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
impl From<AlbumListData> for ResponseData {
    fn from(v: AlbumListData) -> Self {
        ResponseData::AlbumList(v)
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
impl From<AlbumList2Data> for ResponseData {
    fn from(v: AlbumList2Data) -> Self {
        ResponseData::AlbumList2(v)
    }
}

// === Random Songs ===
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
impl From<RandomSongsData> for ResponseData {
    fn from(v: RandomSongsData) -> Self {
        ResponseData::RandomSongs(v)
    }
}

// === Songs by Genre ===
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
impl From<SongsByGenreData> for ResponseData {
    fn from(v: SongsByGenreData) -> Self {
        ResponseData::SongsByGenre(v)
    }
}

// === Now Playing ===
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
impl From<&SubsonicNowPlaying> for NowPlayingEntry {
    fn from(n: &SubsonicNowPlaying) -> Self {
        Self {
            username: n.username.clone(),
            minutes_ago: n.minutes_ago,
            player_id: n.player_id.clone(),
            player_name: n.player_name.clone(),
            song: Child::from(&n.entry),
        }
    }
}
impl From<NowPlayingData> for ResponseData {
    fn from(v: NowPlayingData) -> Self {
        ResponseData::NowPlaying(v)
    }
}

// === Starred ===
#[derive(Debug, Clone, Serialize)]
pub struct StarredData {
    pub starred: StarredInner,
}
#[derive(Debug, Clone, Serialize)]
pub struct StarredInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<StarredData> for ResponseData {
    fn from(v: StarredData) -> Self {
        ResponseData::Starred(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Starred2Data {
    pub starred2: Starred2Inner,
}
#[derive(Debug, Clone, Serialize)]
pub struct Starred2Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<Starred2Data> for ResponseData {
    fn from(v: Starred2Data) -> Self {
        ResponseData::Starred2(v)
    }
}

// === Search Results ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult2Data {
    pub search_result2: SearchResult2Inner,
}
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult2Inner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artist: Vec<ArtistItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<Child>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<SearchResult2Data> for ResponseData {
    fn from(v: SearchResult2Data) -> Self {
        ResponseData::SearchResult2(v)
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
    pub artist: Vec<ArtistID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub album: Vec<AlbumID3Item>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<SearchResult3Data> for ResponseData {
    fn from(v: SearchResult3Data) -> Self {
        ResponseData::SearchResult3(v)
    }
}

// === Playlists ===
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
impl From<PlaylistsData> for ResponseData {
    fn from(v: PlaylistsData) -> Self {
        ResponseData::Playlists(v)
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
    pub entry: Vec<Child>,
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
            entry: p.entries.iter().map(Child::from).collect(),
        }
    }
}
impl From<PlaylistData> for ResponseData {
    fn from(v: PlaylistData) -> Self {
        ResponseData::Playlist(v)
    }
}

// === Users ===
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
impl From<UserData> for ResponseData {
    fn from(v: UserData) -> Self {
        ResponseData::User(v)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersData {
    pub users: UsersList,
}
#[derive(Debug, Clone, Serialize)]
pub struct UsersList {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user: Vec<UserItem>,
}
impl From<UsersData> for ResponseData {
    fn from(v: UsersData) -> Self {
        ResponseData::Users(v)
    }
}

// === Bookmarks ===
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
    pub entry: Child,
}
impl From<&SubsonicBookmark> for BookmarkItem {
    fn from(b: &SubsonicBookmark) -> Self {
        Self {
            position: b.position,
            username: b.username.clone(),
            comment: b.comment.clone(),
            created: b.created.to_rfc3339(),
            changed: b.changed.to_rfc3339(),
            entry: Child::from(&b.entry),
        }
    }
}
impl From<BookmarksData> for ResponseData {
    fn from(v: BookmarksData) -> Self {
        ResponseData::Bookmarks(v)
    }
}

// === Play Queue ===
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
    pub entry: Vec<Child>,
}
impl From<&SubsonicPlayQueue> for PlayQueueInner {
    fn from(q: &SubsonicPlayQueue) -> Self {
        Self {
            current: q.current.clone(),
            position: q.position,
            username: q.username.clone(),
            changed: q.changed.to_rfc3339(),
            changed_by: q.changed_by.clone(),
            entry: q.entries.iter().map(Child::from).collect(),
        }
    }
}
impl From<PlayQueueData> for ResponseData {
    fn from(v: PlayQueueData) -> Self {
        ResponseData::PlayQueue(v)
    }
}

// === Shares ===
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub username: String,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_visited: Option<String>,
    pub visit_count: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entry: Vec<Child>,
}
impl From<&SubsonicShare> for ShareItem {
    fn from(s: &SubsonicShare) -> Self {
        Self {
            id: s.id.clone(),
            url: s.url.clone(),
            description: s.description.clone(),
            username: s.username.clone(),
            created: s.created.to_rfc3339(),
            expires: s.expires.map(|d| d.to_rfc3339()),
            last_visited: s.last_visited.map(|d| d.to_rfc3339()),
            visit_count: s.visit_count,
            entry: s.entries.iter().map(Child::from).collect(),
        }
    }
}
impl From<SharesData> for ResponseData {
    fn from(v: SharesData) -> Self {
        ResponseData::Shares(v)
    }
}

// === Internet Radio Stations ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternetRadioStationsData {
    pub internet_radio_stations: InternetRadioStationsList,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<String>,
}
impl From<&SubsonicInternetRadioStation> for InternetRadioStationItem {
    fn from(s: &SubsonicInternetRadioStation) -> Self {
        Self {
            id: s.id.clone(),
            name: s.name.clone(),
            stream_url: s.stream_url.clone(),
            homepage_url: s.homepage_url.clone(),
        }
    }
}
impl From<InternetRadioStationsData> for ResponseData {
    fn from(v: InternetRadioStationsData) -> Self {
        ResponseData::InternetRadioStations(v)
    }
}

// === Lyrics ===
#[derive(Debug, Clone, Serialize)]
pub struct LyricsData {
    pub lyrics: LyricsInner,
}
#[derive(Debug, Clone, Serialize)]
pub struct LyricsInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub value: String,
}
impl From<&SubsonicLyrics> for LyricsInner {
    fn from(l: &SubsonicLyrics) -> Self {
        Self {
            artist: l.artist.clone(),
            title: l.title.clone(),
            value: l.value.clone(),
        }
    }
}
impl From<LyricsData> for ResponseData {
    fn from(v: LyricsData) -> Self {
        ResponseData::Lyrics(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsListData {
    pub lyrics_list: LyricsListInner,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricsListInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structured_lyrics: Vec<StructuredLyricsItem>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredLyricsItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_artist: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_title: Option<String>,
    pub lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    pub synced: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub line: Vec<LyricLine>,
}
#[derive(Debug, Clone, Serialize)]
pub struct LyricLine {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    pub value: String,
}
impl From<&SubsonicStructuredLyrics> for StructuredLyricsItem {
    fn from(l: &SubsonicStructuredLyrics) -> Self {
        Self {
            display_artist: l.display_artist.clone(),
            display_title: l.display_title.clone(),
            lang: l.lang.clone(),
            offset: l.offset,
            synced: l.synced,
            line: l
                .lines
                .iter()
                .map(|line| LyricLine {
                    start: line.start,
                    value: line.value.clone(),
                })
                .collect(),
        }
    }
}
impl From<LyricsListData> for ResponseData {
    fn from(v: LyricsListData) -> Self {
        ResponseData::LyricsList(v)
    }
}

// === Scan Status ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusData {
    pub scan_status: ScanStatusInner,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusInner {
    pub scanning: bool,
    pub count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scan: Option<String>,
}
impl From<&SubsonicScanStatus> for ScanStatusInner {
    fn from(s: &SubsonicScanStatus) -> Self {
        Self {
            scanning: s.scanning,
            count: s.count,
            folder_count: Some(s.folder_count),
            last_scan: s.last_scan.map(|d| d.to_rfc3339()),
        }
    }
}
impl From<ScanStatusData> for ResponseData {
    fn from(v: ScanStatusData) -> Self {
        ResponseData::ScanStatus(v)
    }
}

// === Artist Info ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfoData {
    pub artist_info: ArtistInfoInner,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfoInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biography: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub similar_artist: Vec<ArtistItem>,
}
impl From<&SubsonicArtistInfo> for ArtistInfoInner {
    fn from(i: &SubsonicArtistInfo) -> Self {
        Self {
            biography: i.biography.clone(),
            music_brainz_id: i.music_brainz_id.clone(),
            last_fm_url: i.last_fm_url.clone(),
            small_image_url: i.small_image_url.clone(),
            medium_image_url: i.medium_image_url.clone(),
            large_image_url: i.large_image_url.clone(),
            similar_artist: i.similar_artists.iter().map(ArtistItem::from).collect(),
        }
    }
}
impl From<ArtistInfoData> for ResponseData {
    fn from(v: ArtistInfoData) -> Self {
        ResponseData::ArtistInfo(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistInfo2Data {
    pub artist_info2: ArtistInfoInner,
}
impl From<ArtistInfo2Data> for ResponseData {
    fn from(v: ArtistInfo2Data) -> Self {
        ResponseData::ArtistInfo2(v)
    }
}

// === Album Info ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfoData {
    pub album_info: AlbumInfoInner,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumInfoInner {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_brainz_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_fm_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image_url: Option<String>,
}
impl From<&SubsonicAlbumInfo> for AlbumInfoInner {
    fn from(i: &SubsonicAlbumInfo) -> Self {
        Self {
            notes: i.notes.clone(),
            music_brainz_id: i.music_brainz_id.clone(),
            last_fm_url: i.last_fm_url.clone(),
            small_image_url: i.small_image_url.clone(),
            medium_image_url: i.medium_image_url.clone(),
            large_image_url: i.large_image_url.clone(),
        }
    }
}
impl From<AlbumInfoData> for ResponseData {
    fn from(v: AlbumInfoData) -> Self {
        ResponseData::AlbumInfo(v)
    }
}

// === Similar Songs ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongsData {
    pub similar_songs: SimilarSongsInner,
}
#[derive(Debug, Clone, Serialize)]
pub struct SimilarSongsInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<SimilarSongsData> for ResponseData {
    fn from(v: SimilarSongsData) -> Self {
        ResponseData::SimilarSongs(v)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarSongs2Data {
    pub similar_songs2: SimilarSongsInner,
}
impl From<SimilarSongs2Data> for ResponseData {
    fn from(v: SimilarSongs2Data) -> Self {
        ResponseData::SimilarSongs2(v)
    }
}

// === Top Songs ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopSongsData {
    pub top_songs: TopSongsInner,
}
#[derive(Debug, Clone, Serialize)]
pub struct TopSongsInner {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub song: Vec<Child>,
}
impl From<TopSongsData> for ResponseData {
    fn from(v: TopSongsData) -> Self {
        ResponseData::TopSongs(v)
    }
}

// === OpenSubsonic Extensions ===
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSubsonicExtensionsData {
    pub open_subsonic_extensions: Vec<OpenSubsonicExtensionItem>,
}
#[derive(Debug, Clone, Serialize)]
pub struct OpenSubsonicExtensionItem {
    pub name: String,
    pub versions: Vec<i32>,
}
impl From<&SubsonicOpenSubsonicExtension> for OpenSubsonicExtensionItem {
    fn from(e: &SubsonicOpenSubsonicExtension) -> Self {
        Self {
            name: e.name.clone(),
            versions: e.versions.clone(),
        }
    }
}
impl From<OpenSubsonicExtensionsData> for ResponseData {
    fn from(v: OpenSubsonicExtensionsData) -> Self {
        ResponseData::OpenSubsonicExtensions(v)
    }
}

// === Helper functions ===

pub fn build_indexes(indexes: &[SubsonicArtistIndex], last_modified: i64) -> IndexesData {
    IndexesData {
        indexes: IndexesList {
            last_modified,
            ignored_articles: "The El La Los Las Le Les".to_string(),
            index: indexes
                .iter()
                .map(|idx| IndexItem {
                    name: idx.id.clone(),
                    artist: idx.artists.iter().map(ArtistItem::from).collect(),
                })
                .collect(),
        },
    }
}

pub fn build_artists(indexes: &[SubsonicArtistIndex], last_modified: i64) -> ArtistsData {
    ArtistsData {
        artists: ArtistsList {
            last_modified,
            ignored_articles: "The El La Los Las Le Les".to_string(),
            index: indexes
                .iter()
                .map(|idx| ArtistIndexItem {
                    name: idx.id.clone(),
                    artist: idx.artists.iter().map(ArtistID3Item::from).collect(),
                })
                .collect(),
        },
    }
}
