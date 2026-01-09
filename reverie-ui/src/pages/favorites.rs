//! 收藏页面 - 收藏的内容

use crate::api::{Album, Artist, Song};
use crate::components::{
    AlbumCard, ArtistCard, EmptyState, LoadingSpinner, PageHeader, TabBar, TrackList,
};
use crate::mock;
use dioxus::prelude::*;

/// 收藏页面组件
#[component]
pub fn FavoritesPage() -> Element {
    let mut active_tab = use_signal(|| 0usize);
    let mut starred_songs = use_signal(Vec::<Song>::new);
    let mut starred_albums = use_signal(Vec::<Album>::new);
    let mut starred_artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    let tabs = vec![
        "歌曲".to_string(),
        "专辑".to_string(),
        "艺术家".to_string(),
    ];

    // 加载收藏内容
    use_effect(move || {
        loading.set(true);

        let (songs, albums, artists) = mock::favorites();
        starred_songs.set(songs);
        starred_albums.set(albums);
        starred_artists.set(artists);
        loading.set(false);
    });

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    let on_artist_click = move |id: String| {
        navigator.push(format!("/artist/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "收藏".to_string(),
                subtitle: Some("您收藏的音乐".to_string())
            }

            TabBar {
                tabs: tabs,
                active_index: active_tab(),
                on_change: move |idx| active_tab.set(idx)
            }

            if loading() {
                LoadingSpinner { message: "正在加载收藏..." }
            } else {
                match active_tab() {
                    0 => rsx! {
                        if starred_songs.read().is_empty() {
                            EmptyState {
                                title: "还没有收藏的歌曲".to_string(),
                                message: Some("收藏您喜欢的歌曲，它们会显示在这里。".to_string())
                            }
                        } else {
                            TrackList {
                                tracks: starred_songs.read().clone(),
                                show_number: true,
                                show_album: true,
                                show_artist: true
                            }
                        }
                    },
                    1 => rsx! {
                        if starred_albums.read().is_empty() {
                            EmptyState {
                                title: "还没有收藏的专辑".to_string(),
                                message: Some("收藏您喜欢的专辑，它们会显示在这里。".to_string())
                            }
                        } else {
                            div {
                                class: "album-grid",
                                for album in starred_albums.read().iter() {
                                    AlbumCard {
                                        key: "{album.id}",
                                        album: album.clone(),
                                        on_click: on_album_click
                                    }
                                }
                            }
                        }
                    },
                    2 => rsx! {
                        if starred_artists.read().is_empty() {
                            EmptyState {
                                title: "还没有收藏的艺术家".to_string(),
                                message: Some("收藏您喜欢的艺术家，他们的名字会显示在这里。".to_string())
                            }
                        } else {
                            div {
                                class: "grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4",
                                for artist in starred_artists.read().iter() {
                                    ArtistCard {
                                        key: "{artist.id}",
                                        artist: artist.clone(),
                                        on_click: on_artist_click
                                    }
                                }
                            }
                        }
                    },
                    _ => rsx! {}
                }
            }
        }
    }
}
