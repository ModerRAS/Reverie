//! 歌单详情页面

use crate::api::Playlist;
use crate::components::{format_duration_long, LoadingSpinner, TrackList};
use crate::mock;
use crate::state::{apply_player_action, PlayerAction, PlayerState};
use dioxus::prelude::*;

/// 歌单详情页面组件
#[component]
pub fn PlaylistDetailPage(id: String) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let mut playlist = use_signal(|| None::<Playlist>);
    let mut loading = use_signal(|| true);

    // 加载歌单详情
    use_effect(move || {
        loading.set(true);

        playlist.set(Some(mock::playlist_detail(&id)));
        loading.set(false);
    });

    if loading() {
        return rsx! {
            LoadingSpinner { message: "正在加载歌单..." }
        };
    }

    let Some(playlist_data) = playlist.read().clone() else {
        return rsx! {
            div { class: "text-center py-12 text-gray-400", "歌单不存在" }
        };
    };

    let total_duration = format_duration_long(playlist_data.duration);
    let songs = playlist_data.entry.clone();
    let songs_for_play = songs.clone();

    rsx! {
        div {
            class: "space-y-6",

            // 歌单头部
            div {
                class: "flex flex-col md:flex-row gap-6",

                // 封面图片
                div {
                    class: "w-48 h-48 md:w-64 md:h-64 flex-shrink-0 rounded-lg overflow-hidden shadow-xl bg-gradient-to-br from-purple-600 to-blue-600",
                    if let Some(ref cover_id) = playlist_data.cover_art {
                        img {
                            class: "w-full h-full object-cover",
                            src: "/rest/getCoverArt.view?id={cover_id}&size=500",
                            alt: "{playlist_data.name}"
                        }
                    } else {
                        div {
                            class: "w-full h-full flex items-center justify-center text-white/60",
                            svg {
                                class: "w-24 h-24",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M15 6H3v2h12V6zm0 4H3v2h12v-2zM3 16h8v-2H3v2zM17 6v8.18c-.31-.11-.65-.18-1-.18-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3V8h3V6h-5z"
                                }
                            }
                        }
                    }
                }

                // 歌单信息
                div {
                    class: "flex flex-col justify-end",
                    p { class: "text-sm text-gray-400 uppercase tracking-wider", "歌单" }
                    h1 { class: "text-4xl md:text-5xl font-bold mt-2", "{playlist_data.name}" }

                    div {
                        class: "flex items-center gap-2 mt-4 text-gray-300",
                        if let Some(ref owner) = playlist_data.owner {
                            span { class: "font-medium", "{owner}" }
                            span { class: "text-gray-500", "•" }
                        }
                        span { "{playlist_data.song_count} 首歌曲, {total_duration}" }
                    }

                    // 操作按钮
                    div {
                        class: "flex items-center gap-4 mt-6",
                        button {
                            class: "w-14 h-14 rounded-full bg-blue-500 text-white flex items-center justify-center hover:bg-blue-400 transition-colors shadow-lg",
                            onclick: move |_| {
                                apply_player_action(&mut player_state.write(), PlayerAction::PlayPlaylist(songs_for_play.clone()));
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
                            title: "随机播放",
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
                            class: "btn-icon text-gray-400 hover:text-white",
                            title: "编辑歌单",
                            svg {
                                class: "w-6 h-6",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z"
                                }
                            }
                        }

                        button {
                            class: "btn-icon text-gray-400 hover:text-white",
                            title: "更多选项",
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

            // 歌曲列表
            TrackList {
                tracks: songs,
                show_number: true,
                show_album: true,
                show_artist: true
            }
        }
    }
}
