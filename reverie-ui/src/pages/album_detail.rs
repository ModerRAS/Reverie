//! 专辑详情页

use crate::api::{Album, Song};
use crate::components::{format_duration_long, LoadingSpinner, TrackList};
use crate::mock;
use crate::state::{apply_player_action, PlayerAction, PlayerState};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AlbumDetailProps {
    pub id: String,
}

/// 专辑详情页组件
#[component]
pub fn AlbumDetailPage(id: String) -> Element {
    let mut player_state = use_context::<Signal<PlayerState>>();
    let mut album = use_signal(|| None::<Album>);
    let mut tracks = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);

    // 加载专辑详情
    use_effect(move || {
        loading.set(true);

        // 演示数据
        let (mock_album, mock_tracks) = mock::album_detail(&id);
        album.set(Some(mock_album));
        tracks.set(mock_tracks);
        loading.set(false);
    });

    if loading() {
        return rsx! {
            LoadingSpinner { message: "正在加载专辑..." }
        };
    }

    let Some(album_data) = album.read().clone() else {
        return rsx! {
            div { class: "text-center py-12 text-gray-400", "专辑不存在" }
        };
    };

    let total_duration = format_duration_long(album_data.duration.unwrap_or(0));
    let track_list = tracks.read().clone();
    let track_list_for_play = track_list.clone();

    rsx! {
        div {
            class: "space-y-6",

            // 专辑头部
            div {
                class: "flex flex-col md:flex-row gap-6",

                // 封面图片
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

                // 专辑信息
                div {
                    class: "flex flex-col justify-end",
                    p { class: "text-sm text-gray-400 uppercase tracking-wider", "专辑" }
                    h1 { class: "text-4xl md:text-5xl font-bold mt-2", "{album_data.name}" }

                    div {
                        class: "flex items-center gap-2 mt-4 text-gray-300",
                        span { class: "font-medium", "{album_data.artist.as_deref().unwrap_or(\"未知艺术家\")}" }
                        span { class: "text-gray-500", "•" }
                        if let Some(year) = album_data.year {
                            span { "{year}" }
                            span { class: "text-gray-500", "•" }
                        }
                        span { "{album_data.song_count.unwrap_or(0)} 首歌曲, {total_duration}" }
                    }

                    // 操作按钮
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
                            class: "btn-icon text-gray-400 hover:text-red-500",
                            title: "添加到收藏",
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

            // 曲目列表
            TrackList {
                tracks: track_list,
                show_number: true,
                show_album: false,
                show_artist: false
            }
        }
    }
}
