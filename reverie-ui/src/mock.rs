//! Mock data provider for Reverie UI.
//!
//! The UI is currently built around simulated data. This module centralizes that data generation
//! so pages can share a consistent dataset during UI development/testing.

use crate::api::{Album, Artist, Playlist, Song};

const GENRES: [&str; 5] = ["Rock", "Pop", "Jazz", "Electronic", "Classical"];

fn genre_for(i: usize) -> String {
    GENRES[i % GENRES.len()].to_string()
}

fn year_for(i: usize) -> i32 {
    2020 + (i as i32 % 5)
}

fn artist_id_for(i: usize) -> String {
    format!("artist-{}", (i % 8) + 1)
}

fn artist_name_for(i: usize) -> String {
    format!("Artist {}", (i % 8) + 1)
}

fn parse_numeric_suffix(id: &str) -> Option<usize> {
    id.rsplit('-').next().and_then(|s| s.parse::<usize>().ok())
}

pub fn albums(count: usize) -> Vec<Album> {
    (1..=count)
        .map(|i| Album {
            id: format!("album-{}", i),
            name: format!("Album {}", i),
            artist: Some(artist_name_for(i)),
            artist_id: Some(artist_id_for(i)),
            cover_art: None,
            song_count: Some(10 + (i as i32 % 5)),
            duration: Some(2400 + i as i32 * 60),
            year: Some(year_for(i)),
            genre: Some(genre_for(i)),
            created: None,
            starred: None,
            play_count: (i as i32) * 10,
        })
        .collect()
}

pub fn artists(count: usize) -> Vec<Artist> {
    (1..=count)
        .map(|i| Artist {
            id: format!("artist-{}", i),
            name: format!("Artist {}", i),
            album_count: (i as i32 % 12) + 1,
            cover_art: None,
            artist_image_url: None,
            starred: None,
        })
        .collect()
}

pub fn songs(count: usize) -> Vec<Song> {
    (1..=count)
        .map(|i| Song {
            id: format!("song-{}", i),
            title: format!("Song {}", i),
            album: Some(format!("Album {}", (i % 12) + 1)),
            album_id: Some(format!("album-{}", (i % 12) + 1)),
            artist: Some(artist_name_for(i)),
            artist_id: Some(artist_id_for(i)),
            track: Some((i % 12) as i32 + 1),
            year: Some(year_for(i)),
            genre: Some(genre_for(i)),
            cover_art: None,
            duration: Some(180 + (i as i32 % 120)),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: Some("audio/mpeg".to_string()),
            path: Some(format!("/music/Artist {}/Album {}/Song {}.mp3", (i % 8) + 1, (i % 12) + 1, i)),
            starred: None,
            play_count: (i as i32) * 3,
        })
        .collect()
}

pub fn playlists(count: usize) -> Vec<Playlist> {
    (1..=count)
        .map(|i| Playlist {
            id: format!("playlist-{}", i),
            name: format!("Playlist {}", i),
            song_count: (i as i32 % 25) + 5,
            duration: ((i as i32 % 25) + 5) * 210,
            owner: Some("demo".to_string()),
            public: Some(i % 2 == 0),
            created: None,
            changed: None,
            cover_art: None,
            entry: Vec::new(),
        })
        .collect()
}

pub fn album_detail(id: &str) -> (Album, Vec<Song>) {
    let n = parse_numeric_suffix(id).unwrap_or(1);

    let album = Album {
        id: id.to_string(),
        name: format!("Album {}", n),
        artist: Some(artist_name_for(n)),
        artist_id: Some(artist_id_for(n)),
        cover_art: None,
        song_count: Some(12),
        duration: Some(12 * 210),
        year: Some(year_for(n)),
        genre: Some(genre_for(n)),
        created: None,
        starred: None,
        play_count: (n as i32) * 10,
    };

    let tracks = (1..=12)
        .map(|i| Song {
            id: format!("song-{}-{}", n, i),
            title: format!("Track {}", i),
            album: Some(album.name.clone()),
            album_id: Some(album.id.clone()),
            artist: album.artist.clone(),
            artist_id: album.artist_id.clone(),
            track: Some(i as i32),
            year: album.year,
            genre: album.genre.clone(),
            cover_art: None,
            duration: Some(180 + (i as i32 % 60)),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: Some("audio/mpeg".to_string()),
            path: Some(format!("/music/{}/{}.mp3", album.name, i)),
            starred: None,
            play_count: (i as i32) * 2,
        })
        .collect();

    (album, tracks)
}

