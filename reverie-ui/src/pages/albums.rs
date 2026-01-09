//! Albums page - Grid view of all albums
//!
//! Displays albums with various sorting options like newest, recently played, etc.

#![allow(unused)]

use crate::api::Album;
use crate::components::{AlbumCard, EmptyState, LoadingSpinner, PageHeader, TabBar};
use crate::mock;
use dioxus::prelude::*;

/// Album list type (matching Navidrome's album views)
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
            Self::RecentlyAdded => "Recently Added",
            Self::RecentlyPlayed => "Recently Played",
            Self::MostPlayed => "Most Played",
            Self::Random => "Random",
            Self::Starred => "Starred",
            Self::ByArtist => "By Artist",
            Self::ByYear => "By Year",
            Self::ByGenre => "By Genre",
        }
    }
}

/// Albums page component
#[component]
pub fn AlbumsPage() -> Element {
    let mut list_type = use_signal(|| AlbumListType::RecentlyAdded);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let navigator = use_navigator();

    // Tab options
    let tabs = vec![
        "Recently Added".to_string(),
        "Recently Played".to_string(),
        "Most Played".to_string(),
        "Random".to_string(),
        "Starred".to_string(),
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
                title: "Albums".to_string(),
                subtitle: Some(format!("{} albums", albums.read().len()))
            }

            TabBar {
                tabs: tabs,
                active_index: active_tab,
                on_change: on_tab_change
            }

            if loading() {
                LoadingSpinner { message: "Loading albums..." }
            } else if let Some(err) = error() {
                div {
                    class: "text-red-500 text-center py-8",
                    "Error: {err}"
                }
            } else if albums.read().is_empty() {
                EmptyState {
                    title: "No albums found".to_string(),
                    message: Some("Your music library is empty. Add some music to get started.".to_string())
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
