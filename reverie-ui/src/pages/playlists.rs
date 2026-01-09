//! Playlists page - Grid view of all playlists

use crate::api::Playlist;
use crate::components::{EmptyState, LoadingSpinner, PageHeader, PlaylistCard};
use crate::mock;
use dioxus::prelude::*;

/// Playlists page component
#[component]
pub fn PlaylistsPage() -> Element {
    let mut playlists = use_signal(Vec::<Playlist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // Load playlists
    use_effect(move || {
        loading.set(true);

        // Demo data

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
                title: "Playlists".to_string(),
                subtitle: Some(format!("{} playlists", playlists.read().len())),

                // Create playlist button
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
                    "New Playlist"
                }
            }

            if loading() {
                LoadingSpinner { message: "Loading playlists..." }
            } else if playlists.read().is_empty() {
                EmptyState {
                    title: "No playlists yet".to_string(),
                    message: Some("Create a playlist to organize your favorite music.".to_string())
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
