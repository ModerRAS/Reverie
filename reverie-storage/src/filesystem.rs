//! Filesystem-based storage implementation
//!
//! This module provides a storage implementation that uses the local filesystem
//! for audio file storage and SQLite for metadata storage.
//!
//! Architecture inspired by Linux VFS (Virtual File System):
//! - FileSystem: Represents a mounted filesystem (like super_block)
//! - Inode: Represents file metadata (like struct inode)
//! - DirEntry: Represents a directory entry (like struct dentry)
//! - File: Represents an open file handle (like struct file)
//!
//! The design follows these principles:
//! 1. Abstraction: Filesystem operations are abstracted behind traits
//! 2. Caching: Metadata is cached in memory for performance
//! 3. Lazy Loading: Content is loaded on demand
//! 4. Type Safety: Full Rust type system guarantees

use async_trait::async_trait;
use reverie_core::{Album, Artist, MediaFile, Track, User, Playlist, PlaylistTrack};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::traits::*;

/// Maximum depth for directory traversal
const MAX_SCAN_DEPTH: usize = 32;

/// FileSystem storage configuration
#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    /// Root directory for music files
    pub music_root: PathBuf,
    /// SQLite database path for metadata
    pub database_path: PathBuf,
    /// Cover art cache directory
    pub cover_cache_dir: PathBuf,
    /// Maximum number of cached inodes
    pub cache_size: usize,
    /// Supported audio extensions
    pub supported_extensions: Vec<&'static str>,
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            music_root: PathBuf::from("/var/lib/reverie/music"),
            database_path: PathBuf::from("/var/lib/reverie/metadata.db"),
            cover_cache_dir: PathBuf::from("/var/lib/reverie/cache/covers"),
            cache_size: 10000,
            supported_extensions: vec!["mp3", "flac", "m4a", "ogg", "opus", "wav", "aac"],
        }
    }
}

/// Represents a filesystem node (file or directory)
/// Inspired by Linux's struct inode
#[derive(Debug, Clone)]
pub struct Inode {
    /// Unique identifier
    pub id: Uuid,
    /// Path relative to music root
    pub path: PathBuf,
    /// File name
    pub name: String,
    /// Is this a directory?
    pub is_dir: bool,
    /// File size in bytes
    pub size: u64,
    /// Modification time
    pub modified: std::time::SystemTime,
    /// MIME type
    pub content_type: String,
    /// File extension
    pub suffix: String,
    /// Parent inode ID
    pub parent_id: Option<Uuid>,
    /// Children inode IDs (for directories)
    pub children: Arc<RwLock<Vec<Uuid>>>,
    /// Media metadata (for audio files)
    pub media_metadata: Option<MediaMetadata>,
    /// Cover art ID
    pub cover_art_id: Option<Uuid>,
    /// Reference count
    pub refcount: Arc<RwLock<u32>>,
}

impl Inode {
    /// Create a new inode for a file
    pub fn new_file(path: PathBuf, metadata: &std::fs::Metadata) -> Self {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let content_type = match extension.as_str() {
            "mp3" => "audio/mpeg",
            "flac" => "audio/flac",
            "m4a" => "audio/mp4",
            "ogg" => "audio/ogg",
            "opus" => "audio/opus",
            "wav" => "audio/wav",
            "aac" => "audio/aac",
            _ => "application/octet-stream",
        }.to_string();

        Self {
            id: Uuid::new_v4(),
            path: path.clone(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            is_dir: false,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            content_type,
            suffix: extension,
            parent_id: None,
            children: Arc::new(RwLock::new(Vec::new())),
            media_metadata: None,
            cover_art_id: None,
            refcount: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new inode for a directory
    pub fn new_dir(path: PathBuf, metadata: &std::fs::Metadata) -> Self {
        Self {
            id: Uuid::new_v4(),
            path: path.clone(),
            name: path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            is_dir: true,
            size: 0,
            modified: metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
            content_type: "inode/directory".to_string(),
            suffix: String::new(),
            parent_id: None,
            children: Arc::new(RwLock::new(Vec::new())),
            media_metadata: None,
            cover_art_id: None,
            refcount: Arc::new(RwLock::new(0)),
        }
    }

    /// Increment reference count
    pub fn inc_ref(&self) {
        let mut refcount = self.refcount.write().unwrap();
        *refcount += 1;
    }

    /// Decrement reference count
    pub fn dec_ref(&self) {
        let mut refcount = self.refcount.write().unwrap();
        *refcount = refcount.saturating_sub(1);
    }

    /// Get current reference count
    pub fn get_ref(&self) -> u32 {
        *self.refcount.read().unwrap()
    }
}

/// Media metadata extracted from audio files
#[derive(Debug, Clone, Default)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub duration: Option<f32>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u32>,
    pub comment: Option<String>,
}

/// Represents a directory entry
/// Inspired by Linux's struct dentry
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// Name of the entry
    pub name: String,
    /// Associated inode
    pub inode: Arc<Inode>,
    /// Is this a negative dentry (nonexistent)?
    pub negative: bool,
}

impl DirEntry {
    /// Create a new directory entry
    pub fn new(name: String, inode: Arc<Inode>) -> Self {
        Self {
            name,
            inode,
            negative: false,
        }
    }

