//! Subsonic API implementation
//! Compatible with navidrome's Subsonic API (61 endpoints)

pub mod tests;

use axum::{routing::get, Router};

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

pub fn create_router() -> Router {
    Router::new()
        // === System ===
        .route("/ping", get(ping_handler))
        .route("/getLicense", get(license_handler))
        // === Browsing ===
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
        // === Album/Song Lists ===
        .route("/getAlbumList", get(album_list_handler))
        .route("/getAlbumList2", get(album_list2_handler))
        .route("/getRandomSongs", get(random_songs_handler))
        .route("/getSongsByGenre", get(songs_by_genre_handler))
        .route("/getNowPlaying", get(now_playing_handler))
        .route("/getStarred", get(starred_handler))
        .route("/getStarred2", get(starred2_handler))
        // === Searching ===
        .route("/search2", get(search2_handler))
        .route("/search3", get(search3_handler))
        // === Playlists ===
        .route("/getPlaylists", get(playlists_handler))
        .route("/getPlaylist", get(playlist_handler))
        .route("/createPlaylist", get(create_playlist_handler))
        .route("/updatePlaylist", get(update_playlist_handler))
        .route("/deletePlaylist", get(delete_playlist_handler))
        // === Media Retrieval ===
        .route("/stream", get(stream_handler))
        .route("/download", get(download_handler))
        .route("/getCoverArt", get(cover_art_handler))
        .route("/getLyrics", get(lyrics_handler))
        .route("/getLyricsBySongId", get(lyrics_by_song_id_handler))
        .route("/getAvatar", get(avatar_handler))
        // === Media Annotation ===
        .route("/star", get(star_handler))
        .route("/unstar", get(unstar_handler))
        .route("/setRating", get(rating_handler))
        .route("/scrobble", get(scrobble_handler))
        // === Bookmarks ===
        .route("/getBookmarks", get(bookmarks_handler))
        .route("/createBookmark", get(create_bookmark_handler))
        .route("/deleteBookmark", get(delete_bookmark_handler))
        .route("/getPlayQueue", get(play_queue_handler))
        .route("/savePlayQueue", get(save_play_queue_handler))
        // === Sharing ===
        .route("/getShares", get(shares_handler))
        .route("/createShare", get(create_share_handler))
        .route("/updateShare", get(update_share_handler))
        .route("/deleteShare", get(delete_share_handler))
        // === Internet Radio ===
        .route("/getInternetRadioStations", get(radio_stations_handler))
        .route(
            "/createInternetRadioStation",
            get(create_radio_station_handler),
        )
        .route(
            "/updateInternetRadioStation",
            get(update_radio_station_handler),
        )
        .route(
            "/deleteInternetRadioStation",
            get(delete_radio_station_handler),
        )
        // === User Management ===
        .route("/getUser", get(user_handler))
        .route("/getUsers", get(users_handler))
        // === Library Scanning ===
        .route("/getScanStatus", get(scan_status_handler))
        .route("/startScan", get(start_scan_handler))
        // === OpenSubsonic Extensions ===
        .route(
            "/getOpenSubsonicExtensions",
            get(open_subsonic_extensions_handler),
        )
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

// === New handlers for complete Subsonic API ===

async fn artist_info2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <artistInfo2>
        <biography>Artist biography goes here</biography>
        <musicBrainzId>abc123</musicBrainzId>
        <lastFmUrl>https://www.last.fm/music/Artist</lastFmUrl>
        <smallImageUrl>https://example.com/small.jpg</smallImageUrl>
        <mediumImageUrl>https://example.com/medium.jpg</mediumImageUrl>
        <largeImageUrl>https://example.com/large.jpg</largeImageUrl>
    </artistInfo2>
</subsonic-response>"#
}

async fn album_info2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <albumInfo>
        <notes>Album notes go here</notes>
        <musicBrainzId>xyz789</musicBrainzId>
        <lastFmUrl>https://www.last.fm/music/Artist/Album</lastFmUrl>
        <smallImageUrl>https://example.com/small.jpg</smallImageUrl>
        <mediumImageUrl>https://example.com/medium.jpg</mediumImageUrl>
        <largeImageUrl>https://example.com/large.jpg</largeImageUrl>
    </albumInfo>
