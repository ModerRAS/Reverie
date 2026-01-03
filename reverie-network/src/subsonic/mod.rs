//! Subsonic API implementation
//! Compatible with navidrome's Subsonic API

pub mod tests;

use axum::{routing::get, Router};

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

pub fn create_router() -> Router {
    Router::new()
        .route("/ping", get(ping_handler))
        .route("/getLicense", get(license_handler))
        .route("/getMusicFolders", get(music_folders_handler))
        .route("/getGenres", get(genres_handler))
        .route("/getScanStatus", get(scan_status_handler))
        .route("/startScan", get(start_scan_handler))
        .route("/getIndexes", get(indexes_handler))
        .route("/getArtists", get(artists_handler))
        .route("/getPlaylists", get(playlists_handler))
        .route("/getMusicDirectory", get(music_directory_handler))
        .route("/getArtist", get(artist_handler))
        .route("/getAlbum", get(album_handler))
        .route("/getSong", get(song_handler))
        .route("/getAlbumInfo", get(album_info_handler))
        .route("/getArtistInfo", get(artist_info_handler))
        .route("/search2", get(search2_handler))
        .route("/search3", get(search3_handler))
        .route("/getPlaylist", get(playlist_handler))
        .route("/createPlaylist", get(create_playlist_handler))
        .route("/updatePlaylist", get(update_playlist_handler))
        .route("/deletePlaylist", get(delete_playlist_handler))
        .route("/stream", get(stream_handler))
        .route("/download", get(download_handler))
        .route("/getCoverArt", get(cover_art_handler))
        .route("/getAvatar", get(avatar_handler))
        .route("/getUser", get(user_handler))
        .route("/getUsers", get(users_handler))
        .route("/getStarred", get(starred_handler))
        .route("/getStarred2", get(starred2_handler))
        .route("/star", get(star_handler))
        .route("/unstar", get(unstar_handler))
        .route("/setRating", get(rating_handler))
        .route("/scrobble", get(scrobble_handler))
        .route("/getNowPlaying", get(now_playing_handler))
        .route("/getLyrics", get(lyrics_handler))
        .route("/getRandomSongs", get(random_songs_handler))
}

async fn ping_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn license_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <license valid="true"/>
</subsonic-response>"#
}

async fn music_folders_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <musicFolders>
        <musicFolder id="1" name="Music"/>
    </musicFolders>
</subsonic-response>"#
}

async fn genres_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <genres>
        <genre name="Rock" songCount="10" albumCount="5"/>
        <genre name="Pop" songCount="8" albumCount="4"/>
    </genres>
</subsonic-response>"#
}

async fn scan_status_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <scanStatus scanning="false" count="100" folderCount="1"/>
</subsonic-response>"#
}

async fn start_scan_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <scanStatus scanning="true" count="0" folderCount="1"/>
</subsonic-response>"#
}

async fn indexes_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <indexes lastModified="0">
        <index name="A"/>
    </indexes>
</subsonic-response>"#
}

async fn artists_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <artists lastModified="0" ignoredArticles="The La Le"/>
</subsonic-response>"#
}

async fn playlists_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <playlists/>
</subsonic-response>"#
}

async fn music_directory_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <directory id="root" name="Music" childCount="2">
        <child id="artist-1" parent="root" isDir="true" title="Test Artist" albumCount="1" duration="2400" coverArt="ar-123"/>
        <child id="folder-1" parent="root" isDir="true" title="Subfolder" childCount="1"/>
    </directory>
</subsonic-response>"#
}

async fn artist_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <artist id="artist-1" name="Test Artist" albumCount="5" coverArt="ar-123"/>
</subsonic-response>"#
}

async fn album_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" year="2023" genre="Rock" songCount="10" duration="2400" coverArt="al-123"/>
</subsonic-response>"#
}

async fn song_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <song id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" albumArtist="Test Artist" track="1" disc="1" year="2023" genre="Rock" size="5000000" contentType="audio/mpeg" suffix="mp3" duration="240.0" bitRate="320" sampleRate="44100" channels="2" path="/music/test/song.mp3"/>
</subsonic-response>"#
}

async fn album_info_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <albumInfo id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" year="2023" genre="Rock" coverArt="al-123" songCount="10" duration="2400"/>
</subsonic-response>"#
}

async fn artist_info_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <artistInfo id="artist-1" name="Test Artist" coverArt="ar-123" albumCount="5"/>
</subsonic-response>"#
}

async fn search2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <searchResult2 totalHits="2">
        <artist id="artist-1" name="Test Artist" albumCount="1" coverArt="ar-123"/>
        <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" coverArt="al-123" songCount="10" duration="2400"/>
        <child id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </searchResult2>
</subsonic-response>"#
}

async fn search3_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <searchResult3 totalHits="2">
        <artist id="artist-1" name="Test Artist" albumCount="1" coverArt="ar-123"/>
        <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" coverArt="al-123" songCount="10" duration="2400"/>
        <song id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </searchResult3>
</subsonic-response>"#
}

async fn playlist_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <playlist id="playlist-1" name="My Playlist" owner="admin" songCount="5" duration="1200" public="false" created="2023-01-01T00:00:00Z" changed="2023-01-01T00:00:00Z">
        <entry id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </playlist>
</subsonic-response>"#
}

async fn create_playlist_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <playlist id="new-playlist-1" name="New Playlist" owner="admin" songCount="0" duration="0" public="false" created="2023-01-01T00:00:00Z" changed="2023-01-01T00:00:00Z"/>
</subsonic-response>"#
}

async fn update_playlist_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn delete_playlist_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn stream_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <stream id="song-1" format="mp3" bitRate="320" duration="240" size="5000000"/>
</subsonic-response>"#
}

async fn download_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn cover_art_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <coverArt id="al-123"/>
</subsonic-response>"#
}

async fn avatar_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <avatar username="admin"/>
</subsonic-response>"#
}

async fn user_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <user id="user-1" username="admin" email="admin@example.com" isAdmin="true" isGuest="false" isSaved="false" canChangePassword="true" canAccessAllFolders="true"/>
</subsonic-response>"#
}

async fn users_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <users>
        <user id="user-1" username="admin" email="admin@example.com" isAdmin="true" isGuest="false" isSaved="false" canChangePassword="true" canAccessAllFolders="true"/>
    </users>
</subsonic-response>"#
}

async fn starred_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <starred>
        <artist id="artist-1" name="Test Artist" albumCount="1" coverArt="ar-123"/>
        <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" coverArt="al-123" songCount="10" duration="2400"/>
        <song id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </starred>
</subsonic-response>"#
}

async fn starred2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <starred2>
        <artist id="artist-1" name="Test Artist" albumCount="1" coverArt="ar-123"/>
        <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" coverArt="al-123" songCount="10" duration="2400"/>
        <song id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </starred2>
</subsonic-response>"#
}

async fn star_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn unstar_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn rating_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn scrobble_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>
</subsonic-response>"#
}

async fn now_playing_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <nowPlaying>
        <entry id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3" username="admin" playerName="Web Client" playerType="WEB" milliseconds="120000"/>
    </nowPlaying>
</subsonic-response>"#
}

async fn lyrics_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <lyrics artist="Test Artist" title="Test Song">Test lyrics content</lyrics>
</subsonic-response>"#
}

async fn random_songs_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <randomSongs>
        <song id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
        <song id="song-2" parent="album-1" title="Another Song" album="Test Album" artist="Test Artist" track="2" duration="180" size="4000000" contentType="audio/mpeg" suffix="mp3"/>
    </randomSongs>
</subsonic-response>"#
}
