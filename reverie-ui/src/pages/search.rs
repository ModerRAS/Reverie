//! Search page - Search results

use crate::api::{Album, Artist, Song};
use crate::components::{
    AlbumCard, ArtistCard, CompactSongList, EmptyState, LoadingSpinner, PageHeader,
};
use crate::state::UiState;
use dioxus::prelude::*;
use crate::mock;

/// Search page component
#[component]
pub fn SearchPage() -> Element {
    let ui_state = use_context::<Signal<UiState>>();
    let query = use_memo(move || ui_state.read().search_query.clone());

    let mut songs = use_signal(Vec::<Song>::new);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    // Search when query changes
    use_effect(move || {
        let q = query();
        if q.is_empty() {
            songs.set(vec![]);
            albums.set(vec![]);
            artists.set(vec![]);
            return;
        }

        loading.set(true);

        // Demo search results
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
                title: "Search for music".to_string(),
                message: Some("Enter a search term to find songs, albums, and artists.".to_string())
            }
        };
    }

    rsx! {
        div {
            class: "space-y-8",

            PageHeader {
                title: format!("Search: \"{}\"", current_query),
                subtitle: Some(format!("{} songs, {} albums, {} artists",
                    songs.read().len(), albums.read().len(), artists.read().len()))
            }

            if loading() {
                LoadingSpinner { message: "Searching..." }
            } else {
                // Artists section
                if !artists.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "Artists" }
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

                // Albums section
                if !albums.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "Albums" }
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

                // Songs section
                if !songs.read().is_empty() {
                    section {
                        class: "space-y-4",
                        h2 { class: "text-xl font-bold", "Songs" }
                        CompactSongList { songs: songs.read().clone() }
                    }
                }

                // No results
                if songs.read().is_empty() && albums.read().is_empty() && artists.read().is_empty() {
                    EmptyState {
                        title: "No results found".to_string(),
                        message: Some(format!("No matches for \"{}\"", current_query))
                    }
                }
            }
        }
    }
}
