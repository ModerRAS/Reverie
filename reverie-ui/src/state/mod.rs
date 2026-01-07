//! Global application state management
//!
//! Uses Dioxus signals for reactive state management.

use crate::api::{Album, Artist, Playlist, Song};
use dioxus::prelude::*;

/// Authentication state
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: String,
    pub server_url: String,
}

/// Player state
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerState {
    pub is_playing: bool,
    pub current_song: Option<Song>,
    pub queue: Vec<Song>,
    pub queue_index: usize,
    pub volume: f32,
    pub progress: f32,
    pub duration: f32,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

/// Repeat mode for the player
#[derive(Debug, Clone, Default, PartialEq, Copy)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }
}

/// UI state for sidebar and panels
#[derive(Debug, Clone, Default, PartialEq)]
pub struct UiState {
    pub sidebar_open: bool,
    pub now_playing_panel_open: bool,
    pub search_query: String,
    pub current_view: ViewType,
}

/// Current view type
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ViewType {
    #[default]
    Albums,
    Artists,
    Songs,
    Playlists,
    Favorites,
    NowPlaying,
    Settings,
    Search,
}

/// Global application context
#[derive(Clone)]
pub struct AppContext {
    pub auth: Signal<AuthState>,
    pub player: Signal<PlayerState>,
    pub ui: Signal<UiState>,
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            auth: Signal::new(AuthState {
                is_authenticated: false,
                username: String::new(),
                server_url: String::from("http://localhost:4533/rest"),
            }),
            player: Signal::new(PlayerState {
                volume: 0.8,
                ..Default::default()
            }),
            ui: Signal::new(UiState {
                sidebar_open: true,
                ..Default::default()
            }),
        }
    }
}

impl Default for AppContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Player actions
pub enum PlayerAction {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    SetVolume(f32),
    SetProgress(f32),
    ToggleShuffle,
    ToggleRepeat,
    PlaySong(Song),
    AddToQueue(Song),
    AddToQueueNext(Song),
    ClearQueue,
    PlayAlbum(Vec<Song>),
    PlayPlaylist(Vec<Song>),
}

/// Apply player action to state
pub fn apply_player_action(state: &mut PlayerState, action: PlayerAction) {
    match action {
        PlayerAction::Play => {
            state.is_playing = true;
        }
        PlayerAction::Pause => {
            state.is_playing = false;
        }
        PlayerAction::Stop => {
            state.is_playing = false;
            state.progress = 0.0;
        }
        PlayerAction::Next => {
            if state.queue_index < state.queue.len().saturating_sub(1) {
                state.queue_index += 1;
                state.current_song = state.queue.get(state.queue_index).cloned();
                state.progress = 0.0;
            } else if state.repeat == RepeatMode::All && !state.queue.is_empty() {
                state.queue_index = 0;
                state.current_song = state.queue.first().cloned();
                state.progress = 0.0;
            }
        }
        PlayerAction::Previous => {
            if state.progress > 3.0 {
                state.progress = 0.0;
            } else if state.queue_index > 0 {
                state.queue_index -= 1;
                state.current_song = state.queue.get(state.queue_index).cloned();
                state.progress = 0.0;
            }
        }
        PlayerAction::SetVolume(vol) => {
            state.volume = vol.clamp(0.0, 1.0);
        }
        PlayerAction::SetProgress(prog) => {
            state.progress = prog;
        }
        PlayerAction::ToggleShuffle => {
            state.shuffle = !state.shuffle;
        }
        PlayerAction::ToggleRepeat => {
            state.repeat = state.repeat.next();
        }
        PlayerAction::PlaySong(song) => {
            state.queue.clear();
            state.queue.push(song.clone());
            state.queue_index = 0;
            state.current_song = Some(song);
            state.is_playing = true;
            state.progress = 0.0;
        }
        PlayerAction::AddToQueue(song) => {
            state.queue.push(song);
        }
        PlayerAction::AddToQueueNext(song) => {
            let insert_pos = state.queue_index + 1;
            if insert_pos <= state.queue.len() {
                state.queue.insert(insert_pos, song);
            } else {
                state.queue.push(song);
            }
        }
        PlayerAction::ClearQueue => {
            state.queue.clear();
            state.queue_index = 0;
            state.current_song = None;
            state.is_playing = false;
            state.progress = 0.0;
        }
        PlayerAction::PlayAlbum(songs) | PlayerAction::PlayPlaylist(songs) => {
            state.queue = songs;
            state.queue_index = 0;
            state.current_song = state.queue.first().cloned();
            state.is_playing = true;
            state.progress = 0.0;
        }
    }
}
