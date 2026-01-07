//! Albums page - Grid view of all albums
//!
//! Displays albums with various sorting options like newest, recently played, etc.

#![allow(unused)]

use crate::api::Album;
use crate::components::{AlbumCard, EmptyState, LoadingSpinner, PageHeader, TabBar};
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

    // Simulated album data for demo
    use_effect(move || {
        loading.set(true);

        // In production, this would call the API
        let demo_albums: Vec<Album> = (1..=24)
            .map(|i| Album {
                id: format!("album-{}", i),
                name: format!("Album {}", i),
                artist: Some(format!("Artist {}", (i % 5) + 1)),
                artist_id: Some(format!("artist-{}", (i % 5) + 1)),
                cover_art: None,
                song_count: Some(10 + (i % 5) as i32),
                duration: Some(2400 + (i * 60) as i32),
                year: Some(2020 + (i % 5) as i32),
                genre: Some(
                    ["Rock", "Pop", "Jazz", "Electronic", "Classical"][i as usize % 5].to_string(),
                ),
                created: None,
                starred: None,
                play_count: i as i32 * 10,
            })
            .collect();

        albums.set(demo_albums);
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
