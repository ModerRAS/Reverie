//! MemoryStorage 基础结构

use crate::traits::*;

use reverie_core::{Album, Artist, Playlist, PlaylistTrack, Track, User};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 使用 HashMap 的内存存储实现
#[derive(Clone)]
pub struct MemoryStorage {
    pub(crate) tracks: Arc<RwLock<HashMap<Uuid, Track>>>,
    pub(crate) albums: Arc<RwLock<HashMap<Uuid, Album>>>,
    pub(crate) artists: Arc<RwLock<HashMap<Uuid, Artist>>>,
    pub(crate) users: Arc<RwLock<HashMap<Uuid, User>>>,
    pub(crate) playlists: Arc<RwLock<HashMap<Uuid, Playlist>>>,
    pub(crate) playlist_tracks: Arc<RwLock<HashMap<Uuid, Vec<PlaylistTrack>>>>,
    pub(crate) files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            tracks: Arc::new(RwLock::new(HashMap::new())),
            albums: Arc::new(RwLock::new(HashMap::new())),
            artists: Arc::new(RwLock::new(HashMap::new())),
            users: Arc::new(RwLock::new(HashMap::new())),
            playlists: Arc::new(RwLock::new(HashMap::new())),
            playlist_tracks: Arc::new(RwLock::new(HashMap::new())),
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}
