//! 歌单页面 - 所有歌单的网格视图

use crate::api::Playlist;
use crate::components::{EmptyState, LoadingSpinner, PageHeader, PlaylistCard};
use crate::mock;
use dioxus::prelude::*;

/// 歌单页面组件
#[component]
pub fn PlaylistsPage() -> Element {
    let mut playlists = use_signal(Vec::<Playlist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // 加载歌单
    use_effect(move || {
        loading.set(true);

        // 演示数据

        playlists.set(mock::playlists(8));
        loading.set(false);
    });

    let on_playlist_click = move |id: String| {
        navigator.push(format!("/playlist/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "歌单".to_string(),
                subtitle: Some(format!("{} 个歌单", playlists.read().len())),

                // 创建歌单按钮
                button {
                    class: "btn-primary flex items-center gap-2",
                    svg {
                        class: "w-5 h-5",
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
                    "新建歌单"
                }
            }

            if loading() {
                LoadingSpinner { message: "正在加载歌单..." }
            } else if playlists.read().is_empty() {
                EmptyState {
                    title: "还没有歌单".to_string(),
                    message: Some("创建一个歌单来整理您喜欢的音乐。".to_string())
                }
            } else {
                div {
                    class: "album-grid",
                    for playlist in playlists.read().iter() {
                        PlaylistCard {
                            key: "{playlist.id}",
                            playlist: playlist.clone(),
                            on_click: on_playlist_click
                        }
                    }
                }
            }
        }
    }
}
