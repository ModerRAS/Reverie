//! Player components - Audio player bar and controls
//!
//! These components provide the music playback interface.

use dioxus::prelude::*;
use crate::state::{PlayerState, RepeatMode, PlayerAction, apply_player_action};
use crate::api::Song;

/// Bottom player bar with playback controls
#[component]
pub fn PlayerBar() -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let player = player_state.read();
    
    let has_song = player.current_song.is_some();
    let is_playing = player.is_playing;
    
    // Don't show player bar if no song in queue
    if !has_song && player.queue.is_empty() {
        return rsx! {};
    }

    let current_song = player.current_song.clone();
    let progress_percent = if player.duration > 0.0 {
        (player.progress / player.duration * 100.0) as i32
    } else {
        0
    };

    rsx! {
        div {
            class: "player-bar",
            
            // Song info (left section)
            div {
                class: "flex items-center gap-3 w-64",
                if let Some(ref song) = current_song {
                    // Cover art
                    div {
                        class: "w-14 h-14 bg-gray-700 rounded flex-shrink-0 overflow-hidden",
                        if song.cover_art.is_some() {
                            img {
                                class: "w-full h-full object-cover",
                                src: "/rest/getCoverArt.view?id={song.cover_art.as_ref().unwrap()}&size=100",
                                alt: "Cover"
                            }
                        } else {
                            div {
                                class: "w-full h-full flex items-center justify-center text-gray-500",
                                svg {
                                    class: "w-8 h-8",
                                    fill: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                                    }
                                }
                            }
                        }
                    }
                    // Song details
                    div {
                        class: "min-w-0",
                        p {
                            class: "text-sm font-medium truncate",
                            "{song.title}"
                        }
                        p {
                            class: "text-xs text-gray-400 truncate",
                            "{song.artist.as_deref().unwrap_or(\"Unknown Artist\")}"
                        }
                    }
                    // Like button
                    button {
                        class: "btn-icon text-gray-400 hover:text-red-500",
                        svg {
                            class: "w-5 h-5",
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
            
            // Playback controls (center section)
            div {
                class: "flex-1 flex flex-col items-center max-w-2xl mx-4",
                
                // Control buttons
                div {
                    class: "flex items-center gap-4 mb-2",
                    
                    // Shuffle button
                    ShuffleButton {}
                    
                    // Previous button
                    button {
                        class: "btn-icon text-gray-300 hover:text-white",
                        onclick: move |_| {
                            apply_player_action(&mut player_state.write(), PlayerAction::Previous);
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M6 6h2v12H6zm3.5 6l8.5 6V6z"
                            }
                        }
                    }
                    
                    // Play/Pause button
                    button {
                        class: "w-10 h-10 rounded-full bg-white text-gray-900 flex items-center justify-center hover:scale-105 transition-transform",
                        onclick: move |_| {
                            if is_playing {
                                apply_player_action(&mut player_state.write(), PlayerAction::Pause);
                            } else {
                                apply_player_action(&mut player_state.write(), PlayerAction::Play);
                            }
                        },
                        if is_playing {
                            svg {
                                class: "w-5 h-5",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M6 4h4v16H6V4zm8 0h4v16h-4V4z"
                                }
                            }
                        } else {
                            svg {
                                class: "w-5 h-5 ml-0.5",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M8 5v14l11-7z"
                                }
                            }
                        }
                    }
                    
                    // Next button
                    button {
                        class: "btn-icon text-gray-300 hover:text-white",
                        onclick: move |_| {
                            apply_player_action(&mut player_state.write(), PlayerAction::Next);
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"
                            }
                        }
                    }
                    
                    // Repeat button
                    RepeatButton {}
                }
                
                // Progress bar
                ProgressBar {
                    progress: progress_percent,
                    current_time: player.progress,
                    total_time: player.duration
                }
            }
            
            // Volume and other controls (right section)
            div {
                class: "flex items-center gap-4 w-64 justify-end",
                
                // Queue button
                button {
                    class: "btn-icon text-gray-400 hover:text-white",
                    svg {
                        class: "w-5 h-5",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M4 6h16M4 10h16M4 14h16M4 18h16"
                        }
                    }
                }
                
                // Volume control
                VolumeControl {}
            }
        }
    }
}