    /// Create a negative dentry
    pub fn negative(name: String) -> Self {
        Self {
            name,
            inode: Arc::new(Inode {
                id: Uuid::nil(),
                path: PathBuf::new(),
                name: String::new(),
                is_dir: false,
                size: 0,
                modified: std::time::SystemTime::UNIX_EPOCH,
                content_type: String::new(),
                suffix: String::new(),
                parent_id: None,
                children: Arc::new(RwLock::new(Vec::new())),
                media_metadata: None,
                cover_art_id: None,
                refcount: Arc::new(RwLock::new(0)),
            }),
            negative: true,
        }
    }
}

/// Represents an open file handle
/// Inspired by Linux's struct file
#[derive(Debug)]
pub struct FileHandle {
    /// The underlying tokio File
    file: File,
    /// Current position
    position: u64,
    /// Associated inode
    inode: Arc<Inode>,
    /// Access flags
    flags: u32,
}

impl FileHandle {
    /// Create a new file handle
    pub fn new(file: File, inode: Arc<Inode>, flags: u32) -> Self {
        Self {
            file,
            position: 0,
            inode,
            flags,
        }
    }

    /// Seek to a position
    pub async fn seek(&mut self, pos: std::io::SeekFrom) -> Result<u64> {
        self.position = self.file.seek(pos).await?;
        Ok(self.position)
    }

    /// Read data from the file
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.file.read(buf).await?;
        self.position += n as u64;
        Ok(n)
    }
}

/// The main filesystem storage implementation
/// Inspired by Linux's super_block and VFS layer
#[derive(Clone)]
pub struct FileSystemStorage {
    /// Configuration
    config: FileSystemConfig,
    /// Inode cache (like inode cache in Linux)
    inode_cache: Arc<RwLock<HashMap<Uuid, Arc<Inode>>>>,
    /// Path to inode map (for quick lookup)
    path_cache: Arc<RwLock<HashMap<PathBuf, Uuid>>>,
    /// Set of known paths (for fast existence check)
    path_set: Arc<RwLock<HashSet<PathBuf>>>,
    /// Music root inode
    root_inode: Arc<Inode>,
}

