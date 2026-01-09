//! 歌曲页面 - 所有歌曲的表格视图

use crate::api::Song;
use crate::components::{EmptyState, LoadingSpinner, PageHeader, TrackList};
use crate::mock;
use dioxus::prelude::*;

/// 歌曲页面组件
#[component]
pub fn SongsPage() -> Element {
    let mut songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);

    // 加载歌曲
    use_effect(move || {
        loading.set(true);

        songs.set(mock::songs(50));
        loading.set(false);
    });

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "歌曲".to_string(),
                subtitle: Some(format!("{} 首歌曲", songs.read().len()))
            }

            if loading() {
                LoadingSpinner { message: "正在加载歌曲..." }
            } else if songs.read().is_empty() {
                EmptyState {
                    title: "没有找到歌曲".to_string(),
                    message: Some("您的音乐库是空的。".to_string())
                }
            } else {
                TrackList {
                    tracks: songs.read().clone(),
                    show_number: true,
                    show_album: true,
                    show_artist: true
                }
            }
        }
    }
}