/// Shuffle button component
#[component]
fn ShuffleButton() -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let shuffle = player_state.read().shuffle;
    
    let class = if shuffle {
        "btn-icon text-blue-500"
    } else {
        "btn-icon text-gray-400 hover:text-white"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| {
                apply_player_action(&mut player_state.write(), PlayerAction::ToggleShuffle);
            },
            svg {
                class: "w-5 h-5",
                fill: "currentColor",
                view_box: "0 0 24 24",
                path {
                    d: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z"
                }
            }
        }
    }
}

/// Repeat button component
#[component]
fn RepeatButton() -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let repeat = player_state.read().repeat;
    
    let (class, icon) = match repeat {
        RepeatMode::Off => ("btn-icon text-gray-400 hover:text-white", false),
        RepeatMode::All => ("btn-icon text-blue-500", false),
        RepeatMode::One => ("btn-icon text-blue-500", true),
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| {
                apply_player_action(&mut player_state.write(), PlayerAction::ToggleRepeat);
            },
            svg {
                class: "w-5 h-5",
                fill: "currentColor",
                view_box: "0 0 24 24",
                if icon {
                    path {
                        d: "M7 7h10v3l4-4-4-4v3H5v6h2V7zm10 10H7v-3l-4 4 4 4v-3h12v-6h-2v4zm-4-2V9h-1l-2 1v1h1.5v4H13z"
                    }
                } else {
                    path {
                        d: "M7 7h10v3l4-4-4-4v3H5v6h2V7zm10 10H7v-3l-4 4 4 4v-3h12v-6h-2v4z"
                    }
                }
            }
        }
    }
}

/// Progress bar component
#[component]
fn ProgressBar(progress: i32, current_time: f32, total_time: f32) -> Element {
    let format_time = |seconds: f32| -> String {
        let mins = (seconds / 60.0) as i32;
        let secs = (seconds % 60.0) as i32;
        format!("{:02}:{:02}", mins, secs)
    };

    rsx! {
        div {
            class: "w-full flex items-center gap-2",
            span {
                class: "text-xs text-gray-400 w-10 text-right",
                "{format_time(current_time)}"
            }
            div {
                class: "progress-bar flex-1",
                div {
                    class: "progress-fill",
                    style: "width: {progress}%"
                }
            }
            span {
                class: "text-xs text-gray-400 w-10",
                "{format_time(total_time)}"
            }
        }
    }
}

/// Volume control component
#[component]
fn VolumeControl() -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let volume = player_state.read().volume;
    let volume_percent = (volume * 100.0) as i32;
    
    let volume_icon = if volume == 0.0 {
        "M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"
    } else if volume < 0.5 {
        "M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z"
    } else {
        "M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"
    };

    rsx! {
        div {
            class: "flex items-center gap-2",
            button {
                class: "btn-icon text-gray-400 hover:text-white",
                onclick: move |_| {
                    let new_vol = if volume > 0.0 { 0.0 } else { 0.8 };
                    player_state.write().volume = new_vol;
                },
                svg {
                    class: "w-5 h-5",
                    fill: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "{volume_icon}"
                    }
                }
            }
            input {
                class: "volume-slider",
                r#type: "range",
                min: "0",
                max: "100",
                value: "{volume_percent}",
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<f32>() {
                        player_state.write().volume = val / 100.0;
                    }
                }
            }
        }
    }
}

/// Now playing bars animation
#[component]
pub fn NowPlayingBars() -> Element {
    rsx! {
        div {
            class: "now-playing-bars",
            div { class: "now-playing-bar", style: "height: 60%; animation-delay: 0ms;" }
            div { class: "now-playing-bar", style: "height: 100%; animation-delay: 150ms;" }
            div { class: "now-playing-bar", style: "height: 40%; animation-delay: 300ms;" }
            div { class: "now-playing-bar", style: "height: 80%; animation-delay: 450ms;" }
        }
    }
}