pub fn artist_detail(id: &str) -> (Artist, Vec<Album>, Vec<Song>) {
    let n = parse_numeric_suffix(id).unwrap_or(1);

    let artist = Artist {
        id: id.to_string(),
        name: format!("Artist {}", n),
        album_count: 5,
        cover_art: None,
        artist_image_url: None,
        starred: None,
    };

    let albums = (1..=5)
        .map(|i| Album {
            id: format!("album-{}-{}", n, i),
            name: format!("Album {}-{}", n, i),
            artist: Some(artist.name.clone()),
            artist_id: Some(artist.id.clone()),
            cover_art: None,
            song_count: Some(10 + (i as i32 % 5)),
            duration: Some(2400 + i as i32 * 60),
            year: Some(2018 + (i as i32 % 7)),
            genre: Some(genre_for(n + i)),
            created: None,
            starred: None,
            play_count: (i as i32) * 7,
        })
        .collect();

    let top_songs = (1..=5)
        .map(|i| Song {
            id: format!("top-song-{}-{}", n, i),
            title: format!("Top Song {}", i),
            album: Some(format!("Album {}-{}", n, (i % 5) + 1)),
            album_id: Some(format!("album-{}-{}", n, (i % 5) + 1)),
            artist: Some(artist.name.clone()),
            artist_id: Some(artist.id.clone()),
            track: Some(i as i32),
            year: Some(2021),
            genre: Some(genre_for(n + i)),
            cover_art: None,
            duration: Some(200 + (i as i32 * 10)),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: Some("audio/mpeg".to_string()),
            path: Some(format!("/music/{}/Top Song {}.mp3", artist.name, i)),
            starred: None,
            play_count: 100 - (i as i32 * 10),
        })
        .collect();

    (artist, albums, top_songs)
}

pub fn playlist_detail(id: &str) -> Playlist {
    let n = parse_numeric_suffix(id).unwrap_or(1);
    let entries = (1..=15)
        .map(|i| Song {
            id: format!("playlist-song-{}-{}", n, i),
            title: format!("Playlist Song {}", i),
            album: Some(format!("Album {}", (i % 10) + 1)),
            album_id: Some(format!("album-{}", (i % 10) + 1)),
            artist: Some(artist_name_for(i)),
            artist_id: Some(artist_id_for(i)),
            track: Some(i as i32),
            year: Some(year_for(i)),
            genre: Some(genre_for(i)),
            cover_art: None,
            duration: Some(180 + (i as i32 % 120)),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: Some("audio/mpeg".to_string()),
            path: Some(format!("/music/playlist/{}/{}.mp3", id, i)),
            starred: None,
            play_count: (i as i32) * 2,
        })
        .collect::<Vec<_>>();

    let total_duration: i32 = entries.iter().filter_map(|s| s.duration).sum();

    Playlist {
        id: id.to_string(),
        name: format!("Playlist {}", n),
        song_count: entries.len() as i32,
        duration: total_duration,
        owner: Some("demo".to_string()),
        public: Some(n % 2 == 0),
        created: None,
        changed: None,
        cover_art: None,
        entry: entries,
    }
}

pub fn favorites() -> (Vec<Song>, Vec<Album>, Vec<Artist>) {
    let fav_songs = songs(10);
    let fav_albums = albums(6);
    let fav_artists = artists(4);
    (fav_songs, fav_albums, fav_artists)
}

pub fn home() -> (Vec<Album>, Vec<Song>) {
    (albums(6), songs(8))
}

pub fn search(query: &str) -> (Vec<Song>, Vec<Album>, Vec<Artist>) {
    let q = query.trim();
    if q.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    // Deterministic subset: keep a few items and inject query in titles.
    let mut result_songs = songs(5);
    for (i, s) in result_songs.iter_mut().enumerate() {
        s.title = format!("{} Result Song {}", q, i + 1);
    }

    let mut result_albums = albums(3);
    for (i, a) in result_albums.iter_mut().enumerate() {
        a.name = format!("{} Result Album {}", q, i + 1);
    }

    let mut result_artists = artists(2);
    for (i, ar) in result_artists.iter_mut().enumerate() {
        ar.name = format!("{} Result Artist {}", q, i + 1);
    }

    (result_songs, result_albums, result_artists)
}
