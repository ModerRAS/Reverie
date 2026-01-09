//! 专辑页面 - 所有专辑的网格视图
//!
//! 显示带有各种排序选项的专辑，如最新、最近播放等。

#![allow(unused)]

use crate::api::Album;
use crate::components::{AlbumCard, EmptyState, LoadingSpinner, PageHeader, TabBar};
use crate::mock;
use dioxus::prelude::*;

/// 专辑列表类型（匹配 Navidrome 的专辑视图）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlbumListType {
    RecentlyAdded,
    RecentlyPlayed,
    MostPlayed,
    Random,
    Starred,
    ByArtist,
    ByYear,
    ByGenre,
}

impl AlbumListType {
    fn label(&self) -> &'static str {
        match self {
            Self::RecentlyAdded => "最近添加",
            Self::RecentlyPlayed => "最近播放",
            Self::MostPlayed => "最多播放",
            Self::Random => "随机",
            Self::Starred => "已收藏",
            Self::ByArtist => "按艺术家",
            Self::ByYear => "按年份",
            Self::ByGenre => "按流派",
        }
    }
}

/// 专辑页面组件
#[component]
pub fn AlbumsPage() -> Element {
    let mut list_type = use_signal(|| AlbumListType::RecentlyAdded);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let navigator = use_navigator();

    // Tab 选项
    let tabs = vec![
        "最近添加".to_string(),
        "最近播放".to_string(),
        "最多播放".to_string(),
        "随机".to_string(),
        "已收藏".to_string(),
    ];

    let active_tab = match list_type() {
        AlbumListType::RecentlyAdded => 0,
        AlbumListType::RecentlyPlayed => 1,
        AlbumListType::MostPlayed => 2,
        AlbumListType::Random => 3,
        AlbumListType::Starred => 4,
        _ => 0,
    };

    use_effect(move || {
        loading.set(true);

        albums.set(mock::albums(24));
        loading.set(false);
    });

    let on_tab_change = move |idx: usize| {
        list_type.set(match idx {
            0 => AlbumListType::RecentlyAdded,
            1 => AlbumListType::RecentlyPlayed,
            2 => AlbumListType::MostPlayed,
            3 => AlbumListType::Random,
            4 => AlbumListType::Starred,
            _ => AlbumListType::RecentlyAdded,
        });
    };

    let on_album_click = move |id: String| {
        navigator.push(format!("/album/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "专辑".to_string(),
                subtitle: Some(format!("{} 张专辑", albums.read().len()))
            }

            TabBar {
                tabs: tabs,
                active_index: active_tab,
                on_change: on_tab_change
            }

            if loading() {
                LoadingSpinner { message: "正在加载专辑..." }
            } else if let Some(err) = error() {
                div {
                    class: "text-red-500 text-center py-8",
                    "错误: {err}"
                }
            } else if albums.read().is_empty() {
                EmptyState {
                    title: "未找到专辑".to_string(),
                    message: Some("您的音乐库是空的。添加一些音乐开始使用。".to_string())
                }
            } else {
                div {
                    class: "album-grid",
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
    }
}
