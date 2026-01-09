//! Subsonic API implementation
//! Compatible with Subsonic API 1.16.1

pub mod response;
pub mod auth;
pub mod params;

use crate::subsonic::params::parse_query_params;
use crate::subsonic::response::{subsonic_ok, subsonic_error};
use axum::{
    body::Body,
    extract::Request,
    http::Uri,
    response::IntoResponse,
    routing::get,
    Router,
};
use reverie_storage::SubsonicStorage;
use std::collections::HashMap;
use std::sync::Arc;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

fn get_query_params(uri: &Uri) -> HashMap<String, String> {
    if let Some(query) = uri.query() {
        parse_query_params(query)
    } else {
        HashMap::new()
    }
}

#[cfg(feature = "axum-server")]
pub fn create_router<S: SubsonicStorage + Clone + 'static>(storage: Arc<S>) -> Router {
    Router::new()
        .route("/ping", get(ping_handler))
        .route("/getLicense", get(license_handler))
        .route("/getMusicFolders", get(music_folders_handler))
        .route("/getIndexes", get(indexes_handler))
        .route("/getMusicDirectory", get(music_directory_handler))
        .route("/getGenres", get(genres_handler))
        .route("/getArtists", get(artists_handler))
        .route("/getArtist", get(artist_handler))
        .route("/getAlbum", get(album_handler))
        .route("/getSong", get(song_handler))
        .route("/getArtistInfo", get(artist_info_handler))
        .route("/getArtistInfo2", get(artist_info2_handler))
        .route("/getAlbumInfo", get(album_info_handler))
        .route("/getAlbumInfo2", get(album_info2_handler))
        .route("/getSimilarSongs", get(similar_songs_handler))
        .route("/getSimilarSongs2", get(similar_songs2_handler))
        .route("/getTopSongs", get(top_songs_handler))
        .route("/getAlbumList", get(album_list_handler))
        .route("/getAlbumList2", get(album_list2_handler))
        .route("/getRandomSongs", get(random_songs_handler))
        .route("/getSongsByGenre", get(songs_by_genre_handler))
        .route("/getNowPlaying", get(now_playing_handler))
        .route("/getStarred", get(starred_handler))
        .route("/getStarred2", get(starred2_handler))
        .route("/search2", get(search2_handler))
        .route("/search3", get(search3_handler))
        .route("/getPlaylists", get(playlists_handler))
        .route("/getPlaylist", get(playlist_handler))
        .route("/createPlaylist", get(create_playlist_handler))
        .route("/updatePlaylist", get(update_playlist_handler))
        .route("/deletePlaylist", get(delete_playlist_handler))
        .route("/stream", get(stream_handler))
        .route("/download", get(download_handler))
        .route("/getCoverArt", get(cover_art_handler))
        .route("/getLyrics", get(lyrics_handler))
        .route("/getLyricsBySongId", get(lyrics_by_song_id_handler))
        .route("/getAvatar", get(avatar_handler))
        .route("/star", get(star_handler))
        .route("/unstar", get(unstar_handler))
        .route("/setRating", get(rating_handler))
        .route("/scrobble", get(scrobble_handler))
        .route("/getBookmarks", get(bookmarks_handler))
        .route("/createBookmark", get(create_bookmark_handler))
        .route("/deleteBookmark", get(delete_bookmark_handler))
        .route("/getPlayQueue", get(play_queue_handler))
        .route("/savePlayQueue", get(save_play_queue_handler))
        .route("/getShares", get(shares_handler))
        .route("/createShare", get(create_share_handler))
        .route("/updateShare", get(update_share_handler))
        .route("/deleteShare", get(delete_share_handler))
        .route("/getInternetRadioStations", get(radio_stations_handler))
        .route("/createInternetRadioStation", get(create_radio_station_handler))
        .route("/updateInternetRadioStation", get(update_radio_station_handler))
        .route("/deleteInternetRadioStation", get(delete_radio_station_handler))
        .route("/getUser", get(user_handler))
        .route("/getUsers", get(users_handler))
        .route("/getScanStatus", get(scan_status_handler))
        .route("/startScan", get(start_scan_handler))
        .route("/getOpenSubsonicExtensions", get(open_subsonic_extensions_handler))
        .with_state(storage)
}

struct SubsonicState<S: SubsonicStorage + Clone> {
    storage: Arc<S>,
}

impl<S: SubsonicStorage + Clone> SubsonicState<S> {
    fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    async fn get_params(&self, uri: &Uri) -> HashMap<String, String> {
        get_query_params(uri)
    }
}

macro_rules! handler {
    ($name:ident) => {
        async fn $name<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
            let params = state.get_params(&uri).await;
            $name_inner(&state.storage, &params).await
        }
    };
}

async fn ping_handler<S: SubsonicStorage + Clone>(_: State<SubsonicState<S>>, _: Uri) -> String {
    subsonic_ok()
}

async fn license_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    let valid = state.storage.get_license().await.unwrap_or(true);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <license valid="{}"/>
</subsonic-response>"#,
        SUBSONIC_API_VERSION, valid
    )
}

async fn music_folders_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_music_folders().await {
        Ok(folders) => {
            let folders_xml: String = folders
                .iter()
                .map(|f| format!(r#"<musicFolder id="{}" name="{}"/>"#, f.id, f.name))
                .collect();

            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <musicFolders>{}</musicFolders>
</subsonic-response>"#,
                SUBSONIC_API_VERSION, folders_xml
            )
        }
        Err(_) => subsonic_error(0, "Failed to get music folders"),
    }
}

async fn indexes_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let music_folder_id = params.get("musicFolderId").and_then(|v| v.parse().ok());
    let if_modified_since = params.get("ifModifiedSince").and_then(|v| v.parse().ok());

    match state.storage.get_indexes(music_folder_id, if_modified_since).await {
        Ok(indexes) => {
            let mut indexes_xml = String::new();
            for idx in &indexes {
                let mut artists_xml = String::new();
                for artist in &idx.artists {
                    artists_xml.push_str(&format!(
                        r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#,
                        artist.id, artist.name, artist.album_count, artist.cover_art.unwrap_or_default()
                    ));
                }
                indexes_xml.push_str(&format!(r#"<index name="{}">{}</index>"#, idx.name, artists_xml));
            }

            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <indexes lastModified="0" ignoredArticles="The La Le">
        {}
    </indexes>
</subsonic-response>"#,
                SUBSONIC_API_VERSION, indexes_xml
            )
        }
        Err(_) => subsonic_error(0, "Failed to get indexes"),
    }
}

