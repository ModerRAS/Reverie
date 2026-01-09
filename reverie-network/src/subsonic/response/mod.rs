//! Subsonic API 响应类型和序列化
//!
//! 根据 Subsonic API 规范支持 JSON 和 XML 两种输出格式。

// Core response types
pub mod core;

// DTO modules
pub mod albums;
pub mod artists;
pub mod misc;
pub mod playlists;
pub mod songs;
pub mod users;

// Re-export all types for convenience
pub use core::{
    ErrorResponse, ResponseData, SubsonicResponse, SubsonicResponseInner,
};

pub use albums::{
    AlbumData, AlbumID3Item, AlbumInfo, AlbumInfoData, AlbumList2Data, AlbumListData,
    AlbumList2Inner, AlbumListInner, AlbumWithSongs, SimilarSongs2Data, SimilarSongsData,
    SimilarSongs2Inner, SimilarSongsInner, TopSongsData, TopSongsInner,
};

pub use artists::{
    ArtistData, ArtistID3Item, ArtistInfo, ArtistInfo2, ArtistInfo2Data, ArtistInfoData,
    ArtistIndexItem, ArtistItem, ArtistWithAlbums, ArtistsData, ImageItem, LinkItem,
    MusicFolderItem, MusicFoldersData, MusicFoldersList,
};

pub use misc::{
    BookmarksData, BookmarksList, BookmarkItem, GenresData, GenresList, GenreItem,
    InternetRadioStationItem, InternetRadioStationsData, InternetRadioStationsList,
    License, LicenseData, LyricsData, LyricsItem, LyricsListData, LyricsListInner,
    OpenSubsonicExtensionItem, OpenSubsonicExtensionsData, OpenSubsonicExtensionsList,
    PlayQueueData, PlayQueueInner, ScanStatusData, ScanStatusItem,
};

pub use playlists::{
    PlaylistData, PlaylistItem, PlaylistWithEntries, PlaylistsData, PlaylistsList,
};

pub use songs::{
    Child, DirectoryData, DirectoryItem, NowPlayingData, NowPlayingEntry, NowPlayingInner,
    RandomSongsData, RandomSongsInner, SearchResult2Data, SearchResult2Inner,
    SearchResult3Data, SearchResult3Inner, SongData, SongsByGenreData, SongsByGenreInner,
    Starred2Data, Starred2Inner, StarredData, StarredInner,
};

pub use users::{
    ShareItem, SharesData, SharesList, UserData, UserItem, UsersData, UsersList,
};
