//! 存储抽象 traits
//!
//! 这些 traits 定义了存储操作的接口，
//! 允许在不更改核心应用程序逻辑的情况下切换不同的实现。

pub mod core;
pub mod user;
pub mod file;
pub mod storage;
pub mod subsonic;

pub use core::{AlbumStorage, ArtistStorage, TrackStorage};
pub use file::{FileMetadata, FileStorage};
pub use storage::Storage;
pub use subsonic::SubsonicStorage;
pub use user::{PlaylistStorage, UserStorage};
