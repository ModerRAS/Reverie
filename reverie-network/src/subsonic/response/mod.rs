//! Subsonic API 响应类型和序列化
//!
//! 根据 Subsonic API 规范支持 JSON 和 XML 两种输出格式。

// Core response types
pub mod core;

// DTO modules
pub mod albums;
pub mod artists;
pub mod chat;
pub mod jukebox;
pub mod misc;
pub mod playlists;
pub mod podcasts;
pub mod songs;
pub mod users;
pub mod videos;

// Re-export all types for convenience
pub use core::{ErrorResponse, ResponseData, SubsonicResponse, SubsonicResponseInner};

pub use albums::{
    AlbumData, AlbumID3Item, AlbumInfo, AlbumInfoData, AlbumList2Data, AlbumList2Inner,
    AlbumListData, AlbumListInner, AlbumWithSongs, SimilarSongs2Data, SimilarSongs2Inner,
    SimilarSongsData, SimilarSongsInner, TopSongsData, TopSongsInner,
};

pub use artists::{
    build_artists, build_indexes, ArtistData, ArtistID3Item, ArtistIndexItem, ArtistInfo,
    ArtistInfo2, ArtistInfo2Data, ArtistInfoData, ArtistItem, ArtistWithAlbums, ArtistsData,
    ArtistsList, ImageItem, IndexItem, IndexesData, IndexesList, LinkItem, MusicFolderItem,
    MusicFoldersData, MusicFoldersList,
};

pub use misc::{
    BookmarkItem, BookmarksData, BookmarksList, GenreItem, GenresData, GenresInner, GenresList,
    InternetRadioStationItem, InternetRadioStationsData, InternetRadioStationsList, License,
    LicenseData, LyricsData, LyricsItem, LyricsListData, LyricsListInner,
    OpenSubsonicExtensionItem, OpenSubsonicExtensionsData, OpenSubsonicExtensionsList,
    PlayQueueData, PlayQueueInner, ScanStatusData, ScanStatusItem, StructuredLyricsItem,
};

pub use playlists::{
    PlaylistData, PlaylistItem, PlaylistWithEntries, PlaylistsData, PlaylistsInner, PlaylistsList,
};

pub use songs::{
    Child, DirectoryData, DirectoryInner, DirectoryItem, NowPlayingData, NowPlayingEntry,
    NowPlayingInner, RandomSongsData, RandomSongsInner, SearchResult2Data, SearchResult2Inner,
    SearchResult3Data, SearchResult3Inner, SongData, SongsByGenreData, SongsByGenreInner,
    Starred2Data, Starred2Inner, StarredData, StarredInner,
};

pub use users::{
    ShareItem, SharesData, SharesList, UserData, UserItem, UsersData, UsersInner, UsersList,
};

pub use videos::{
    CaptionItem, CaptionsData, CaptionsInner, VideoInfoData, VideoInfoItem, VideosData,
};

pub use jukebox::{JukeboxStatusData, JukeboxStatusItem};

pub use chat::{ChatMessageItem, ChatMessagesData};

pub use podcasts::{
    NewestPodcastsData, NewestPodcastsInner, PodcastChannelItem, PodcastEpisodeItem,
    PodcastEpisodesInner, PodcastsData, PodcastsInner,
};
