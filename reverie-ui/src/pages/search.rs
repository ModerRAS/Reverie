//! 搜索页面 - 搜索结果

use crate::api::{Album, Artist, Song};
use crate::components::{
    AlbumCard, ArtistCard, CompactSongList, EmptyState, LoadingSpinner, PageHeader,
};
use crate::mock;
use crate::state::UiState;
use dioxus::prelude::*;

/// 搜索页面组件
#[component]
pub fn SearchPage() -> Element {
    let ui_state = use_context::<Signal<UiState>>();
    let query = use_memo(move || ui_state.read().search_query.clone());

    let mut songs = use_signal(Vec::<Song>::new);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    // 当查询内容变化时进行搜索
    use_effect(move || {
        let q = query();
        if q.is_empty() {
            songs.set(vec![]);
            albums.set(vec![]);
            artists.set(vec![]);
            return;
        }

        loading.set(true);

        // 演示搜索结果
        let (result_songs, result_albums, result_artists) = mock::search(&q);
        songs.set(result_songs);
        albums.set(result_albums);
        artists.set(result_artists);
        loading.set(false);
    });

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    let on_artist_click = move |id: String| {
        navigator.push(format!("/artist/{}", id));
    };

    let current_query = query();

    if current_query.is_empty() {
        return rsx! {
            EmptyState {
                title: "搜索音乐".to_string(),
                message: Some("输入搜索关键词来查找歌曲、专辑和艺术家。".to_string())
            }
        };
    }

    rsx! {
        div {
            class: "space-y-8",

            PageHeader {
                title: format!("搜索: \"{}\"", current_query),
                subtitle: Some(format!("{} 首歌曲, {} 张专辑, {} 位艺术家",
                    songs.read().len(), albums.read().len(), artists.read().len()))
            }

            if loading() {
                LoadingSpinner { message: "正在搜索..." }
            } else {
                // 艺术家部分
                if !artists.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "艺术家" }
                        div {
                            class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4",
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

                // 专辑部分
                if !albums.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "专辑" }
                        div {
                            class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4",
                            for album in albums.read().iter() {
                                AlbumCard {
                                    key: "{album.id}",
                                    album: album.clone(),
                                    on_click: on_album_click
                                }
                            }
                        }
                    }
                }

                // 歌曲部分
                if !songs.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "歌曲" }
                        CompactSongList { songs: songs.read().clone() }
                    }
                }

                // 无结果
                if songs.read().is_empty() && albums.read().is_empty() && artists.read().is_empty() {
                    EmptyState {
                        title: "未找到结果".to_string(),
                        message: Some(format!("没有找到与 \"{}\" 匹配的内容", current_query))
                    }
                }
            }
        }
    }
}