async fn music_directory_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_music_directory(&id).await {
        Ok(Some(dir)) => {
            let mut children_xml = String::new();
            for child in &dir.children {
                let mut attrs = vec![
                    ("id", child.id.as_str()),
                    ("parent", child.parent.as_str()),
                    ("title", child.title.as_str()),
                    ("isDir", if child.is_dir { "true" } else { "false" }),
                ];
                if !child.album.is_empty() { attrs.push(("album", child.album.as_str())); }
                if !child.artist.is_empty() { attrs.push(("artist", child.artist.as_str())); }
                if child.year > 0 { attrs.push(("year", child.year.to_string().as_str())); }
                if !child.genre.is_empty() { attrs.push(("genre", child.genre.as_str())); }
                if child.track > 0 { attrs.push(("track", child.track.to_string().as_str())); }
                if child.duration > 0 { attrs.push(("duration", child.duration.to_string().as_str())); }
                if child.size > 0 { attrs.push(("size", child.size.to_string().as_str())); }
                if !child.suffix.is_empty() { attrs.push(("suffix", child.suffix.as_str())); }
                if !child.content_type.is_empty() { attrs.push(("contentType", child.content_type.as_str())); }
                if !child.cover_art.is_empty() { attrs.push(("coverArt", child.cover_art.as_str())); }

                let attr_str: String = attrs.iter().map(|(k, v)| format!("{}='{}'", k, v)).collect::<Vec<_>>().join(" ");
                children_xml.push_str(&format!("<child {}/>", attr_str));
            }

            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <directory id="{}" name="{}" childCount="{}">
        {}
    </directory>
</subsonic-response>"#,
                SUBSONIC_API_VERSION, dir.id, dir.name, dir.children.len(), children_xml
            )
        }
        Ok(None) => subsonic_error(70, "Directory not found"),
        Err(_) => subsonic_error(0, "Failed to get directory"),
    }
}

async fn genres_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_genres().await {
        Ok(genres) => {
            let genres_xml: String = genres
                .iter()
                .map(|g| format!(r#"<genre name="{}" songCount="{}" albumCount="{}"/>"#, g.value, g.song_count, g.album_count))
                .collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <genres>{}</genres>
</subsonic-response>"#, SUBSONIC_API_VERSION, genres_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get genres"),
    }
}

async fn artists_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let music_folder_id = params.get("musicFolderId").and_then(|v| v.parse().ok());

    match state.storage.get_artists(music_folder_id).await {
        Ok(indexes) => {
            let mut indexes_xml = String::new();
            for idx in &indexes {
                let mut artists_xml = String::new();
                for artist in &idx.artists {
                    let cover = artist.cover_art.as_deref().unwrap_or_default();
                    artists_xml.push_str(&format!(r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#, artist.id, artist.name, artist.album_count, cover));
                }
                indexes_xml.push_str(&format!(r#"<index name="{}">{}</index>"#, idx.name, artists_xml));
            }
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <artists lastModified="0" ignoredArticles="The La Le">{}</artists>
</subsonic-response>"#, SUBSONIC_API_VERSION, indexes_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get artists"),
    }
}

async fn artist_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_artist(&id).await {
        Ok(Some(artist)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, artist.id, artist.name, artist.album_count, artist.cover_art.as_deref().unwrap_or(""))
        }
        Ok(None) => subsonic_error(70, "Artist not found"),
        Err(_) => subsonic_error(0, "Failed to get artist"),
    }
}

async fn album_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_album(&id).await {
        Ok(Some(album)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <album id="{}" name="{}" artist="{}" artistId="{}" year="{}" genre="{}" songCount="{}" duration="{}" coverArt="{}"/>
</subsonic-response>"#,
                SUBSONIC_API_VERSION, album.id, album.name,
                album.artist.as_deref().unwrap_or(""),
                album.artist_id.as_deref().unwrap_or(""),
                album.year.unwrap_or(0),
                album.genre.as_deref().unwrap_or(""),
                album.song_count.unwrap_or(0),
                album.duration.unwrap_or(0),
                album.cover_art.as_deref().unwrap_or(""))
        }
        Ok(None) => subsonic_error(70, "Album not found"),
        Err(_) => subsonic_error(0, "Failed to get album"),
    }
}

async fn song_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_song(&id).await {
        Ok(Some(song)) => {
            let attrs = vec![
                ("id", song.id.as_str()),
                ("parent", song.parent.as_deref().unwrap_or("")),
                ("title", song.title.as_str()),
                ("album", song.album.as_deref().unwrap_or("")),
                ("artist", song.artist.as_deref().unwrap_or("")),
                ("track", song.track.map(|t| t.to_string()).as_deref().unwrap_or("")),
                ("year", song.year.map(|y| y.to_string()).as_deref().unwrap_or("")),
                ("genre", song.genre.as_deref().unwrap_or("")),
                ("coverArt", song.cover_art.as_deref().unwrap_or("")),
                ("duration", song.duration.map(|d| d.to_string()).as_deref().unwrap_or("")),
                ("size", song.size.map(|s| s.to_string()).as_deref().unwrap_or("")),
                ("contentType", song.content_type.as_deref().unwrap_or("")),
                ("suffix", song.suffix.as_deref().unwrap_or("")),
            ];
            let attr_str: String = attrs.iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| format!("{}='{}'", k, v)).collect::<Vec<_>>().join(" ");
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <song {} />
</subsonic-response>"#, SUBSONIC_API_VERSION, attr_str)
        }
        Ok(None) => subsonic_error(70, "Song not found"),
        Err(_) => subsonic_error(0, "Failed to get song"),
    }
}

async fn artist_info_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());

    match state.storage.get_artist_info(&id, count, None).await {
        Ok(info) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <artistInfo id="{}" name="{}" coverArt="{}" albumCount="{}">
        <biography>{}</biography>
        <musicBrainzId>{}</musicBrainzId>
        <lastFmUrl>{}</lastFmUrl>
        <smallImageUrl>{}</smallImageUrl>
        <mediumImageUrl>{}</mediumImageUrl>
        <largeImageUrl>{}</largeImageUrl>
    </artistInfo>
</subsonic-response>"#, SUBSONIC_API_VERSION, id, info.artist_name, info.cover_art, info.album_count, info.biography, info.music_brainz_id, info.last_fm_url, info.small_image_url, info.medium_image_url, info.large_image_url)
        }
        Err(_) => subsonic_error(0, "Failed to get artist info"),
    }
}

async fn artist_info2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());

    match state.storage.get_artist_info2(&id, count, None).await {
        Ok(info) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <artistInfo2>
        <biography>{}</biography>
        <musicBrainzId>{}</musicBrainzId>
        <lastFmUrl>{}</lastFmUrl>
        <smallImageUrl>{}</smallImageUrl>
        <mediumImageUrl>{}</mediumImageUrl>
        <largeImageUrl>{}</largeImageUrl>
    </artistInfo2>
</subsonic-response>"#, SUBSONIC_API_VERSION, info.biography, info.music_brainz_id, info.last_fm_url, info.small_image_url, info.medium_image_url, info.large_image_url)
        }
        Err(_) => subsonic_error(0, "Failed to get artist info"),
    }
}

