//! Search page - Search results

use dioxus::prelude::*;
use crate::api::{Album, Artist, Song};
use crate::state::UiState;
use crate::components::{AlbumCard, ArtistCard, CompactSongList, PageHeader, LoadingSpinner, EmptyState};

/// Search page component
#[component]
pub fn SearchPage() -> Element {
    let ui_state = use_context::<Signal<UiState>>();
    let query = ui_state.read().search_query.clone();
    
    let mut songs = use_signal(Vec::<Song>::new);
    let mut albums = use_signal(Vec::<Album>::new);
    let mut artists = use_signal(Vec::<Artist>::new);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    // Search when query changes
    use_effect(move || {
        let q = query.clone();
        if q.is_empty() {
            songs.set(vec![]);
            albums.set(vec![]);
            artists.set(vec![]);
            return;
        }
        
        loading.set(true);
        
        // Demo search results
        let result_songs: Vec<Song> = (1..=5).map(|i| Song {
            id: format!("search-song-{}", i),
            title: format!("{} - Result {}", q, i),
            album: Some(format!("Album {}", i)),
            album_id: Some(format!("album-{}", i)),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            track: Some(i as i32),
            year: Some(2023),
            genre: None,
            cover_art: None,
            duration: Some(200),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: None,
            path: None,
            starred: None,
            play_count: 0,
        }).collect();
        
        let result_albums: Vec<Album> = (1..=3).map(|i| Album {
            id: format!("search-album-{}", i),
            name: format!("{} Album {}", q, i),
            artist: Some(format!("Artist {}", i)),
            artist_id: Some(format!("artist-{}", i)),
            cover_art: None,
            song_count: Some(10),
            duration: Some(2400),
            year: Some(2023),
            genre: None,
            created: None,
            starred: None,
            play_count: 0,
        }).collect();
        
        let result_artists: Vec<Artist> = (1..=2).map(|i| Artist {
            id: format!("search-artist-{}", i),
            name: format!("{} Artist {}", q, i),
            album_count: 5,
            cover_art: None,
            artist_image_url: None,
            starred: None,
        }).collect();
        
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

    if query.is_empty() {
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
                title: format!("Search: \"{}\"", query),
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
                        message: Some(format!("No matches for \"{}\"", query))
                    }
                }
            }
        }
    }
}
