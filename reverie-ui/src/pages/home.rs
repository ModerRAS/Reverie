//! 首页 - 带有最近播放等内容的仪表盘

use crate::api::{Album, Song};
use crate::components::{AlbumCard, LoadingSpinner, SongCard};
use crate::mock;
use dioxus::prelude::*;

/// 首页组件
#[component]
pub fn HomePage() -> Element {
    let mut recent_albums = use_signal(Vec::<Album>::new);
    let mut recent_songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // 加载首页数据
    use_effect(move || {
        loading.set(true);

        let (home_albums, home_songs) = mock::home();
        recent_albums.set(home_albums);
        recent_songs.set(home_songs);

        loading.set(false);
    });

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    if loading() {
        return rsx! {
            LoadingSpinner { message: "正在加载..." }
        };
    }

    rsx! {
        div {
            class: "space-y-8",

            // 欢迎头部
            section {
                class: "mb-8",
                h1 { class: "text-3xl font-bold", "晚上好" }
                p { class: "text-gray-400 mt-1", "这是您最近收听的内容" }
            }

            // 最近播放的专辑
            section {
                class: "space-y-4",
                div {
                    class: "flex items-center justify-between",
                    h2 { class: "text-2xl font-bold", "最近播放" }
                    button {
                        class: "text-sm text-gray-400 hover:text-white",
                        "显示全部"
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

            // 最近添加部分
            section {
                class: "space-y-4",
                div {
                    class: "flex items-center justify-between",
                    h2 { class: "text-2xl font-bold", "最近添加" }
                    button {
                        class: "text-sm text-gray-400 hover:text-white",
                        "显示全部"
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

            // 快速访问卡片
            section {
                class: "space-y-4",
                h2 { class: "text-2xl font-bold", "快速访问" }
                div {
                    class: "grid grid-cols-2 md:grid-cols-4 gap-4",

                    QuickAccessCard {
                        title: "喜欢的歌曲",
                        icon: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z",
                        color: "from-purple-600 to-blue-600"
                    }

                    QuickAccessCard {
                        title: "随机混合",
                        icon: "M10.59 9.17L5.41 4 4 5.41l5.17 5.17 1.42-1.41zM14.5 4l2.04 2.04L4 18.59 5.41 20 17.96 7.46 20 9.5V4h-5.5zm.33 9.41l-1.41 1.41 3.13 3.13L14.5 20H20v-5.5l-2.04 2.04-3.13-3.13z",
                        color: "from-green-600 to-teal-600"
                    }

                    QuickAccessCard {
                        title: "最多播放",
                        icon: "M16 6l2.29 2.29-4.88 4.88-4-4L2 16.59 3.41 18l6-6 4 4 6.3-6.29L22 12V6z",
                        color: "from-orange-600 to-red-600"
                    }

                    QuickAccessCard {
                        title: "新发布",
                        icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z",
                        color: "from-pink-600 to-rose-600"
                    }
                }
            }
        }
    }
}

/// 快速访问卡片组件
#[component]
fn QuickAccessCard(title: &'static str, icon: &'static str, color: &'static str) -> Element {
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
