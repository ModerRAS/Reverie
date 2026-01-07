//! Artist detail page

use crate::api::{Album, Artist, Song};
use crate::components::{AlbumCard, CompactSongList, LoadingSpinner};
use dioxus::prelude::*;

/// Artist detail page component
#[component]
pub fn ArtistDetailPage(id: String) -> Element {
    let mut artist = use_signal(|| None::<Artist>);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut top_songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // Load artist details
    use_effect(move || {
        loading.set(true);

        // Demo data
        let demo_artist = Artist {
            id: id.clone(),
            name: format!("Artist {}", id.split('-').next_back().unwrap_or("1")),
            album_count: 5,
            cover_art: None,
            artist_image_url: None,
            starred: None,
        };

        let demo_albums: Vec<Album> = (1..=5)
            .map(|i| Album {
                id: format!("{}-album-{}", id, i),
                name: format!("Album {}", i),
                artist: Some(demo_artist.name.clone()),
                artist_id: Some(id.clone()),
                cover_art: None,
                song_count: Some(10),
                duration: Some(2400),
                year: Some(2020 + i),
                genre: Some("Rock".to_string()),
                created: None,
                starred: None,
                play_count: i * 50,
            })
            .collect();

        let demo_songs: Vec<Song> = (1..=5)
            .map(|i| Song {
                id: format!("{}-topsong-{}", id, i),
                title: format!("Top Song {}", i),
                album: Some(format!("Album {}", i)),
                album_id: Some(format!("{}-album-{}", id, i)),
                artist: Some(demo_artist.name.clone()),
                artist_id: Some(id.clone()),
                track: Some(1),
                year: Some(2023),
                genre: None,
                cover_art: None,
                duration: Some(220),
                bit_rate: Some(320),
                suffix: Some("mp3".to_string()),
                content_type: None,
                path: None,
                starred: None,
                play_count: (6 - i) * 100,
            })
            .collect();

        artist.set(Some(demo_artist));
        albums.set(demo_albums);
        top_songs.set(demo_songs);
        loading.set(false);
    });

    if loading() {
        return rsx! {
            LoadingSpinner { message: "Loading artist..." }
        };
    }

    let Some(artist_data) = artist.read().clone() else {
        return rsx! {
            div { class: "text-center py-12 text-gray-400", "Artist not found" }
        };
    };

    let on_album_click = move |album_id: String| {
        navigator.push(format!("/album/{}", album_id));
    };

    rsx! {
        div {
            class: "space-y-8",

            // Artist header
            div {
                class: "relative",

                // Background gradient
                div {
                    class: "absolute inset-0 h-80 bg-gradient-to-b from-blue-900/50 to-transparent -z-10"
                }

                div {
                    class: "flex flex-col md:flex-row items-center md:items-end gap-6 pt-12",

                    // Artist image
                    div {
                        class: "w-48 h-48 rounded-full overflow-hidden bg-gray-800 shadow-2xl",
                        if let Some(ref cover_id) = artist_data.cover_art {
                            img {
                                class: "w-full h-full object-cover",
                                src: "/rest/getCoverArt.view?id={cover_id}&size=500",
                                alt: "{artist_data.name}"
                            }
                        } else {
                            div {
                                class: "w-full h-full flex items-center justify-center text-gray-600",
                                svg {
                                    class: "w-24 h-24",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
                                    }
                                }
                            }
                        }
                    }

                    // Artist info
                    div {
                        class: "text-center md:text-left",
                        p { class: "text-sm text-gray-400 uppercase tracking-wider", "Artist" }
                        h1 { class: "text-5xl md:text-6xl font-bold mt-2", "{artist_data.name}" }
                        p { class: "text-gray-400 mt-2", "{artist_data.album_count} albums" }

                        // Action buttons
                        div {
                            class: "flex items-center justify-center md:justify-start gap-4 mt-6",
                            button {
                                class: "btn-primary flex items-center gap-2",
                                svg {
                                    class: "w-5 h-5",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M8 5v14l11-7z"
                                    }
                                }
                                "Play All"
                            }
                            button {
                                class: "btn-secondary flex items-center gap-2",
                                svg {
                                    class: "w-5 h-5",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"
                                    }
                                }
                                "Shuffle"
                            }
                            button {
                                class: "btn-icon text-gray-400 hover:text-red-500",
                                title: "Add to favorites",
                                svg {
                                    class: "w-6 h-6",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Top songs
            section {
                class: "space-y-4",
                h2 { class: "text-2xl font-bold", "Top Songs" }
                CompactSongList { songs: top_songs.read().clone() }
            }

            // Albums
            section {
                class: "space-y-4",
                h2 { class: "text-2xl font-bold", "Albums" }
                div {
                    class: "album-grid",
                    for album in albums.read().iter() {
                        AlbumCard {
                            key: "{album.id}",
                            album: album.clone(),
                            on_click: on_album_click
                        }
                    }
                }
            }
        }
    }
}
