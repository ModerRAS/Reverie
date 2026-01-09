//! Favorites page - Starred content

use crate::api::{Album, Artist, Song};
use crate::components::{
    AlbumCard, ArtistCard, EmptyState, LoadingSpinner, PageHeader, TabBar, TrackList,
};
use crate::mock;
use dioxus::prelude::*;

/// Favorites page component
#[component]
pub fn FavoritesPage() -> Element {
    let mut active_tab = use_signal(|| 0usize);
    let mut starred_songs = use_signal(Vec::<Song>::new);
    let mut starred_albums = use_signal(Vec::<Album>::new);
    let mut starred_artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    let tabs = vec![
        "Songs".to_string(),
        "Albums".to_string(),
        "Artists".to_string(),
    ];

    // Load starred content
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
                title: "Favorites".to_string(),
                subtitle: Some("Your starred music".to_string())
            }

            TabBar {
                tabs: tabs,
                active_index: active_tab(),
                on_change: move |idx| active_tab.set(idx)
            }

            if loading() {
                LoadingSpinner { message: "Loading favorites..." }
            } else {
                match active_tab() {
                    0 => rsx! {
                        if starred_songs.read().is_empty() {
                            EmptyState {
                                title: "No starred songs".to_string(),
                                message: Some("Star your favorite songs to see them here.".to_string())
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
                                title: "No starred albums".to_string(),
                                message: Some("Star your favorite albums to see them here.".to_string())
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
                                title: "No starred artists".to_string(),
                                message: Some("Star your favorite artists to see them here.".to_string())
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