async fn album_info_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_album_info(&id).await {
        Ok(info) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <albumInfo>
        <notes>{}</notes>
        <musicBrainzId>{}</musicBrainzId>
        <lastFmUrl>{}</lastFmUrl>
        <smallImageUrl>{}</smallImageUrl>
        <mediumImageUrl>{}</mediumImageUrl>
        <largeImageUrl>{}</largeImageUrl>
    </albumInfo>
</subsonic-response>"#, SUBSONIC_API_VERSION, info.notes, info.music_brainz_id, info.last_fm_url, info.small_image_url, info.medium_image_url, info.large_image_url)
        }
        Err(_) => subsonic_error(0, "Failed to get album info"),
    }
}

async fn album_info2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_album_info2(&id).await {
        Ok(info) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <albumInfo>
        <notes>{}</notes>
        <musicBrainzId>{}</musicBrainzId>
        <lastFmUrl>{}</lastFmUrl>
        <smallImageUrl>{}</smallImageUrl>
        <mediumImageUrl>{}</mediumImageUrl>
        <largeImageUrl>{}</largeImageUrl>
    </albumInfo>
</subsonic-response>"#, SUBSONIC_API_VERSION, info.notes, info.music_brainz_id, info.last_fm_url, info.small_image_url, info.medium_image_url, info.large_image_url)
        }
        Err(_) => subsonic_error(0, "Failed to get album info"),
    }
}

async fn similar_songs_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());

    match state.storage.get_similar_songs(&id, count).await {
        Ok(songs) => {
            let songs_xml: String = songs.iter().map(|s| format!(r#"<song id="{}" title="{}"/>"#, s.id, s.title)).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <similarSongs>{}</similarSongs>
</subsonic-response>"#, SUBSONIC_API_VERSION, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get similar songs"),
    }
}

async fn similar_songs2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());

    match state.storage.get_similar_songs2(&id, count).await {
        Ok(songs) => {
            let songs_xml: String = songs.iter().map(|s| format!(r#"<song id="{}" title="{}"/>"#, s.id, s.title)).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <similarSongs2>{}</similarSongs2>
</subsonic-response>"#, SUBSONIC_API_VERSION, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get similar songs"),
    }
}

async fn top_songs_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let artist = params.get("artist").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());

    match state.storage.get_top_songs(&artist, count).await {
        Ok(top_songs) => {
            let songs_xml: String = top_songs.song.iter().map(|s| format!(r#"<song id="{}" title="{}"/>"#, s.id, s.title)).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <topSongs>{}</topSongs>
</subsonic-response>"#, SUBSONIC_API_VERSION, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get top songs"),
    }
}

async fn album_list_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let list_type = params.get("type").cloned().unwrap_or_else(|| "newest".to_string());
    let size = params.get("size").and_then(|v| v.parse().ok());
    let offset = params.get("offset").and_then(|v| v.parse().ok());
    let from_year = params.get("fromYear").and_then(|v| v.parse().ok());
    let to_year = params.get("toYear").and_then(|v| v.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());

    match state.storage.get_album_list(&list_type, size, offset, from_year, to_year, genre, None).await {
        Ok(albums) => {
            let albums_xml: String = albums.iter().map(|a| {
                format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" year="{}" genre="{}" coverArt="{}" songCount="{}" duration="{}"/>"#,
                    a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""),
                    a.year.unwrap_or(0), a.genre.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""),
                    a.song_count.unwrap_or(0), a.duration.unwrap_or(0))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <albumList>{}</albumList>
</subsonic-response>"#, SUBSONIC_API_VERSION, albums_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get album list"),
    }
}

async fn album_list2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let list_type = params.get("type").cloned().unwrap_or_else(|| "newest".to_string());
    let size = params.get("size").and_then(|v| v.parse().ok());
    let offset = params.get("offset").and_then(|v| v.parse().ok());
    let from_year = params.get("fromYear").and_then(|v| v.parse().ok());
    let to_year = params.get("toYear").and_then(|v| v.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());

    match state.storage.get_album_list2(&list_type, size, offset, from_year, to_year, genre, None).await {
        Ok(albums) => {
            let albums_xml: String = albums.iter().map(|a| {
                format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" year="{}" genre="{}" coverArt="{}" songCount="{}" duration="{}"/>"#,
                    a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""),
                    a.year.unwrap_or(0), a.genre.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""),
                    a.song_count.unwrap_or(0), a.duration.unwrap_or(0))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <albumList2>{}</albumList2>
</subsonic-response>"#, SUBSONIC_API_VERSION, albums_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get album list"),
    }
}

async fn random_songs_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let size = params.get("size").and_then(|v| v.parse().ok());
    let genre = params.get("genre").map(|s| s.as_str());
    let from_year = params.get("fromYear").and_then(|v| v.parse().ok());
    let to_year = params.get("toYear").and_then(|v| v.parse().ok());

    match state.storage.get_random_songs(size, genre, from_year, to_year, None).await {
        Ok(songs) => {
            let songs_xml: String = songs.iter().map(|s| {
                format!(r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}"/>"#,
                    s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""),
                    s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""),
                    s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                    s.size.map(|s| s.to_string()).as_deref().unwrap_or(""),
                    s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <randomSongs>{}</randomSongs>
</subsonic-response>"#, SUBSONIC_API_VERSION, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get random songs"),
    }
}

async fn songs_by_genre_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let genre = params.get("genre").cloned().unwrap_or_default();
    let count = params.get("count").and_then(|v| v.parse().ok());
    let offset = params.get("offset").and_then(|v| v.parse().ok());

    match state.storage.get_songs_by_genre(&genre, count, offset, None).await {
        Ok(songs) => {
            let songs_xml: String = songs.iter().map(|s| {
                format!(r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}" genre="{}"/>"#,
                    s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""),
                    s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""),
                    s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                    s.size.map(|s| s.to_string()).as_deref().unwrap_or(""),
                    s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""), genre)
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <songsByGenre>{}</songsByGenre>
</subsonic-response>"#, SUBSONIC_API_VERSION, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get songs by genre"),
    }
}

