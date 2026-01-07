//! Card components for albums, artists, playlists
//!
//! Grid-based display cards for music content.

use crate::api::{Album, Artist, Playlist, Song};
use crate::state::{apply_player_action, PlayerAction, PlayerState};
use dioxus::prelude::*;

/// Album card for grid display
#[component]
pub fn AlbumCard(
    album: Album,
    #[props(default)] on_click: Option<EventHandler<String>>,
) -> Element {
    let album_id = album.id.clone();
    let cover_url = album
        .cover_art
        .as_ref()
        .map(|id| format!("/rest/getCoverArt.view?id={}&size=300", id))
        .unwrap_or_default();

    rsx! {
        div {
            class: "album-card group",
            onclick: move |_| {
                if let Some(handler) = &on_click {
                    handler.call(album_id.clone());
                }
            },

            // Cover art
            div {
                class: "relative aspect-square bg-gray-700",
                if !cover_url.is_empty() {
                    img {
                        class: "album-cover",
                        src: "{cover_url}",
                        alt: "{album.name}",
                        loading: "lazy"
                    }
                } else {
                    div {
                        class: "w-full h-full flex items-center justify-center text-gray-500",
                        svg {
                            class: "w-16 h-16",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                            }
                        }
                    }
                }

                // Play button overlay
                div {
                    class: "absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 flex items-center justify-center transition-all",
                    button {
                        class: "w-12 h-12 rounded-full bg-blue-500 text-white opacity-0 group-hover:opacity-100 transform scale-90 group-hover:scale-100 transition-all shadow-lg flex items-center justify-center",
                        svg {
                            class: "w-6 h-6 ml-0.5",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M8 5v14l11-7z"
                            }
                        }
                    }
                }
            }

            // Album info
            div {
                class: "p-3",
                h3 {
                    class: "font-medium text-white truncate",
                    "{album.name}"
                }
                p {
                    class: "text-sm text-gray-400 truncate mt-1",
                    "{album.artist.as_deref().unwrap_or(\"Unknown Artist\")}"
                }
                if let Some(year) = album.year {
                    p {
                        class: "text-xs text-gray-500 mt-1",
                        "{year}"
                    }
                }
            }
        }
    }
}

/// Artist card for grid display
#[component]
pub fn ArtistCard(
    artist: Artist,
    #[props(default)] on_click: Option<EventHandler<String>>,
) -> Element {
    let artist_id = artist.id.clone();
    let cover_url = artist
        .cover_art
        .as_ref()
        .map(|id| format!("/rest/getCoverArt.view?id={}&size=300", id))
        .unwrap_or_default();

    rsx! {
        div {
            class: "card card-hover cursor-pointer group",
            onclick: move |_| {
                if let Some(handler) = &on_click {
                    handler.call(artist_id.clone());
                }
            },

            // Artist image (circular)
            div {
                class: "p-4",
                div {
                    class: "relative aspect-square rounded-full overflow-hidden bg-gray-700",
                    if !cover_url.is_empty() {
                        img {
                            class: "w-full h-full object-cover",
                            src: "{cover_url}",
                            alt: "{artist.name}",
                            loading: "lazy"
                        }
                    } else {
                        div {
                            class: "w-full h-full flex items-center justify-center text-gray-500",
                            svg {
                                class: "w-16 h-16",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
                                }
                            }
                        }
                    }

                    // Play button overlay
                    div {
                        class: "absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 flex items-center justify-center transition-all rounded-full",
                        button {
                            class: "w-12 h-12 rounded-full bg-blue-500 text-white opacity-0 group-hover:opacity-100 transform scale-90 group-hover:scale-100 transition-all shadow-lg flex items-center justify-center",
                            svg {
                                class: "w-6 h-6 ml-0.5",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M8 5v14l11-7z"
                                }
                            }
                        }
                    }
                }
            }

            // Artist info
            div {
                class: "px-4 pb-4 text-center",
                h3 {
                    class: "font-medium text-white truncate",
                    "{artist.name}"
                }
                p {
                    class: "text-sm text-gray-400 mt-1",
                    "{artist.album_count} albums"
                }
            }
        }
    }
}

