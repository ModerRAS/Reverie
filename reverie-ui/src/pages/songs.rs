//! Songs page - Table view of all songs

use crate::api::Song;
use crate::components::{EmptyState, LoadingSpinner, PageHeader, TrackList};
use crate::mock;
use dioxus::prelude::*;

/// Songs page component
#[component]
pub fn SongsPage() -> Element {
    let mut songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);

    // Load songs
    use_effect(move || {
        loading.set(true);

        songs.set(mock::songs(50));
        loading.set(false);
    });

    rsx! {
        div {
            class: "space-y-6",

            PageHeader {
                title: "Songs".to_string(),
                subtitle: Some(format!("{} songs", songs.read().len()))
            }

            if loading() {
                LoadingSpinner { message: "Loading songs..." }
            } else if songs.read().is_empty() {
                EmptyState {
                    title: "No songs found".to_string(),
                    message: Some("Your music library is empty.".to_string())
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
