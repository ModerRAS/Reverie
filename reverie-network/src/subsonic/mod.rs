//! Subsonic API implementation (stub)
//!
//! Reverie aims to be compatible with Subsonic API 1.16.1.
//!
//! The full Subsonic handler set is still a work-in-progress.
//! For now, we keep the routing surface and return minimal "ok" responses so the
//! server can run end-to-end (UI + /rest routing) while the real implementation
//! is iterated on.

use axum::{
    extract::{Query, State},
    routing::get,
    Router,
};
use reverie_storage::SubsonicStorage;
use std::{collections::HashMap, sync::Arc};

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

fn subsonic_ok() -> String {
    format!(
        r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<subsonic-response status=\"ok\" version=\"{}\"/>"#,
        SUBSONIC_API_VERSION
    )
}

#[derive(Clone)]
pub(crate) struct SubsonicState<S: SubsonicStorage + Clone> {
    pub(crate) storage: Arc<S>,
}

impl<S: SubsonicStorage + Clone> SubsonicState<S> {
    pub(crate) fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
}

/// Create the Subsonic router.
///
/// Note: The returned router is *missing* `SubsonicState<S>` and is intended to be
/// nested into an outer router that provides state via `Router::with_state`.
#[cfg(feature = "axum-server")]
pub fn create_router<S: SubsonicStorage + Clone + 'static>() -> Router<SubsonicState<S>> {
    Router::new()
        .route("/ping", get(stub_handler::<S>))
        .route("/getLicense", get(stub_handler::<S>))
        .route("/getMusicFolders", get(stub_handler::<S>))
        .route("/getIndexes", get(stub_handler::<S>))
        .route("/getMusicDirectory", get(stub_handler::<S>))
        .route("/getGenres", get(stub_handler::<S>))
        .route("/getArtists", get(stub_handler::<S>))
        .route("/getArtist", get(stub_handler::<S>))
        .route("/getAlbum", get(stub_handler::<S>))
        .route("/getSong", get(stub_handler::<S>))
        .route("/getArtistInfo", get(stub_handler::<S>))
        .route("/getArtistInfo2", get(stub_handler::<S>))
        .route("/getAlbumInfo", get(stub_handler::<S>))
        .route("/getAlbumInfo2", get(stub_handler::<S>))
        .route("/getSimilarSongs", get(stub_handler::<S>))
        .route("/getSimilarSongs2", get(stub_handler::<S>))
        .route("/getTopSongs", get(stub_handler::<S>))
        .route("/getAlbumList", get(stub_handler::<S>))
        .route("/getAlbumList2", get(stub_handler::<S>))
        .route("/getRandomSongs", get(stub_handler::<S>))
        .route("/getSongsByGenre", get(stub_handler::<S>))
        .route("/getNowPlaying", get(stub_handler::<S>))
        .route("/getStarred", get(stub_handler::<S>))
        .route("/getStarred2", get(stub_handler::<S>))
        .route("/search2", get(stub_handler::<S>))
        .route("/search3", get(stub_handler::<S>))
        .route("/getPlaylists", get(stub_handler::<S>))
        .route("/getPlaylist", get(stub_handler::<S>))
        .route("/createPlaylist", get(stub_handler::<S>))
        .route("/updatePlaylist", get(stub_handler::<S>))
        .route("/deletePlaylist", get(stub_handler::<S>))
        .route("/stream", get(stub_handler::<S>))
        .route("/download", get(stub_handler::<S>))
        .route("/getCoverArt", get(stub_handler::<S>))
        .route("/getLyrics", get(stub_handler::<S>))
        .route("/getLyricsBySongId", get(stub_handler::<S>))
        .route("/getAvatar", get(stub_handler::<S>))
        .route("/star", get(stub_handler::<S>))
        .route("/unstar", get(stub_handler::<S>))
        .route("/setRating", get(stub_handler::<S>))
        .route("/scrobble", get(stub_handler::<S>))
        .route("/getBookmarks", get(stub_handler::<S>))
        .route("/createBookmark", get(stub_handler::<S>))
        .route("/deleteBookmark", get(stub_handler::<S>))
        .route("/getPlayQueue", get(stub_handler::<S>))
        .route("/savePlayQueue", get(stub_handler::<S>))
        .route("/getShares", get(stub_handler::<S>))
        .route("/createShare", get(stub_handler::<S>))
        .route("/updateShare", get(stub_handler::<S>))
        .route("/deleteShare", get(stub_handler::<S>))
        .route("/getInternetRadioStations", get(stub_handler::<S>))
        .route("/createInternetRadioStation", get(stub_handler::<S>))
        .route("/updateInternetRadioStation", get(stub_handler::<S>))
        .route("/deleteInternetRadioStation", get(stub_handler::<S>))
        .route("/getUser", get(stub_handler::<S>))
        .route("/getUsers", get(stub_handler::<S>))
        .route("/getScanStatus", get(stub_handler::<S>))
        .route("/startScan", get(stub_handler::<S>))
        .route("/getOpenSubsonicExtensions", get(stub_handler::<S>))
}

async fn stub_handler<S: SubsonicStorage + Clone>(
    State(_state): State<SubsonicState<S>>,
    Query(_params): Query<HashMap<String, String>>,
) -> String {
    subsonic_ok()
}