async fn now_playing_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_now_playing().await {
        Ok(entries) => {
            let entries_xml: String = entries.iter().map(|e| {
                format!(r#"<entry id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}" username="{}" playerName="{}" playerType="{}" milliseconds="{}"/>"#,
                    e.song.id, e.song.parent.as_deref().unwrap_or(""), e.song.title, e.song.album.as_deref().unwrap_or(""),
                    e.song.artist.as_deref().unwrap_or(""), e.song.track.map(|t| t.to_string()).as_deref().unwrap_or(""),
                    e.song.duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                    e.song.size.map(|s| s.to_string()).as_deref().unwrap_or(""),
                    e.song.content_type.as_deref().unwrap_or(""), e.song.suffix.as_deref().unwrap_or(""),
                    e.username, e.player_name.as_deref().unwrap_or(""), e.player_type.as_deref().unwrap_or(""),
                    e.milliseconds.map(|m| m.to_string()).as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <nowPlaying>{}</nowPlaying>
</subsonic-response>"#, SUBSONIC_API_VERSION, entries_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get now playing"),
    }
}

async fn starred_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_starred(None).await {
        Ok(starred) => {
            let artists_xml: String = starred.artist.iter().map(|a| format!(r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#, a.id, a.name, a.album_count, a.cover_art.as_deref().unwrap_or(""))).collect();
            let albums_xml: String = starred.album.iter().map(|a| format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" coverArt="{}"/>"#, a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""))).collect();
            let songs_xml: String = starred.song.iter().map(|s| format!(r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}"/>"#, s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""), s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""), s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""))).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <starred>{}{}{}</starred>
</subsonic-response>"#, SUBSONIC_API_VERSION, artists_xml, albums_xml, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get starred"),
    }
}

async fn starred2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_starred2(None).await {
        Ok(starred) => {
            let artists_xml: String = starred.artist.iter().map(|a| format!(r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#, a.id, a.name, a.album_count, a.cover_art.as_deref().unwrap_or(""))).collect();
            let albums_xml: String = starred.album.iter().map(|a| format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" coverArt="{}"/>"#, a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""))).collect();
            let songs_xml: String = starred.song.iter().map(|s| format!(r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}"/>"#, s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""), s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""), s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""))).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <starred2>{}{}{}</starred2>
</subsonic-response>"#, SUBSONIC_API_VERSION, artists_xml, albums_xml, songs_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get starred"),
    }
}

async fn search2_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let query = params.get("query").cloned().unwrap_or_default();
    let artist_count = params.get("artistCount").and_then(|v| v.parse().ok());
    let artist_offset = params.get("artistOffset").and_then(|v| v.parse().ok());
    let album_count = params.get("albumCount").and_then(|v| v.parse().ok());
    let album_offset = params.get("albumOffset").and_then(|v| v.parse().ok());
    let song_count = params.get("songCount").and_then(|v| v.parse().ok());
    let song_offset = params.get("songOffset").and_then(|v| v.parse().ok());

    match state.storage.search2(&query, artist_count, artist_offset, album_count, album_offset, song_count, song_offset).await {
        Ok(result) => {
            let total_hits = result.artist.len() + result.album.len() + result.song.len();
            let artists_xml: String = result.artist.iter().map(|a| format!(r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#, a.id, a.name, a.album_count, a.cover_art.as_deref().unwrap_or(""))).collect();
            let albums_xml: String = result.album.iter().map(|a| format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" coverArt="{}" songCount="{}" duration="{}"/>"#, a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""), a.song_count.unwrap_or(0), a.duration.unwrap_or(0))).collect();
            let songs_xml: String = result.song.iter().map(|s| format!(r#"<child id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}"/>"#, s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""), s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""), s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""), s.size.map(|s| s.to_string()).as_deref().unwrap_or(""), s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""))).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <searchResult2 totalHits="{}">{}{}{}</searchResult2>
</subsonic-response>"#, SUBSONIC_API_VERSION, total_hits, artists_xml, albums_xml, songs_xml)
        }
        Err(_) => subsonic_error(0, "Search failed"),
    }
}

async fn search3_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let query = params.get("query").cloned().unwrap_or_default();
    let artist_count = params.get("artistCount").and_then(|v| v.parse().ok());
    let artist_offset = params.get("artistOffset").and_then(|v| v.parse().ok());
    let album_count = params.get("albumCount").and_then(|v| v.parse().ok());
    let album_offset = params.get("albumOffset").and_then(|v| v.parse().ok());
    let song_count = params.get("songCount").and_then(|v| v.parse().ok());
    let song_offset = params.get("songOffset").and_then(|v| v.parse().ok());

    match state.storage.search3(&query, artist_count, artist_offset, album_count, album_offset, song_count, song_offset).await {
        Ok(result) => {
            let total_hits = result.artist.len() + result.album.len() + result.song.len();
            let artists_xml: String = result.artist.iter().map(|a| format!(r#"<artist id="{}" name="{}" albumCount="{}" coverArt="{}"/>"#, a.id, a.name, a.album_count, a.cover_art.as_deref().unwrap_or(""))).collect();
            let albums_xml: String = result.album.iter().map(|a| format!(r#"<album id="{}" name="{}" artist="{}" artistId="{}" coverArt="{}" songCount="{}" duration="{}"/>"#, a.id, a.name, a.artist.as_deref().unwrap_or(""), a.artist_id.as_deref().unwrap_or(""), a.cover_art.as_deref().unwrap_or(""), a.song_count.unwrap_or(0), a.duration.unwrap_or(0))).collect();
            let songs_xml: String = result.song.iter().map(|s| format!(r#"<song id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}"/>"#, s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""), s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""), s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""), s.size.map(|s| s.to_string()).as_deref().unwrap_or(""), s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""))).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <searchResult3 totalHits="{}">{}{}{}</searchResult3>
</subsonic-response>"#, SUBSONIC_API_VERSION, total_hits, artists_xml, albums_xml, songs_xml)
        }
        Err(_) => subsonic_error(0, "Search failed"),
    }
}

async fn playlists_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let username = params.get("username").map(|s| s.as_str());

    match state.storage.get_playlists(username).await {
        Ok(playlists) => {
            let playlists_xml: String = playlists.iter().map(|p| {
                format!(r#"<playlist id="{}" name="{}" owner="{}" songCount="{}" duration="{}" public="{}" created="{}" changed="{}" coverArt="{}"/>"#,
                    p.id, p.name, p.owner.as_deref().unwrap_or(""), p.song_count, p.duration,
                    if p.public.unwrap_or(false) { "true" } else { "false" },
                    p.created.as_deref().unwrap_or(""), p.changed.as_deref().unwrap_or(""), p.cover_art.as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <playlists>{}</playlists>
</subsonic-response>"#, SUBSONIC_API_VERSION, playlists_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get playlists"),
    }
}

async fn playlist_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_playlist(&id).await {
        Ok(Some(playlist)) => {
            let entries_xml: String = playlist.entry.iter().map(|s| {
                format!(r#"<entry id="{}" parent="{}" title="{}" album="{}" artist="{}" track="{}" duration="{}" size="{}" contentType="{}" suffix="{}"/>"#,
                    s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""),
                    s.artist.as_deref().unwrap_or(""), s.track.map(|t| t.to_string()).as_deref().unwrap_or(""),
                    s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                    s.size.map(|s| s.to_string()).as_deref().unwrap_or(""),
                    s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <playlist id="{}" name="{}" owner="{}" songCount="{}" duration="{}" public="{}" created="{}" changed="{}">
        {}
    </playlist>
</subsonic-response>"#, SUBSONIC_API_VERSION, playlist.id, playlist.name, playlist.owner.as_deref().unwrap_or(""), playlist.song_count, playlist.duration,
                if playlist.public.unwrap_or(false) { "true" } else { "false" }, playlist.created.as_deref().unwrap_or(""), playlist.changed.as_deref().unwrap_or(""), entries_xml)
        }
        Ok(None) => subsonic_error(70, "Playlist not found"),
        Err(_) => subsonic_error(0, "Failed to get playlist"),
    }
}

async fn create_playlist_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let name = params.get("name").map(|s| s.as_str());
    let playlist_id = params.get("playlistId").map(|s| s.as_str());
    let song_ids_str = params.get("songIds").map(|s| s.split(',').collect::<Vec<_>>());
    let song_ids: Vec<&str> = song_ids_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

    match state.storage.create_playlist(name, playlist_id, &song_ids).await {
        Ok(playlist) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <playlist id="{}" name="{}" owner="{}" songCount="{}" duration="{}" public="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, playlist.id, playlist.name, playlist.owner.as_deref().unwrap_or(""), playlist.song_count, playlist.duration, if playlist.public.unwrap_or(false) { "true" } else { "false" })
        }
        Err(_) => subsonic_error(0, "Failed to create playlist"),
    }
}

async fn update_playlist_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let playlist_id = params.get("playlistId").cloned().unwrap_or_default();
    let name = params.get("name").map(|s| s.as_str());
    let comment = params.get("comment").map(|s| s.as_str());
    let public = params.get("public").map(|s| s == "true" || s == "1");
    let song_ids_to_add_str = params.get("songIdsToAdd").map(|s| s.split(',').collect::<Vec<_>>());
    let song_ids_to_add: Vec<&str> = song_ids_to_add_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let song_indexes_str = params.get("songIndexesToRemove").map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect::<Vec<_>>());
    let song_indexes: Vec<i32> = song_indexes_str.unwrap_or_default();

    match state.storage.update_playlist(&playlist_id, name, comment, public, &song_ids_to_add, &song_indexes).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to update playlist"),
    }
}

async fn delete_playlist_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.delete_playlist(&id).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to delete playlist"),
    }
}

async fn stream_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let format = params.get("format").cloned().unwrap_or_else(|| "mp3".to_string());
    let bitrate = params.get("bitrate").and_then(|v| v.parse().ok()).unwrap_or(320);

    match state.storage.get_stream_path(&id).await {
        Ok(Some(_path)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <stream id="{}" format="{}" bitRate="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, id, format, bitrate)
        }
        Ok(None) => subsonic_error(70, "Song not found"),
        Err(_) => subsonic_error(0, "Failed to get stream"),
    }
}

async fn download_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_stream_path(&id).await {
        Ok(Some(_path)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <download id="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, id)
        }
        Ok(None) => subsonic_error(70, "Song not found"),
        Err(_) => subsonic_error(0, "Failed to get download"),
    }
}

async fn cover_art_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let size = params.get("size").and_then(|v| v.parse().ok()).unwrap_or(0);

    match state.storage.get_cover_art_path(&id).await {
        Ok(Some(_path)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <coverArt id="{}" size="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, id, size)
        }
        Ok(None) => subsonic_error(70, "Cover art not found"),
        Err(_) => subsonic_error(0, "Failed to get cover art"),
    }
}

async fn lyrics_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let artist = params.get("artist").map(|s| s.as_str());
    let title = params.get("title").map(|s| s.as_str());

    match state.storage.get_lyrics(artist, title).await {
        Ok(Some(lyrics)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <lyrics artist="{}" title="{}">{}</lyrics>
</subsonic-response>"#, SUBSONIC_API_VERSION, lyrics.artist.as_deref().unwrap_or(""), lyrics.title.as_deref().unwrap_or(""), lyrics.value)
        }
        Ok(None) => subsonic_error(40, "Lyrics not found"),
        Err(_) => subsonic_error(0, "Failed to get lyrics"),
    }
}

async fn lyrics_by_song_id_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.get_lyrics_by_song_id(&id).await {
        Ok(lyrics_list) => {
            let lyrics_xml: String = lyrics_list.iter().map(|l| {
                format!(r#"<structuredLyrics displayArtist="{}" displayTitle="{}" lang="{}" synced="{}"><line>{}</line></structuredLyrics>"#,
                    l.display_artist, l.display_title, l.lang, l.synced, l.lyrics.join("</line><line>"))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <lyricsList>{}</lyricsList>
</subsonic-response>"#, SUBSONIC_API_VERSION, lyrics_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get lyrics"),
    }
}

async fn avatar_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let username = params.get("username").cloned().unwrap_or_default();

    match state.storage.get_avatar_path(&username).await {
        Ok(Some(_path)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <avatar username="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, username)
        }
        Ok(None) => subsonic_error(70, "Avatar not found"),
        Err(_) => subsonic_error(0, "Failed to get avatar"),
    }
}

async fn star_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id_str = params.get("id").map(|s| s.split(',').collect::<Vec<_>>());
    let album_id_str = params.get("albumId").map(|s| s.split(',').collect::<Vec<_>>());
    let artist_id_str = params.get("artistId").map(|s| s.split(',').collect::<Vec<_>>());

    let ids: Vec<&str> = id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let album_ids: Vec<&str> = album_id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let artist_ids: Vec<&str> = artist_id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

    match state.storage.star(&ids, &album_ids, &artist_ids).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to star"),
    }
}

async fn unstar_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id_str = params.get("id").map(|s| s.split(',').collect::<Vec<_>>());
    let album_id_str = params.get("albumId").map(|s| s.split(',').collect::<Vec<_>>());
    let artist_id_str = params.get("artistId").map(|s| s.split(',').collect::<Vec<_>>());

    let ids: Vec<&str> = id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let album_ids: Vec<&str> = album_id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let artist_ids: Vec<&str> = artist_id_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);

    match state.storage.unstar(&ids, &album_ids, &artist_ids).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to unstar"),
    }
}

async fn rating_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let rating = params.get("rating").and_then(|v| v.parse().ok()).unwrap_or(0);

    match state.storage.set_rating(&id, rating).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to set rating"),
    }
}

async fn scrobble_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let time = params.get("time").and_then(|v| v.parse().ok());
    let submission = params.get("submission").map(|s| s == "true" || s == "1").unwrap_or(true);

    match state.storage.scrobble(&id, time, submission).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to scrobble"),
    }
}

