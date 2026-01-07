//! List components for songs, tracks, etc.
//!
//! Table and list views for music content.

use dioxus::prelude::*;
use crate::api::Song;
use crate::state::{PlayerState, PlayerAction, apply_player_action};
use super::common::format_duration;
use super::player::NowPlayingBars;

/// Track list table for album/playlist views
#[component]
pub fn TrackList(
    tracks: Vec<Song>,
    #[props(default = true)] show_number: bool,
    #[props(default = false)] show_album: bool,
    #[props(default = false)] show_artist: bool,
) -> Element {
    let player_state = use_context::<Signal<PlayerState>>();
    let current_song_id = player_state.read().current_song.as_ref().map(|s| s.id.clone());
    let is_playing = player_state.read().is_playing;

    rsx! {
        div {
            class: "track-list rounded-lg overflow-hidden",
            
            // Header row
            div {
                class: "flex items-center gap-4 px-4 py-2 text-xs text-gray-400 uppercase tracking-wider border-b border-gray-700",
                if show_number {
                    div { class: "w-10 text-center", "#" }
                }
                div { class: "flex-1", "Title" }
                if show_artist {
                    div { class: "w-48 hidden md:block", "Artist" }
                }
                if show_album {
                    div { class: "w-48 hidden lg:block", "Album" }
                }
                div { class: "w-12 text-right", "Duration" }
                div { class: "w-20", "" } // Actions column
            }
            
            // Track rows
            for (idx, track) in tracks.iter().enumerate() {
                {
                    let track_id = track.id.clone();
                    let is_current = current_song_id.as_ref() == Some(&track_id);
                    let row_class = if is_current {
                        "track-item playing"
                    } else {
                        "track-item"
                    };
                    
                    rsx! {
                        TrackRow {
                            key: "{track.id}",
                            track: track.clone(),
                            index: idx + 1,
                            show_number: show_number,
                            show_album: show_album,
                            show_artist: show_artist,
                            is_current: is_current,
                            is_playing: is_playing && is_current,
                            row_class: row_class.to_string()
                        }
                    }
                }
            }
        }
    }
}

/// Single track row
#[component]
fn TrackRow(
    track: Song,
    index: usize,
    show_number: bool,
    show_album: bool,
    show_artist: bool,
    is_current: bool,
    is_playing: bool,
    row_class: String,
) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let track_clone = track.clone();
    let track_for_queue = track.clone();
    let duration = track.duration.map(format_duration).unwrap_or_default();
    let cover_url = track.cover_art.as_ref()
        .map(|id| format!("/rest/getCoverArt.view?id={}&size=50", id));

    rsx! {
        div {
            class: "{row_class}",
            ondblclick: move |_| {
                apply_player_action(&mut player_state.write(), PlayerAction::PlaySong(track_clone.clone()));
            },
            
            // Track number or playing indicator
            if show_number {
                div {
                    class: "w-10 text-center text-gray-400",
                    if is_playing {
                        NowPlayingBars {}
                    } else if is_current {
                        svg {
                            class: "w-4 h-4 mx-auto text-blue-500",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M8 5v14l11-7z"
                            }
                        }
                    } else {
                        span { "{index}" }
                    }
                }
            }
            
            // Title with cover art
            div {
                class: "flex-1 flex items-center gap-3 min-w-0",
                
                // Cover art (small)
                if let Some(ref url) = cover_url {
                    img {
                        class: "w-10 h-10 rounded object-cover flex-shrink-0",
                        src: "{url}",
                        alt: ""
                    }
                } else {
                    div {
                        class: "w-10 h-10 rounded bg-gray-700 flex items-center justify-center flex-shrink-0",
                        svg {
                            class: "w-5 h-5 text-gray-500",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                            }
                        }
                    }
                }
                
                // Title text
                div {
                    class: "min-w-0",
                    p {
                        class: if is_current { "text-blue-400 truncate" } else { "text-white truncate" },
                        "{track.title}"
                    }
                    // Show artist inline on mobile if not showing artist column
                    if !show_artist {
                        p {
                            class: "text-xs text-gray-400 truncate md:hidden",
                            "{track.artist.as_deref().unwrap_or(\"Unknown Artist\")}"
                        }
                    }
                }
            }
            
            // Artist column
            if show_artist {
                div {
                    class: "w-48 text-gray-400 truncate hidden md:block",
                    "{track.artist.as_deref().unwrap_or(\"Unknown Artist\")}"
                }
            }
            
            // Album column
            if show_album {
                div {
                    class: "w-48 text-gray-400 truncate hidden lg:block",
                    "{track.album.as_deref().unwrap_or(\"\")}"
                }
            }
            
            // Duration
            div {
                class: "w-12 text-gray-400 text-right text-sm",
                "{duration}"
            }
            
            // Action buttons
            div {
                class: "w-20 flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity",
                
                // Add to queue
                button {
                    class: "btn-icon text-gray-400 hover:text-white",
                    title: "Add to queue",
                    onclick: move |e| {
                        e.stop_propagation();
                        apply_player_action(&mut player_state.write(), PlayerAction::AddToQueue(track_for_queue.clone()));
                    },
                    svg {
                        class: "w-4 h-4",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M12 4v16m8-8H4"
                        }
                    }
                }
                
                // More options
                button {
                    class: "btn-icon text-gray-400 hover:text-white",
                    svg {
                        class: "w-4 h-4",
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
}

/// Compact song list (for search results, etc.)
#[component]
pub fn CompactSongList(songs: Vec<Song>) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();

    rsx! {
        div {
            class: "space-y-1",
            for song in songs {
                {
                    let song_clone = song.clone();
                    let cover_url = song.cover_art.as_ref()
                        .map(|id| format!("/rest/getCoverArt.view?id={}&size=50", id));
                    let duration = song.duration.map(format_duration).unwrap_or_default();
                    
                    rsx! {
                        div {
                            key: "{song.id}",
                            class: "flex items-center gap-3 p-2 rounded hover:bg-gray-800 cursor-pointer group",
                            ondblclick: move |_| {
                                apply_player_action(&mut player_state.write(), PlayerAction::PlaySong(song_clone.clone()));
                            },
                            
                            // Cover
                            if let Some(url) = cover_url {
                                img {
                                    class: "w-10 h-10 rounded object-cover",
                                    src: "{url}",
                                    alt: ""
                                }
                            } else {
                                div {
                                    class: "w-10 h-10 rounded bg-gray-700 flex items-center justify-center",
                                    svg {
                                        class: "w-5 h-5 text-gray-500",
                                        fill: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                                        }
                                    }
                                }
                            }
                            
                            // Info
                            div {
                                class: "flex-1 min-w-0",
                                p { class: "text-sm text-white truncate", "{song.title}" }
                                p { class: "text-xs text-gray-400 truncate", "{song.artist.as_deref().unwrap_or(\"Unknown\")}" }
                            }
                            
                            // Duration
                            span { class: "text-sm text-gray-500", "{duration}" }
                        }
                    }
                }
            }
        }
    }
}