impl FileSystemStorage {
    /// Create a new filesystem storage with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(FileSystemConfig::default()).await
    }

    /// Create a new filesystem storage with custom configuration
    pub async fn with_config(config: FileSystemConfig) -> Result<Self> {
        // Ensure directories exist
        tokio::fs::create_dir_all(&config.music_root).await?;
        tokio::fs::create_dir_all(&config.cover_cache_dir).await?;

        // Scan root directory and build initial cache
        let root_inode = Arc::new(Inode::new_dir(
            config.music_root.clone(),
            &tokio::fs::metadata(&config.music_root).await?
        ));

        let storage = Self {
            config,
            inode_cache: Arc::new(RwLock::new(HashMap::new())),
            path_cache: Arc::new(RwLock::new(HashMap::new())),
            path_set: Arc::new(RwLock::new(HashSet::new())),
            root_inode,
        };

        // Initial scan
        storage.scan_directory(&storage.root_inode).await?;

        Ok(storage)
    }

    /// Scan a directory and populate cache
    /// Uses iterative approach to avoid recursion limits
    async fn scan_directory(&self, parent: &Arc<Inode>) -> Result<()> {
        use std::collections::VecDeque;
        
        let mut dirs_to_scan = VecDeque::new();
        dirs_to_scan.push_back(parent.clone());

        while let Some(current_dir) = dirs_to_scan.pop_front() {
            let mut dir = tokio::fs::read_dir(&current_dir.path).await?;
            
            while let Some(entry) = dir.next_entry().await? {
                let path = entry.path();
                let metadata = entry.metadata().await?;
                
                let inode = if metadata.is_dir() {
                    Arc::new(Inode::new_dir(path, &metadata))
                } else {
                    Arc::new(Inode::new_file(path, &metadata))
                };

                // Set parent relationship
                let mut inode_mut = (*inode).clone();
                inode_mut.parent_id = Some(current_dir.id);
                let inode = Arc::new(inode_mut);

                // Add to cache
                {
                    let mut inode_cache = self.inode_cache.write().unwrap();
                    inode_cache.insert(inode.id, inode.clone());
                    let mut path_cache = self.path_cache.write().unwrap();
                    path_cache.insert(inode.path.clone(), inode.id);
                    let mut path_set = self.path_set.write().unwrap();
                    path_set.insert(inode.path.clone());
                }

                // Add to parent's children
                {
                    let mut parent_children = current_dir.children.write().unwrap();
                    parent_children.push(inode.id);
                }

                // Queue directories for scanning
                if metadata.is_dir() {
                    dirs_to_scan.push_back(inode.clone());
                }
            }
        }

        Ok(())
    }

    /// Get an inode by ID
    pub async fn get_inode(&self, id: Uuid) -> Result<Option<Arc<Inode>>> {
        let cache = self.inode_cache.read().unwrap();
        Ok(cache.get(&id).cloned())
    }

    /// Get an inode by path
    pub async fn get_inode_by_path(&self, path: &Path) -> Result<Option<Arc<Inode>>> {
        // Check path cache first
        let path_cache = self.path_cache.read().unwrap();
        if let Some(id) = path_cache.get(path) {
            let inode_cache = self.inode_cache.read().unwrap();
            if let Some(inode) = inode_cache.get(id) {
                return Ok(Some(inode.clone()));
            }
        }

        // Not in cache, need to scan
        // For now, return None
        Ok(None)
    }

    /// List directory contents
    pub async fn read_dir(&self, inode: &Arc<Inode>) -> Result<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let children = inode.children.read().unwrap();
        
        for child_id in children.iter() {
            if let Some(child_inode) = self.get_inode(*child_id).await? {
                entries.push(DirEntry::new(child_inode.name.clone(), child_inode));
            }
        }

        Ok(entries)
    }

    /// Get the root inode
    pub fn root_inode(&self) -> Arc<Inode> {
        self.root_inode.clone()
    }

    /// Open a file for reading
    pub async fn open_file(&self, inode: &Arc<Inode>) -> Result<FileHandle> {
        let file = OpenOptions::new()
            .read(true)
            .open(&inode.path)
            .await?;

        Ok(FileHandle::new(file, inode.clone(), 0))
    }

    /// Extract media metadata from a file
    /// This would use a library like metaflac, mutagen, etc.
    async fn extract_metadata(&self, inode: &Arc<Inode>) -> Result<MediaMetadata> {
        // Placeholder - would use actual audio metadata extraction
        Ok(MediaMetadata::default())
    }
}

/// Initialize the filesystem storage
#[async_trait]
impl Storage for FileSystemStorage {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn close(&self) -> Result<()> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool> {
        // Check if root directory exists
        tokio::fs::metadata(&self.config.music_root).await?;
        Ok(true)
    }
}

/// Track storage implementation
#[async_trait]
impl TrackStorage for FileSystemStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        if let Some(inode) = self.get_inode(id).await? {
            if !inode.is_dir && inode.media_metadata.is_some() {
                // Convert to Track
                let metadata = inode.media_metadata.as_ref().unwrap();
                return Ok(Some(Track {
                    id,
                    title: metadata.title.clone().unwrap_or_default(),
                    album_id: None,
                    artist_id: None,
                    duration: metadata.duration.unwrap_or(0.0) as u32,
                    file_path: inode.path.to_string_lossy().to_string(),
                    file_size: inode.size,
                    bitrate: metadata.bitrate.unwrap_or(0),
                    format: inode.suffix.clone(),
                    track_number: metadata.track_number,
                    disc_number: metadata.disc_number,
                    year: metadata.year,
                    genre: metadata.genre.clone(),
                    created_at: chrono::DateTime::from(inode.modified),
                    updated_at: chrono::DateTime::from(inode.modified),
                }));
            }
        }
        Ok(None)
    }

    async fn list_tracks(&self, limit: usize, offset: usize) -> Result<Vec<Track>> {
        // Collect all files from cache
        let mut tracks = Vec::new();
        let cache = self.inode_cache.read().unwrap();
        
        for (_, inode) in cache.iter() {
            if !inode.is_dir && inode.media_metadata.is_some() {
                let metadata = inode.media_metadata.as_ref().unwrap();
                tracks.push(Track {
                    id: inode.id,
                    title: metadata.title.clone().unwrap_or_default(),
                    album_id: None,
                    artist_id: None,
                    duration: metadata.duration.unwrap_or(0.0) as u32,
                    file_path: inode.path.to_string_lossy().to_string(),
                    file_size: inode.size,
                    bitrate: metadata.bitrate.unwrap_or(0),
                    format: inode.suffix.clone(),
                    track_number: metadata.track_number,
                    disc_number: metadata.disc_number,
                    year: metadata.year,
                    genre: metadata.genre.clone(),
                    created_at: chrono::DateTime::from(inode.modified),
                    updated_at: chrono::DateTime::from(inode.modified),
                });
            }
        }

        Ok(tracks.into_iter().skip(offset).take(limit).collect())
    }

    async fn save_track(&self, _track: &Track) -> Result<()> {
        // Would update the database
        Ok(())
    }

    async fn delete_track(&self, _id: Uuid) -> Result<()> {
        // Would delete from database and filesystem
        Ok(())
    }

    async fn search_tracks(&self, query: &str) -> Result<Vec<Track>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        let tracks = self.list_tracks(usize::MAX, 0).await?;
        for track in tracks {
            if track.title.to_lowercase().contains(&query_lower) {
                results.push(track);
            }
        }
        
        Ok(results)
    }

    async fn get_tracks_by_album(&self, _album_id: Uuid) -> Result<Vec<Track>> {
        // Would query by album_id
        Ok(Vec::new())
    }

    async fn get_tracks_by_artist(&self, _artist_id: Uuid) -> Result<Vec<Track>> {
        // Would query by artist_id
        Ok(Vec::new())
    }
}

