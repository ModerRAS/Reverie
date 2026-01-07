//! Home page - Dashboard with recently played, etc.

use dioxus::prelude::*;
use crate::api::{Album, Song};
use crate::components::{AlbumCard, SongCard, LoadingSpinner};

/// Home page component
#[component]
pub fn HomePage() -> Element {
    let mut recent_albums = use_signal(Vec::<Album>::new);
    let mut recent_songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // Load home page data
    use_effect(move || {
        loading.set(true);
        
        // Demo data
        let demo_albums: Vec<Album> = (1..=6).map(|i| Album {
            id: format!("recent-album-{}", i),
            name: format!("Recently Played {}", i),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            cover_art: None,
            song_count: Some(10),
            duration: Some(2400),
            year: Some(2023),
            genre: None,
            created: None,
            starred: None,
            play_count: 100 - i as i32 * 10,
        }).collect();
        
        let demo_songs: Vec<Song> = (1..=8).map(|i| Song {
            id: format!("recent-song-{}", i),
            title: format!("Recent Song {}", i),
            album: Some(format!("Album {}", i)),
            album_id: Some(format!("album-{}", i)),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            track: Some(1),
            year: Some(2023),
            genre: None,
            cover_art: None,
            duration: Some(200),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: None,
            path: None,
            starred: None,
            play_count: 50 - i as i32 * 5,
        }).collect();
        
        recent_albums.set(demo_albums);
        recent_songs.set(demo_songs);
        loading.set(false);
    });

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    if loading() {
        return rsx! {
            LoadingSpinner { message: "Loading..." }
        };
    }

    rsx! {
        div {
            class: "space-y-8",
            
            // Welcome header
            section {
                class: "mb-8",
                h1 { class: "text-3xl font-bold", "Good evening" }
                p { class: "text-gray-400 mt-1", "Here's what you've been listening to" }
            }
            
            // Recently played albums
            section {
                class: "space-y-4",
                div {
                    class: "flex items-center justify-between",
                    h2 { class: "text-2xl font-bold", "Recently Played" }
                    button {
                        class: "text-sm text-gray-400 hover:text-white",
                        "Show all"
                    }
                }
                div {
                    class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4",
                    for album in recent_albums.read().iter() {
                        AlbumCard {
                            key: "{album.id}",
                            album: album.clone(),
                            on_click: on_album_click
                        }
                    }
                }
            }
            
            // Recently added section
            section {
                class: "space-y-4",
                div {
                    class: "flex items-center justify-between",
                    h2 { class: "text-2xl font-bold", "Recently Added" }
                    button {
                        class: "text-sm text-gray-400 hover:text-white",
                        "Show all"
                    }
                }
                div {
                    class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-2",
                    for song in recent_songs.read().iter() {
                        SongCard {
                            key: "{song.id}",
                            song: song.clone(),
                            show_album: true
                        }
                    }
                }
            }
            
            // Quick access cards
            section {
                class: "space-y-4",
                h2 { class: "text-2xl font-bold", "Quick Access" }
                div {
                    class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                    
                    QuickAccessCard {
                        title: "Liked Songs",
                        icon: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z",
                        color: "from-purple-600 to-blue-600"
                    }
                    
                    QuickAccessCard {
                        title: "Random Mix",
                        icon: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z",
                        color: "from-green-600 to-teal-600"
                    }
                    
                    QuickAccessCard {
                        title: "Most Played",
                        icon: "M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z",
                        color: "from-orange-600 to-red-600"
                    }
                    
                    QuickAccessCard {
                        title: "New Releases",
                        icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z",
                        color: "from-pink-600 to-rose-600"
                    }
                }
            }
        }
    }
}

/// Quick access card component
#[component]
fn QuickAccessCard(
    title: &'static str,
    icon: &'static str,
    color: &'static str,
) -> Element {
    rsx! {
        div {
            class: "relative rounded-lg overflow-hidden cursor-pointer group",
            div {
                class: "absolute inset-0 bg-gradient-to-br {color} opacity-80 group-hover:opacity-100 transition-opacity"
            }
            div {
                class: "relative p-4 flex items-center gap-4",
                svg {
                    class: "w-10 h-10 text-white",
                    fill: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "{icon}"
                    }
                }
                span { class: "font-bold text-white", "{title}" }
            }
        }
    }
}
