//! Album detail page

use dioxus::prelude::*;
use crate::api::{Album, Song};
use crate::state::{PlayerState, PlayerAction, apply_player_action};
use crate::components::{TrackList, LoadingSpinner, format_duration_long};

#[derive(Props, Clone, PartialEq)]
pub struct AlbumDetailProps {
    pub id: String,
}

/// Album detail page component
#[component]
pub fn AlbumDetailPage(id: String) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let mut album = use_signal(|| None::<Album>);
    let mut tracks = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);

    // Load album details
    use_effect(move || {
        loading.set(true);
        
        // Demo data
        let demo_album = Album {
            id: id.clone(),
            name: format!("Album {}", id.split('-').last().unwrap_or("1")),
            artist: Some("Demo Artist".to_string()),
            artist_id: Some("artist-1".to_string()),
            cover_art: None,
            song_count: Some(12),
            duration: Some(3600),
            year: Some(2023),
            genre: Some("Rock".to_string()),
            created: Some("2023-01-01T00:00:00Z".to_string()),
            starred: None,
            play_count: 100,
        };
        
        let demo_tracks: Vec<Song> = (1..=12).map(|i| Song {
            id: format!("{}-track-{}", id, i),
            title: format!("Track {}", i),
            album: Some(demo_album.name.clone()),
            album_id: Some(id.clone()),
            artist: demo_album.artist.clone(),
            artist_id: demo_album.artist_id.clone(),
            track: Some(i as i32),
            year: demo_album.year,
            genre: demo_album.genre.clone(),
            cover_art: None,
            duration: Some(200 + (i * 10) as i32),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: None,
            path: None,
            starred: None,
            play_count: i as i32 * 10,
        }).collect();
        
        album.set(Some(demo_album));
        tracks.set(demo_tracks);
        loading.set(false);
    });

    if loading() {
        return rsx! {
            LoadingSpinner { message: "Loading album..." }
        };
    }

    let Some(album_data) = album.read().clone() else {
        return rsx! {
            div { class: "text-center py-12 text-gray-400", "Album not found" }
        };
    };

    let total_duration = format_duration_long(album_data.duration.unwrap_or(0));
    let track_list = tracks.read().clone();
    let track_list_for_play = track_list.clone();

    rsx! {
        div {
            class: "space-y-6",
            
            // Album header
            div {
                class: "flex flex-col md:flex-row gap-6",
                
                // Cover art
                div {
                    class: "w-48 h-48 md:w-64 md:h-64 flex-shrink-0 rounded-lg overflow-hidden bg-gray-800 shadow-xl",
                    if let Some(ref cover_id) = album_data.cover_art {
                        img {
                            class: "w-full h-full object-cover",
                            src: "/rest/getCoverArt.view?id={cover_id}&size=500",
                            alt: "{album_data.name}"
                        }
                    } else {
                        div {
                            class: "w-full h-full flex items-center justify-center text-gray-600",
                            svg {
                                class: "w-24 h-24",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                                }
                            }
                        }
                    }
                }
                
                // Album info
                div {
                    class: "flex flex-col justify-end",
                    p { class: "text-sm text-gray-400 uppercase tracking-wider", "Album" }
                    h1 { class: "text-4xl md:text-5xl font-bold mt-2", "{album_data.name}" }
                    
                    div {
                        class: "flex items-center gap-2 mt-4 text-gray-300",
                        span { class: "font-medium", "{album_data.artist.as_deref().unwrap_or(\"Unknown Artist\")}" }
                        span { class: "text-gray-500", "•" }
                        if let Some(year) = album_data.year {
                            span { "{year}" }
                            span { class: "text-gray-500", "•" }
                        }
                        span { "{album_data.song_count.unwrap_or(0)} songs, {total_duration}" }
                    }
                    
                    // Action buttons
                    div {
                        class: "flex items-center gap-4 mt-6",
                        button {
                            class: "w-14 h-14 rounded-full bg-blue-500 text-white flex items-center justify-center hover:bg-blue-400 transition-colors shadow-lg",
                            onclick: move |_| {
                                apply_player_action(&mut player_state.write(), PlayerAction::PlayAlbum(track_list_for_play.clone()));
                            },
                            svg {
                                class: "w-7 h-7 ml-1",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M8 5v14l11-7z"
                                }
                            }
                        }
                        
                        button {
                            class: "btn-icon text-gray-400 hover:text-white",
                            title: "Shuffle",
                            svg {
                                class: "w-6 h-6",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"
                                }
                            }
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
                        
                        button {
                            class: "btn-icon text-gray-400 hover:text-white",
                            title: "More options",
                            svg {
                                class: "w-6 h-6",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"
                                }
                            }
                        }
                    }
                }
            }
            
            // Track list
            TrackList {
                tracks: track_list,
                show_number: true,
                show_album: false,
                show_artist: false
            }
        }
    }
}
