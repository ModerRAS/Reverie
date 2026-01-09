//! 艺术家页面 - 所有艺术家的网格视图

use crate::api::Artist;
use crate::components::{ArtistCard, EmptyState, LoadingSpinner, PageHeader};
use crate::mock;
use dioxus::prelude::*;

/// 艺术家页面组件
#[component]
pub fn ArtistsPage() -> Element {
    let mut artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // 加载艺术家
    use_effect(move || {
        loading.set(true);

        // 演示数据
        artists.set(mock::artists(20));
        loading.set(false);
    });

    let on_artist_click = move |id: String| {
        navigator.push(format!("/artist/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "艺术家".to_string(),
                subtitle: Some(format!("{} 位艺术家", artists.read().len()))
            }

            if loading() {
                LoadingSpinner { message: "正在加载艺术家..." }
            } else if artists.read().is_empty() {
                EmptyState {
                    title: "未找到艺术家".to_string(),
                    message: Some("您的音乐库是空的。".to_string())
                }
            } else {
                div {
                    class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4",
                    for artist in artists.read().iter() {
                        ArtistCard {
                            key: "{artist.id}",
                            artist: artist.clone(),
                            on_click: on_artist_click
                        }
                    }
                }
            }
        }
    }
}
