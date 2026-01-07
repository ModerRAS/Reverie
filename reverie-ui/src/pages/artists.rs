//! Artists page - Grid view of all artists

use dioxus::prelude::*;
use crate::api::Artist;
use crate::components::{ArtistCard, PageHeader, LoadingSpinner, EmptyState};

/// Artists page component
#[component]
pub fn ArtistsPage() -> Element {
    let mut artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| true);
    let navigator = use_navigator();

    // Load artists
    use_effect(move || {
        loading.set(true);
        
        // Demo data
        let demo_artists: Vec<Artist> = (1..=20).map(|i| Artist {
            id: format!("artist-{}", i),
            name: format!("Artist {}", i),
            album_count: 3 + (i % 5) as i32,
            cover_art: None,
            artist_image_url: None,
            starred: None,
        }).collect();
        
        artists.set(demo_artists);
        loading.set(false);
    });

    let on_artist_click = move |id: String| {
        navigator.push(format!("/artist/{}", id));
    };

    rsx! {
        div {
            class: "space-y-6",
            
            PageHeader {
                title: "Artists".to_string(),
                subtitle: Some(format!("{} artists", artists.read().len()))
            }
            
            if loading() {
                LoadingSpinner { message: "Loading artists..." }
            } else if artists.read().is_empty() {
                EmptyState {
                    title: "No artists found".to_string(),
                    message: Some("Your music library is empty.".to_string())
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