async fn bookmarks_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_bookmarks().await {
        Ok(bookmarks) => {
            let bookmarks_xml: String = bookmarks.iter().map(|b| {
                format!(r#"<bookmark position="{}" username="{}" comment="{}" created="{}" changed="{}"><entry id="{}"/></bookmark>"#,
                    b.position, b.username, b.comment.as_deref().unwrap_or(""),
                    b.created.as_deref().unwrap_or(""), b.changed.as_deref().unwrap_or(""), b.entry.id)
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <bookmarks>{}</bookmarks>
</subsonic-response>"#, SUBSONIC_API_VERSION, bookmarks_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get bookmarks"),
    }
}

async fn create_bookmark_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let position = params.get("position").and_then(|v| v.parse().ok()).unwrap_or(0);
    let comment = params.get("comment").map(|s| s.as_str());

    match state.storage.create_bookmark(&id, position, comment).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to create bookmark"),
    }
}

async fn delete_bookmark_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.delete_bookmark(&id).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to delete bookmark"),
    }
}

async fn play_queue_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_play_queue().await {
        Ok(Some(queue)) => {
            let entries_xml: String = queue.entry.iter().map(|s| {
                format!(r#"<entry id="{}" parent="{}" title="{}" album="{}" artist="{}" duration="{}" size="{}" contentType="{}" suffix="{}"/>"#,
                    s.id, s.parent.as_deref().unwrap_or(""), s.title, s.album.as_deref().unwrap_or(""),
                    s.artist.as_deref().unwrap_or(""), s.duration.map(|d| d.to_string()).as_deref().unwrap_or(""),
                    s.size.map(|s| s.to_string()).as_deref().unwrap_or(""),
                    s.content_type.as_deref().unwrap_or(""), s.suffix.as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <playQueue username="{}" current="{}" position="{}" changed="{}" changedBy="{}">{}</playQueue>
</subsonic-response>"#, SUBSONIC_API_VERSION, queue.username, queue.current.as_deref().unwrap_or(""), queue.position.map(|p| p.to_string()).as_deref().unwrap_or(""), queue.changed.as_deref().unwrap_or(""), queue.changed_by.as_deref().unwrap_or(""), entries_xml)
        }
        Ok(None) => subsonic_error(70, "Play queue not found"),
        Err(_) => subsonic_error(0, "Failed to get play queue"),
    }
}

async fn save_play_queue_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let ids_str = params.get("ids").map(|s| s.split(',').collect::<Vec<_>>());
    let ids: Vec<&str> = ids_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let current = params.get("current").map(|s| s.as_str());
    let position = params.get("position").and_then(|v| v.parse().ok());

    match state.storage.save_play_queue(&ids, current, position).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to save play queue"),
    }
}

