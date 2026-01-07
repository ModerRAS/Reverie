//! Pages - Main views for the application
//!
//! Each page represents a full-screen view in the application.

pub mod album_detail;
pub mod albums;
pub mod artist_detail;
pub mod artists;
pub mod favorites;
pub mod home;
pub mod login;
pub mod playlist_detail;
pub mod playlists;
pub mod search;
pub mod settings;
pub mod songs;

pub use album_detail::AlbumDetailPage;
pub use albums::AlbumsPage;
pub use artist_detail::ArtistDetailPage;
pub use artists::ArtistsPage;
pub use favorites::FavoritesPage;
pub use home::HomePage;
pub use login::LoginPage;
pub use playlist_detail::PlaylistDetailPage;
pub use playlists::PlaylistsPage;
pub use search::SearchPage;
pub use settings::SettingsPage;
pub use songs::SongsPage;