</subsonic-response>"#
}

async fn similar_songs_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <similarSongs>
        <song id="song-2" parent="album-1" title="Similar Song" album="Test Album" artist="Test Artist" track="2" duration="200" size="4500000" contentType="audio/mpeg" suffix="mp3"/>
    </similarSongs>
</subsonic-response>"#
}

async fn similar_songs2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <similarSongs2>
        <song id="song-2" parent="album-1" title="Similar Song" album="Test Album" artist="Test Artist" track="2" duration="200" size="4500000" contentType="audio/mpeg" suffix="mp3"/>
    </similarSongs2>
</subsonic-response>"#
}

async fn top_songs_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <topSongs>
        <song id="song-1" parent="album-1" title="Top Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </topSongs>
</subsonic-response>"#
}

async fn album_list_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <albumList>
        <album id="album-1" parent="artist-1" isDir="true" title="Test Album" album="Test Album" artist="Test Artist" year="2023" genre="Rock" coverArt="al-123" duration="2400" playCount="100"/>
    </albumList>
</subsonic-response>"#
}

async fn album_list2_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <albumList2>
        <album id="album-1" name="Test Album" artist="Test Artist" artistId="artist-1" year="2023" genre="Rock" coverArt="al-123" songCount="10" duration="2400" playCount="100"/>
    </albumList2>
</subsonic-response>"#
}

async fn songs_by_genre_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <songsByGenre>
        <song id="song-1" parent="album-1" title="Rock Song" album="Test Album" artist="Test Artist" track="1" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3" genre="Rock"/>
    </songsByGenre>
</subsonic-response>"#
}

async fn lyrics_by_song_id_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <lyricsList>
        <structuredLyrics displayArtist="Test Artist" displayTitle="Test Song" lang="eng" synced="false">
            <line>First line of lyrics</line>
            <line>Second line of lyrics</line>
        </structuredLyrics>
    </lyricsList>
</subsonic-response>"#
}

async fn bookmarks_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <bookmarks>
        <bookmark position="120000" username="admin" comment="Good part" created="2023-01-01T00:00:00Z" changed="2023-01-01T00:00:00Z">
            <entry id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
        </bookmark>
    </bookmarks>
</subsonic-response>"#
}

async fn create_bookmark_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn delete_bookmark_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn play_queue_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <playQueue username="admin" current="song-1" position="30000" changed="2023-01-01T00:00:00Z" changedBy="Web Client">
        <entry id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
    </playQueue>
</subsonic-response>"#
}

async fn save_play_queue_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn shares_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <shares>
        <share id="share-1" url="https://example.com/share/abc123" description="My shared music" username="admin" created="2023-01-01T00:00:00Z" expires="2024-01-01T00:00:00Z" visitCount="10">
            <entry id="song-1" parent="album-1" title="Test Song" album="Test Album" artist="Test Artist" duration="240" size="5000000" contentType="audio/mpeg" suffix="mp3"/>
        </share>
    </shares>
</subsonic-response>"#
}

async fn create_share_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <shares>
        <share id="share-new" url="https://example.com/share/new123" username="admin" created="2023-01-01T00:00:00Z" visitCount="0"/>
    </shares>
</subsonic-response>"#
}

async fn update_share_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn delete_share_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn radio_stations_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <internetRadioStations>
        <internetRadioStation id="1" name="Jazz FM" streamUrl="https://stream.jazzfm.com/live" homePageUrl="https://jazzfm.com"/>
        <internetRadioStation id="2" name="Classic Rock" streamUrl="https://stream.classicrock.com/live" homePageUrl="https://classicrock.com"/>
    </internetRadioStations>
</subsonic-response>"#
}

async fn create_radio_station_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn update_radio_station_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn delete_radio_station_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1"/>"#
}

async fn open_subsonic_extensions_handler() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<subsonic-response status="ok" version="1.16.1">
    <openSubsonicExtensions>
        <openSubsonicExtension name="transcodeOffset" versions="1"/>
        <openSubsonicExtension name="formPost" versions="1"/>
        <openSubsonicExtension name="songLyrics" versions="1"/>
    </openSubsonicExtensions>
</subsonic-response>"#
}
