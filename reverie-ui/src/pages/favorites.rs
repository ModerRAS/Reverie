//! Favorites page - Starred content

use dioxus::prelude::*;
use crate::api::{Album, Artist, Song};
use crate::components::{AlbumCard, ArtistCard, TrackList, PageHeader, LoadingSpinner, EmptyState, TabBar};

/// Favorites page component
#[component]
pub fn FavoritesPage() -> Element {
    let mut active_tab = use_signal(|| 0usize);
    let mut starred_songs = use_signal(Vec::<Song>::new);
    let mut starred_albums = use_signal(Vec::<Album>::new);
    let mut starred_artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    let tabs = vec![
        "Songs".to_string(),
        "Albums".to_string(),
        "Artists".to_string(),
    ];

    // Load starred content
    use_effect(move || {
        loading.set(true);
        
        // Demo data - starred items
        let songs: Vec<Song> = (1..=10).map(|i| Song {
            id: format!("starred-song-{}", i),
            title: format!("Favorite Song {}", i),
            album: Some(format!("Album {}", i)),
            album_id: Some(format!("album-{}", i)),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            track: Some(i as i32),
            year: Some(2023),
            genre: None,
            cover_art: None,
            duration: Some(200 + i as i32 * 10),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: None,
            path: None,
            starred: Some("2024-01-01T00:00:00Z".to_string()),
            play_count: i as i32 * 20,
        }).collect();
        
        let albums: Vec<Album> = (1..=6).map(|i| Album {
            id: format!("starred-album-{}", i),
            name: format!("Favorite Album {}", i),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            cover_art: None,
            song_count: Some(10),
            duration: Some(2400),
            year: Some(2023),
            genre: None,
            created: None,
            starred: Some("2024-01-01T00:00:00Z".to_string()),
            play_count: 0,
        }).collect();
        
        let artists: Vec<Artist> = (1..=4).map(|i| Artist {
            id: format!("starred-artist-{}", i),
            name: format!("Favorite Artist {}", i),
            album_count: 5,
            cover_art: None,
            artist_image_url: None,
            starred: Some("2024-01-01T00:00:00Z".to_string()),
        }).collect();
        
        starred_songs.set(songs);
        starred_albums.set(albums);
        starred_artists.set(artists);
        loading.set(false);
    });

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    let on_artist_click = move |id: String| {
        navigator.push(format!("/artist/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",
            
            PageHeader {
                title: "Favorites".to_string(),
                subtitle: Some("Your starred music".to_string())
            }
            
            TabBar {
                tabs: tabs,
                active_index: active_tab(),
                on_change: move |idx| active_tab.set(idx)
            }
            
            if loading() {
                LoadingSpinner { message: "Loading favorites..." }
            } else {
                match active_tab() {
                    0 => rsx! {
                        if starred_songs.read().is_empty() {
                            EmptyState {
                                title: "No starred songs".to_string(),
                                message: Some("Star your favorite songs to see them here.".to_string())
                            }
                        } else {
                            TrackList {
                                tracks: starred_songs.read().clone(),
                                show_number: true,
                                show_album: true,
                                show_artist: true
                            }
                        }
                    },
                    1 => rsx! {
                        if starred_albums.read().is_empty() {
                            EmptyState {
                                title: "No starred albums".to_string(),
                                message: Some("Star your favorite albums to see them here.".to_string())
                            }
                        } else {
                            div {
                                class: "album-grid",
                                for album in starred_albums.read().iter() {
                                    AlbumCard {
                                        key: "{album.id}",
                                        album: album.clone(),
                                        on_click: on_album_click
                                    }
                                }
                            }
                        }
                    },
                    2 => rsx! {
                        if starred_artists.read().is_empty() {
                            EmptyState {
                                title: "No starred artists".to_string(),
                                message: Some("Star your favorite artists to see them here.".to_string())
                            }
                        } else {
                            div {
                                class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4",
                                for artist in starred_artists.read().iter() {
                                    ArtistCard {
                                        key: "{artist.id}",
                                        artist: artist.clone(),
                                        on_click: on_artist_click
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! {}
                }
            }
        }
    }
}
