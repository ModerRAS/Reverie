//! Songs page - Table view of all songs

use dioxus::prelude::*;
use crate::api::Song;
use crate::components::{TrackList, PageHeader, LoadingSpinner, EmptyState};

/// Songs page component
#[component]
pub fn SongsPage() -> Element {
    let mut songs = use_signal(Vec::<Song>::new);
    let mut loading = use_signal(|| true);

    // Load songs
    use_effect(move || {
        loading.set(true);
        
        // Demo data
        let demo_songs: Vec<Song> = (1..=50).map(|i| Song {
            id: format!("song-{}", i),
            title: format!("Song Title {}", i),
            album: Some(format!("Album {}", (i % 10) + 1)),
            album_id: Some(format!("album-{}", (i % 10) + 1)),
            artist: Some(format!("Artist {}", (i % 5) + 1)),
            artist_id: Some(format!("artist-{}", (i % 5) + 1)),
            track: Some((i % 12) as i32 + 1),
            year: Some(2020 + (i % 5) as i32),
            genre: Some(["Rock", "Pop", "Jazz", "Electronic", "Classical"][i % 5].to_string()),
            cover_art: None,
            duration: Some(180 + (i * 10) as i32),
            bit_rate: Some(320),
            suffix: Some("mp3".to_string()),
            content_type: Some("audio/mpeg".to_string()),
            path: Some(format!("/music/song-{}.mp3", i)),
            starred: None,
            play_count: (i * 5) as i32,
        }).collect();
        
        songs.set(demo_songs);
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