async fn shares_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_shares().await {
        Ok(shares) => {
            let shares_xml: String = shares.iter().map(|s| {
                format!(r#"<share id="{}" url="{}" description="{}" username="{}" created="{}" expires="{}" visitCount="{}"><entry id="{}"/></share>"#,
                    s.id, s.url, s.description.as_deref().unwrap_or(""), s.username,
                    s.created.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""),
                    s.expires.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""),
                    s.visit_count, s.entry.first().map(|e| e.id.as_str()).unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <shares>{}</shares>
</subsonic-response>"#, SUBSONIC_API_VERSION, shares_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get shares"),
    }
}

async fn create_share_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let ids_str = params.get("ids").map(|s| s.split(',').collect::<Vec<_>>());
    let ids: Vec<&str> = ids_str.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
    let description = params.get("description").map(|s| s.as_str());
    let expires = params.get("expires").and_then(|v| v.parse().ok());

    match state.storage.create_share(&ids, description, expires).await {
        Ok(share) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <shares><share id="{}" url="{}" username="{}" created="{}" expires="{}" visitCount="0"/></shares>
</subsonic-response>"#, SUBSONIC_API_VERSION, share.id, share.url, share.username, share.created.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""), share.expires.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""))
        }
        Err(_) => subsonic_error(0, "Failed to create share"),
    }
}

async fn update_share_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let description = params.get("description").map(|s| s.as_str());
    let expires = params.get("expires").and_then(|v| v.parse().ok());

    match state.storage.update_share(&id, description, expires).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to update share"),
    }
}

async fn delete_share_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.delete_share(&id).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to delete share"),
    }
}

async fn radio_stations_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_internet_radio_stations().await {
        Ok(stations) => {
            let stations_xml: String = stations.iter().map(|s| {
                format!(r#"<internetRadioStation id="{}" name="{}" streamUrl="{}" homePageUrl="{}"/>"#,
                    s.id, s.name, s.stream_url, s.homepage_url.as_deref().unwrap_or(""))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <internetRadioStations>{}</internetRadioStations>
</subsonic-response>"#, SUBSONIC_API_VERSION, stations_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get radio stations"),
    }
}

async fn create_radio_station_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let stream_url = params.get("streamUrl").cloned().unwrap_or_default();
    let name = params.get("name").cloned().unwrap_or_default();
    let homepage_url = params.get("homePageUrl").map(|s| s.as_str());

    match state.storage.create_internet_radio_station(&stream_url, &name, homepage_url).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to create radio station"),
    }
}

async fn update_radio_station_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();
    let stream_url = params.get("streamUrl").cloned().unwrap_or_default();
    let name = params.get("name").cloned().unwrap_or_default();
    let homepage_url = params.get("homePageUrl").map(|s| s.as_str());

    match state.storage.update_internet_radio_station(&id, &stream_url, &name, homepage_url).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to update radio station"),
    }
}

async fn delete_radio_station_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let id = params.get("id").cloned().unwrap_or_default();

    match state.storage.delete_internet_radio_station(&id).await {
        Ok(_) => subsonic_ok(),
        Err(_) => subsonic_error(0, "Failed to delete radio station"),
    }
}

async fn user_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, uri: Uri) -> String {
    let params = state.get_params(&uri).await;
    let username = params.get("username").cloned().unwrap_or_default();

    match state.storage.get_user(&username).await {
        Ok(Some(user)) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <user id="{}" username="{}" email="{}" isAdmin="{}" isGuest="{}" isSaved="{}" canChangePassword="{}" canAccessAllFolders="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, user.id, user.username, user.email.as_deref().unwrap_or(""), if user.admin_role.unwrap_or(false) { "true" } else { "false" }, if user.guest_role.unwrap_or(false) { "true" } else { "false" }, "false", if user.can_change_password { "true" } else { "false" }, "true")
        }
        Ok(None) => subsonic_error(70, "User not found"),
        Err(_) => subsonic_error(0, "Failed to get user"),
    }
}

async fn users_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_users().await {
        Ok(users) => {
            let users_xml: String = users.iter().map(|u| {
                format!(r#"<user id="{}" username="{}" email="{}" isAdmin="{}" isGuest="{}" isSaved="{}" canChangePassword="{}" canAccessAllFolders="{}"/>"#,
                    u.id, u.username, u.email.as_deref().unwrap_or(""), if u.admin_role.unwrap_or(false) { "true" } else { "false" }, if u.guest_role.unwrap_or(false) { "true" } else { "false" }, "false", if u.can_change_password { "true" } else { "false" }, "true")
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <users>{}</users>
</subsonic-response>"#, SUBSONIC_API_VERSION, users_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get users"),
    }
}

async fn scan_status_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_scan_status().await {
        Ok(status) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <scanStatus scanning="{}" count="{}" folderCount="{}" lastScan="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, if status.scanning { "true" } else { "false" }, status.count, status.folder_count, status.last_scan.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""))
        }
        Err(_) => subsonic_error(0, "Failed to get scan status"),
    }
}