/// Playlist card for grid display
#[component]
pub fn PlaylistCard(
    playlist: Playlist,
    #[props(default)] on_click: Option<EventHandler<String>>,
) -> Element {
    let playlist_id = playlist.id.clone();
    let cover_url = playlist
        .cover_art
        .as_ref()
        .map(|id| format!("/rest/getCoverArt.view?id={}&size=300", id))
        .unwrap_or_default();
    let duration = super::common::format_duration_long(playlist.duration);

    rsx! {
        div {
            class: "album-card group",
            onclick: move |_| {
                if let Some(handler) = &on_click {
                    handler.call(playlist_id.clone());
                }
            },

            // Cover art
            div {
                class: "relative aspect-square bg-gray-700",
                if !cover_url.is_empty() {
                    img {
                        class: "album-cover",
                        src: "{cover_url}",
                        alt: "{playlist.name}",
                        loading: "lazy"
                    }
                } else {
                    div {
                        class: "w-full h-full flex items-center justify-center text-gray-500 bg-gradient-to-br from-blue-900 to-purple-900",
                        svg {
                            class: "w-16 h-16",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M4 6h16M4 10h16M4 14h16M4 18h16"
                            }
                        }
                    }
                }

                // Play button overlay
                div {
                    class: "absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-40 flex items-center justify-center transition-all",
                    button {
                        class: "w-12 h-12 rounded-full bg-blue-500 text-white opacity-0 group-hover:opacity-100 transform scale-90 group-hover:scale-100 transition-all shadow-lg flex items-center justify-center",
                        svg {
                            class: "w-6 h-6 ml-0.5",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M8 5v14l11-7z"
                            }
                        }
                    }
                }
            }

            // Playlist info
            div {
                class: "p-3",
                h3 {
                    class: "font-medium text-white truncate",
                    "{playlist.name}"
                }
                p {
                    class: "text-sm text-gray-400 truncate mt-1",
                    "{playlist.song_count} songs • {duration}"
                }
            }
        }
    }
}

/// Song card for horizontal display (like recently played)
#[component]
pub fn SongCard(song: Song, #[props(default)] show_album: bool) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let cover_url = song
        .cover_art
        .as_ref()
        .map(|id| format!("/rest/getCoverArt.view?id={}&size=100", id))
        .unwrap_or_default();
    let song_clone = song.clone();

    rsx! {
        div {
            class: "flex items-center gap-3 p-2 rounded-lg hover:bg-gray-800 cursor-pointer group",
            onclick: move |_| {
                apply_player_action(&mut player_state.write(), PlayerAction::PlaySong(song_clone.clone()));
            },

            // Cover art
            div {
                class: "relative w-12 h-12 flex-shrink-0 rounded overflow-hidden bg-gray-700",
                if !cover_url.is_empty() {
                    img {
                        class: "w-full h-full object-cover",
                        src: "{cover_url}",
                        alt: "{song.title}"
                    }
                }

                // Play overlay
                div {
                    class: "absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-50 flex items-center justify-center transition-all",
                    svg {
                        class: "w-6 h-6 text-white opacity-0 group-hover:opacity-100 transition-opacity",
                        fill: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M8 5v14l11-7z"
                        }
                    }
                }
            }

            // Song info
            div {
                class: "min-w-0 flex-1",
                p {
                    class: "text-sm font-medium text-white truncate",
                    "{song.title}"
                }
                p {
                    class: "text-xs text-gray-400 truncate",
                    if show_album {
                        "{song.artist.as_deref().unwrap_or(\"Unknown\")} • {song.album.as_deref().unwrap_or(\"Unknown Album\")}"
                    } else {
                        "{song.artist.as_deref().unwrap_or(\"Unknown Artist\")}"
                    }
                }
            }
        }
    }
}