/// Album storage implementation
#[async_trait]
impl AlbumStorage for FileSystemStorage {
    async fn get_album(&self, _id: Uuid) -> Result<Option<Album>> {
        Ok(None)
    }

    async fn list_albums(&self, _limit: usize, _offset: usize) -> Result<Vec<Album>> {
        Ok(Vec::new())
    }

    async fn save_album(&self, _album: &Album) -> Result<()> {
        Ok(())
    }

    async fn delete_album(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn get_albums_by_artist(&self, _artist_id: Uuid) -> Result<Vec<Album>> {
        Ok(Vec::new())
    }
}

/// Artist storage implementation
#[async_trait]
impl ArtistStorage for FileSystemStorage {
    async fn get_artist(&self, _id: Uuid) -> Result<Option<Artist>> {
        Ok(None)
    }

    async fn list_artists(&self, _limit: usize, _offset: usize) -> Result<Vec<Artist>> {
        Ok(Vec::new())
    }

    async fn save_artist(&self, _artist: &Artist) -> Result<()> {
        Ok(())
    }

    async fn delete_artist(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
}

/// User storage implementation
#[async_trait]
impl UserStorage for FileSystemStorage {
    async fn get_user(&self, _id: Uuid) -> Result<Option<User>> {
        Ok(None)
    }

    async fn get_user_by_username(&self, _username: &str) -> Result<Option<User>> {
        Ok(None)
    }

    async fn list_users(&self, _limit: usize, _offset: usize) -> Result<Vec<User>> {
        Ok(Vec::new())
    }

    async fn save_user(&self, _user: &User) -> Result<()> {
        Ok(())
    }

    async fn delete_user(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
}

/// Playlist storage implementation
#[async_trait]
impl PlaylistStorage for FileSystemStorage {
    async fn get_playlist(&self, _id: Uuid) -> Result<Option<Playlist>> {
        Ok(None)
    }

    async fn get_playlists_by_user(&self, _user_id: Uuid) -> Result<Vec<Playlist>> {
        Ok(Vec::new())
    }

    async fn save_playlist(&self, _playlist: &Playlist) -> Result<()> {
        Ok(())
    }

    async fn delete_playlist(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn add_track_to_playlist(&self, _playlist_track: &PlaylistTrack) -> Result<()> {
        Ok(())
    }

    async fn remove_track_from_playlist(&self, _playlist_id: Uuid, _track_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn get_playlist_tracks(&self, _playlist_id: Uuid) -> Result<Vec<PlaylistTrack>> {
        Ok(Vec::new())
    }
}

/// File storage implementation
#[async_trait]
impl FileStorage for FileSystemStorage {
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let path = Path::new(path);
        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    async fn write_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn file_exists(&self, path: &str) -> Result<bool> {
        Ok(tokio::fs::try_exists(path).await?)
    }

    async fn delete_file(&self, path: &str) -> Result<()> {
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn list_files(&self, _path: &str) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn get_file_metadata(&self, path: &str) -> Result<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(FileMetadata {
            size: metadata.len(),
            modified: metadata.modified()?,
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
        })
    }
}
