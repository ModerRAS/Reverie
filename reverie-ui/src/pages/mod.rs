//! Pages - Main views for the application
//!
//! Each page represents a full-screen view in the application.

pub mod albums;
pub mod artists;
pub mod songs;
pub mod playlists;
pub mod favorites;
pub mod search;
pub mod settings;
pub mod album_detail;
pub mod artist_detail;
pub mod playlist_detail;
pub mod login;
pub mod home;

pub use albums::AlbumsPage;
pub use artists::ArtistsPage;
pub use songs::SongsPage;
pub use playlists::PlaylistsPage;
pub use favorites::FavoritesPage;
pub use search::SearchPage;
pub use settings::SettingsPage;
pub use album_detail::AlbumDetailPage;
pub use artist_detail::ArtistDetailPage;
pub use playlist_detail::PlaylistDetailPage;
pub use login::LoginPage;
pub use home::HomePage;