async fn start_scan_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.start_scan().await {
        Ok(status) => {
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <scanStatus scanning="true" count="{}" folderCount="{}" lastScan="{}"/>
</subsonic-response>"#, SUBSONIC_API_VERSION, status.count, status.folder_count, status.last_scan.map(|t| t.to_rfc3339()).as_deref().unwrap_or(""))
        }
        Err(_) => subsonic_error(0, "Failed to start scan"),
    }
}

async fn open_subsonic_extensions_handler<S: SubsonicStorage + Clone>(State(state): State<SubsonicState<S>>, _: Uri) -> String {
    match state.storage.get_open_subsonic_extensions().await {
        Ok(extensions) => {
            let extensions_xml: String = extensions.iter().map(|e| {
                format!(r#"<openSubsonicExtension name="{}" versions="{}"/>"#, e.name, e.versions.join(","))
            }).collect();
            format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="{}">
    <openSubsonicExtensions>{}</openSubsonicExtensions>
</subsonic-response>"#, SUBSONIC_API_VERSION, extensions_xml)
        }
        Err(_) => subsonic_error(0, "Failed to get extensions"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use reverie_core::{SubsonicDirectory, SubsonicArtist, SubsonicAlbum, MediaFile, SubsonicGenre, SubsonicMusicFolder};
    use reverie_storage::StorageError;
    use std::sync::Arc;

    #[derive(Clone)]
    struct MockStorage;

    #[async_trait]
    impl SubsonicStorage for MockStorage {
        async fn get_license(&self) -> Result<bool, StorageError> { Ok(true) }
        async fn get_music_folders(&self) -> Result<Vec<SubsonicMusicFolder>, StorageError> {
            Ok(vec![SubsonicMusicFolder { id: "1".to_string(), name: "Music".to_string() }])
        }
        async fn get_indexes(&self, _: Option<i32>, _: Option<i64>) -> Result<reverie_core::SubsonicArtistIndexes, StorageError> {
            Ok(vec![])
        }
        async fn get_genres(&self) -> Result<Vec<SubsonicGenre>, StorageError> {
            Ok(vec![SubsonicGenre { value: "Rock".to_string(), song_count: 10, album_count: 5 }])
        }
        async fn get_music_directory(&self, _: &str) -> Result<Option<SubsonicDirectory>, StorageError> { Ok(None) }
        async fn get_artists(&self, _: Option<i32>) -> Result<reverie_core::SubsonicArtistIndexes, StorageError> { Ok(vec![]) }
        async fn get_artist(&self, _: &str) -> Result<Option<SubsonicArtist>, StorageError> { Ok(None) }
        async fn get_album(&self, _: &str) -> Result<Option<SubsonicAlbum>, StorageError> { Ok(None) }
        async fn get_song(&self, _: &str) -> Result<Option<MediaFile>, StorageError> { Ok(None) }
        async fn get_artist_info(&self, _: &str, _: Option<i32>, _: Option<bool>) -> Result<reverie_core::SubsonicArtistInfo, StorageError> { Ok(Default::default()) }
        async fn get_artist_info2(&self, _: &str, _: Option<i32>, _: Option<bool>) -> Result<reverie_core::SubsonicArtistInfo, StorageError> { Ok(Default::default()) }
        async fn get_album_info(&self, _: &str) -> Result<reverie_core::SubsonicAlbumInfo, StorageError> { Ok(Default::default()) }
        async fn get_album_info2(&self, _: &str) -> Result<reverie_core::SubsonicAlbumInfo, StorageError> { Ok(Default::default()) }
        async fn get_similar_songs(&self, _: &str, _: Option<i32>) -> Result<Vec<MediaFile>, StorageError> { Ok(vec![]) }
        async fn get_similar_songs2(&self, _: &str, _: Option<i32>) -> Result<Vec<MediaFile>, StorageError> { Ok(vec![]) }
        async fn get_top_songs(&self, _: &str, _: Option<i32>) -> Result<reverie_core::SubsonicTopSongs, StorageError> { Ok(Default::default()) }
        async fn get_album_list(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<&str>, _: Option<i32>) -> Result<Vec<SubsonicAlbum>, StorageError> { Ok(vec![]) }
        async fn get_album_list2(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<&str>, _: Option<i32>) -> Result<Vec<SubsonicAlbum>, StorageError> { Ok(vec![]) }
        async fn get_random_songs(&self, _: Option<i32>, _: Option<&str>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<Vec<MediaFile>, StorageError> { Ok(vec![]) }
        async fn get_songs_by_genre(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<Vec<MediaFile>, StorageError> { Ok(vec![]) }
        async fn get_now_playing(&self) -> Result<Vec<reverie_core::SubsonicNowPlaying>, StorageError> { Ok(vec![]) }
        async fn get_starred(&self, _: Option<i32>) -> Result<reverie_core::SubsonicStarred, StorageError> { Ok(Default::default()) }
        async fn get_starred2(&self, _: Option<i32>) -> Result<reverie_core::SubsonicStarred, StorageError> { Ok(Default::default()) }
        async fn search2(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<reverie_core::SubsonicSearchResult2, StorageError> { Ok(Default::default()) }
        async fn search3(&self, _: &str, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>, _: Option<i32>) -> Result<reverie_core::SubsonicSearchResult3, StorageError> { Ok(Default::default()) }
        async fn get_playlists(&self, _: Option<&str>) -> Result<Vec<reverie_core::SubsonicPlaylist>, StorageError> { Ok(vec![]) }
        async fn get_playlist(&self, _: &str) -> Result<Option<reverie_core::SubsonicPlaylistWithSongs>, StorageError> { Ok(None) }
        async fn create_playlist(&self, _: Option<&str>, _: Option<&str>, _: &[&str]) -> Result<reverie_core::SubsonicPlaylistWithSongs, StorageError> { Ok(Default::default()) }
        async fn update_playlist(&self, _: &str, _: Option<&str>, _: Option<&str>, _: Option<bool>, _: &[&str], _: &[i32]) -> Result<(), StorageError> { Ok(()) }
        async fn delete_playlist(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn get_stream_path(&self, _: &str) -> Result<Option<String>, StorageError> { Ok(None) }
        async fn get_cover_art_path(&self, _: &str) -> Result<Option<String>, StorageError> { Ok(None) }
        async fn get_lyrics(&self, _: Option<&str>, _: Option<&str>) -> Result<Option<reverie_core::SubsonicLyrics>, StorageError> { Ok(None) }
        async fn get_lyrics_by_song_id(&self, _: &str) -> Result<Vec<reverie_core::SubsonicStructuredLyrics>, StorageError> { Ok(vec![]) }
        async fn get_avatar_path(&self, _: &str) -> Result<Option<String>, StorageError> { Ok(None) }
        async fn star(&self, _: &[&str], _: &[&str], _: &[&str]) -> Result<(), StorageError> { Ok(()) }
        async fn unstar(&self, _: &[&str], _: &[&str], _: &[&str]) -> Result<(), StorageError> { Ok(()) }
        async fn set_rating(&self, _: &str, _: i32) -> Result<(), StorageError> { Ok(()) }
        async fn scrobble(&self, _: &str, _: Option<i64>, _: bool) -> Result<(), StorageError> { Ok(()) }
        async fn get_bookmarks(&self) -> Result<Vec<reverie_core::SubsonicBookmark>, StorageError> { Ok(vec![]) }
        async fn create_bookmark(&self, _: &str, _: i64, _: Option<&str>) -> Result<(), StorageError> { Ok(()) }
        async fn delete_bookmark(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn get_play_queue(&self) -> Result<Option<reverie_core::SubsonicPlayQueue>, StorageError> { Ok(None) }
        async fn save_play_queue(&self, _: &[&str], _: Option<&str>, _: Option<i64>) -> Result<(), StorageError> { Ok(()) }
        async fn get_shares(&self) -> Result<Vec<reverie_core::SubsonicShare>, StorageError> { Ok(vec![]) }
        async fn create_share(&self, _: &[&str], _: Option<&str>, _: Option<i64>) -> Result<reverie_core::SubsonicShare, StorageError> { Ok(Default::default()) }
        async fn update_share(&self, _: &str, _: Option<&str>, _: Option<i64>) -> Result<(), StorageError> { Ok(()) }
        async fn delete_share(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn get_internet_radio_stations(&self) -> Result<Vec<reverie_core::SubsonicInternetRadioStation>, StorageError> { Ok(vec![]) }
        async fn create_internet_radio_station(&self, _: &str, _: &str, _: Option<&str>) -> Result<(), StorageError> { Ok(()) }
        async fn update_internet_radio_station(&self, _: &str, _: &str, _: &str, _: Option<&str>) -> Result<(), StorageError> { Ok(()) }
        async fn delete_internet_radio_station(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn get_user(&self, _: &str) -> Result<Option<reverie_core::SubsonicUser>, StorageError> { Ok(None) }
        async fn get_users(&self) -> Result<Vec<reverie_core::SubsonicUser>, StorageError> { Ok(vec![]) }
        async fn create_user(&self, _: &str, _: &str, _: Option<&str>, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: bool, _: &[i32]) -> Result<(), StorageError> { Ok(()) }
        async fn update_user(&self, _: &str, _: Option<&str>, _: Option<&str>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<bool>, _: Option<i32>) -> Result<(), StorageError> { Ok(()) }
        async fn delete_user(&self, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn change_password(&self, _: &str, _: &str) -> Result<(), StorageError> { Ok(()) }
        async fn get_scan_status(&self) -> Result<reverie_core::SubsonicScanStatus, StorageError> { Ok(Default::default()) }
        async fn start_scan(&self) -> Result<reverie_core::SubsonicScanStatus, StorageError> { Ok(Default::default()) }
        async fn get_open_subsonic_extensions(&self) -> Result<Vec<reverie_core::OpenSubsonicExtension>, StorageError> { Ok(vec![]) }
    }

    #[test]
    fn test_get_query_params_with_uri() {
        use http::Uri;
        let uri = Uri::from_static("/rest/getSong?id=123&type=album");
        let params = get_query_params(&uri);
        assert_eq!(params.get("id"), Some(&"123".to_string()));
        assert_eq!(params.get("type"), Some(&"album".to_string()));
    }

    #[test]
    fn test_get_query_params_empty_uri() {
        use http::Uri;
        let uri = Uri::from_static("/rest/ping");
        let params = get_query_params(&uri);
        assert!(params.is_empty());
    }

    #[test]
    fn test_get_query_params_complex_query() {
        use http::Uri;
        let uri = Uri::from_static("/rest/search3?query=rock&artistCount=5&albumCount=10&songCount=20");
        let params = get_query_params(&uri);
        assert_eq!(params.get("query"), Some(&"rock".to_string()));
        assert_eq!(params.get("artistCount"), Some(&"5".to_string()));
        assert_eq!(params.get("albumCount"), Some(&"10".to_string()));
        assert_eq!(params.get("songCount"), Some(&"20".to_string()));
    }

    #[test]
    fn test_subsonic_api_version_constant() {
        assert_eq!(SUBSONIC_API_VERSION, "1.16.1");
    }

    #[tokio::test]
    async fn test_ping_handler_returns_ok() {
        let storage = Arc::new(MockStorage);
        let state = SubsonicState::new(storage);
        let response = ping_handler(State(state), Uri::from_static("/rest/ping")).await;
        assert!(response.contains(r#"status="ok""#));
        assert!(response.contains(r#"version="1.16.1""#));
    }

    #[tokio::test]
    async fn test_genres_handler_returns_xml() {
        let storage = Arc::new(MockStorage);
        let state = SubsonicState::new(storage);
        let response = genres_handler(State(state), Uri::from_static("/rest/getGenres")).await;
        assert!(response.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(response.contains(r#"<subsonic-response"#));
        assert!(response.contains(r#"<genres>"#));
    }

    #[tokio::test]
    async fn test_music_folders_handler_returns_xml() {
        let storage = Arc::new(MockStorage);
        let state = SubsonicState::new(storage);
        let response = music_folders_handler(State(state), Uri::from_static("/rest/getMusicFolders")).await;
        assert!(response.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(response.contains(r#"<subsonic-response"#));
        assert!(response.contains(r#"<musicFolders>"#));
        assert!(response.contains(r#"musicFolder"#));
    }

    #[tokio::test]
    async fn test_error_response_for_missing_resource() {
        let storage = Arc::new(MockStorage);
        let state = SubsonicState::new(storage);
        let response = artist_handler(State(state), Uri::from_static("/rest/getArtist?id=missing")).await;
        assert!(response.contains(r#"status="failed""#));
        assert!(response.contains(r#"error code="70""#));
        assert!(response.contains("Artist not found"));
    }

    #[test]
    fn test_create_router_has_all_routes() {
        let storage = Arc::new(MockStorage);
        let router = create_router(storage);
        let routes: Vec<_> = router.iter().collect();
        let route_paths: Vec<&str> = routes.iter().filter_map(|r| r.path()).collect();

        assert!(route_paths.contains(&"/ping"));
        assert!(route_paths.contains(&"/getLicense"));
        assert!(route_paths.contains(&"/getMusicFolders"));
        assert!(route_paths.contains(&"/getIndexes"));
        assert!(route_paths.contains(&"/getGenres"));
        assert!(route_paths.contains(&"/getArtists"));
        assert!(route_paths.contains(&"/getAlbum"));
        assert!(route_paths.contains(&"/getSong"));
        assert!(route_paths.contains(&"/search2"));
        assert!(route_paths.contains(&"/search3"));
        assert!(route_paths.contains(&"/stream"));
        assert!(route_paths.contains(&"/getCoverArt"));
    }

    #[test]
    fn test_router_has_expected_route_count() {
        let storage = Arc::new(MockStorage);
        let router = create_router(storage);
        let routes: Vec<_> = router.iter().collect();
        assert_eq!(routes.len(), 47);
    }
}
