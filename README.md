# Reverie

A modern, lightweight music streaming server written in Rust, inspired by [Navidrome](https://github.com/navidrome/navidrome). Reverie features a fully abstracted storage and network layer, making it easy to swap implementations and extend functionality.

## 🎯 Key Features

- **Abstracted Storage Layer**: Pluggable storage backends (filesystem, database, cloud storage)
- **Abstracted Network Layer**: Flexible HTTP server implementations and external connectivity
- **Type-Safe**: Built with Rust for memory safety and performance
- **Async-First**: Fully asynchronous using Tokio
- **Modular Architecture**: Clean separation between domain logic, storage, and network layers
- **Subsonic API Compatible**: Fully compatible with Subsonic API, works with any Subsonic client
- **Web UI (Optional)**: `reverie-ui` provides a Navidrome-like web interface built with Dioxus

## 📊 Current Development Status

### Implemented Features

**Core Features**
- ✅ Core domain models (Track, Album, Artist, User, Playlist)
- ✅ Storage abstraction layer (SubsonicStorage trait)
- ✅ Memory storage implementation
- ✅ Axum HTTP server
- ✅ Subsonic API endpoints (36 endpoints, 44 tests)

**Subsonic API Endpoints (Completed)**

Basic Endpoints:
- ✅ `/ping` - Health check
- ✅ `/getLicense` - Get license information
- ✅ `/getMusicFolders` - Get music folders
- ✅ `/getGenres` - Get genre list
- ✅ `/getScanStatus` - Get scan status
- ✅ `/startScan` - Start scan

Browse Endpoints:
- ✅ `/getIndexes` - Get artist indexes
- ✅ `/getArtists` - Get artist list
- ✅ `/getMusicDirectory` - Get music directory
- ✅ `/getArtist` - Get artist details
- ✅ `/getAlbum` - Get album details
- ✅ `/getSong` - Get song details
- ✅ `/getAlbumInfo` - Get album info
- ✅ `/getArtistInfo` - Get artist info

Search Endpoints:
- ✅ `/search2` - Search (returns Artist/Album/Child)
- ✅ `/search3` - Search (returns ID3 version)

Playlist Endpoints:
- ✅ `/getPlaylists` - Get playlists
- ✅ `/getPlaylist` - Get playlist details
- ✅ `/createPlaylist` - Create playlist
- ✅ `/updatePlaylist` - Update playlist
- ✅ `/deletePlaylist` - Delete playlist

User Endpoints:
- ✅ `/getUser` - Get user info
- ✅ `/getUsers` - Get all users

Streaming Endpoints:
- ✅ `/stream` - Stream media
- ✅ `/download` - Download
- ✅ `/getCoverArt` - Get cover art
- ✅ `/getAvatar` - Get user avatar

Star & Rating:
- ✅ `/getStarred` - Get starred content
- ✅ `/getStarred2` - Get starred content (ID3 version)
- ✅ `/star` - Star
- ✅ `/unstar` - Unstar
- ✅ `/setRating` - Set rating

Scrobble:
- ✅ `/scrobble` - Scrobble
- ✅ `/getNowPlaying` - Get now playing
- ✅ `/getRandomSongs` - Get random songs
- ✅ `/getLyrics` - Get lyrics

### Test Coverage

- ✅ 36 Subsonic API tests
- ✅ 8 Memory storage tests
- ✅ Total: 44 tests passing

## 🏗️ Architecture

Reverie is organized into multiple crates with clear responsibilities:

### Core Modules

```
reverie/
├── reverie-core/       # Domain models and business logic
├── reverie-storage/    # Storage abstraction layer
├── reverie-network/    # Network abstraction layer
└── reverie-server/     # Main application server
└── reverie-ui/         # Optional Web UI (Dioxus)
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    reverie-server                       │
│              (Application Orchestration)                │
└────────────────────┬──────────────────┬─────────────────┘
                     │                  │
         ┌───────────▼──────────┐   ┌──▼──────────────────┐
         │  reverie-network     │   │  reverie-storage    │
         │  (HTTP/Networking)   │   │  (Data Persistence) │
         └───────────┬──────────┘   └──┬──────────────────┘
                     │                  │
                     └──────────┬───────┘
                                │
                    ┌───────────▼──────────┐
                    │    reverie-core      │
                    │  (Domain Models)     │
                    └──────────────────────┘
```

### Storage Abstraction

The storage layer provides traits for different types of operations:

- **TrackStorage**: Manage music tracks
- **AlbumStorage**: Manage albums
- **ArtistStorage**: Manage artists
- **UserStorage**: Manage users
- **PlaylistStorage**: Manage playlists
- **FileStorage**: Manage file operations (audio files, cover art)

**Available Implementations:**
- ✅ In-Memory (for testing/development)
- ✅ Filesystem (with metadata caching)
- 🚧 SQLite integration (planned)
- 🚧 PostgreSQL (planned)
- 🚧 S3-compatible storage (planned)

### Network Abstraction

The network layer provides traits for HTTP serving and external connections:

- **HttpServer**: HTTP server implementation
- **MediaStreamer**: Audio streaming with optional transcoding
- **ExternalConnection**: External network connectivity (federation, cloud sync)

**Available Implementations:**
- ✅ Axum-based HTTP server
- 🚧 Transcoding support (planned)
- 🚧 Federation protocol (planned)

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
# Build all crates
cargo build --release

# Build only the server
cargo build --release -p reverie-server
```

### Running

```bash
# Run the server
cargo run --release -p reverie-server

# Or run the compiled binary
./target/release/reverie
```

The server will start on `http://127.0.0.1:4533` by default.

### Running the Web UI (optional)

The Web UI is a separate crate (`reverie-ui`) and talks to the server via the Subsonic API under `/rest`.

Prerequisites:

```bash
cargo install dioxus-cli
```

Run:

```bash
cd reverie-ui
dx serve
```

Then open `http://localhost:8080`. In dev, the UI proxies `/rest` to `http://127.0.0.1:4533/rest`.

### More Docs

See the detailed docs under `Docs/` (architecture, summary, etc.).

### API Endpoints

Reverie implements a comprehensive Subsonic API compatible interface (version 1.16.1).

**Base URL:** `/rest`

**Authentication:** Subsonic clients authenticate using the standard Subsonic authentication protocol.

**Basic Endpoints:**
- `GET /rest/ping` - Health check
- `GET /rest/getLicense` - Get license information
- `GET /rest/getMusicFolders` - Get music folders list
- `GET /rest/getGenres` - Get genre list
- `GET /rest/getScanStatus` - Get scan status
- `GET /rest/startScan` - Start scan

**Browse Endpoints:**
- `GET /rest/getIndexes` - Get artist indexes
- `GET /rest/getArtists` - Get complete artist list
- `GET /rest/getMusicDirectory` - Get music directory
- `GET /rest/getArtist` - Get artist details
- `GET /rest/getAlbum` - Get album details
- `GET /rest/getSong` - Get song details
- `GET /rest/getAlbumInfo` - Get album info
- `GET /rest/getArtistInfo` - Get artist info

**Search Endpoints:**
- `GET /rest/search2` - Search (returns Artist/Album/Child)
- `GET /rest/search3` - Search (returns ID3 version)

**Playlist Endpoints:**
- `GET /rest/getPlaylists` - Get all playlists
- `GET /rest/getPlaylist` - Get playlist details
- `GET /rest/createPlaylist` - Create playlist
- `GET /rest/updatePlaylist` - Update playlist
- `GET /rest/deletePlaylist` - Delete playlist

**User Endpoints:**
- `GET /rest/getUser` - Get user info
- `GET /rest/getUsers` - Get all users

**Streaming Endpoints:**
- `GET /rest/stream` - Stream media
- `GET /rest/download` - Download audio file
- `GET /rest/getCoverArt` - Get cover art
- `GET /rest/getAvatar` - Get user avatar

**Star & Rating:**
- `GET /rest/getStarred` - Get starred content
- `GET /rest/getStarred2` - Get starred content (ID3 version)
- `GET /rest/star` - Star
- `GET /rest/unstar` - Unstar
- `GET /rest/setRating` - Set rating

**Scrobble:**
- `GET /rest/scrobble` - Scrobble
- `GET /rest/getNowPlaying` - Get now playing
- `GET /rest/getRandomSongs` - Get random songs
- `GET /rest/getLyrics` - Get lyrics

## 🔧 Development

### Project Structure

Each crate has a specific purpose:

**reverie-core**: Domain models and shared types
- No external dependencies (except serialization)
- Pure data structures
- Business logic types

**reverie-storage**: Storage abstraction and implementations
- Trait definitions for storage operations
- Multiple backend implementations
- Async-first API

**reverie-network**: Network abstraction and HTTP server
- HTTP server trait and implementations
- API handlers and routing
- External connectivity abstractions

**reverie-server**: Main application
- Wires together storage and network layers
- Configuration management
- Application startup and lifecycle

### Adding a New Storage Backend

1. Implement the storage traits in `reverie-storage/src/`
2. Add your implementation to the features in `reverie-storage/Cargo.toml`
3. Update the server to use your storage backend

Example:

```rust
use async_trait::async_trait;
use reverie_storage::{TrackStorage, Result};

pub struct MyCustomStorage { /* ... */ }

#[async_trait]
impl TrackStorage for MyCustomStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        // Your implementation
    }
    // ... implement other methods
}
```

### Adding a New HTTP Server

1. Implement the `HttpServer` trait in `reverie-network/src/`
2. Add your implementation to the features
3. Update the server to use your HTTP implementation

## 🎨 Design Principles

1. **Abstraction First**: Core logic is independent of implementation details
2. **Dependency Injection**: Implementations are injected at runtime
3. **Type Safety**: Leverage Rust's type system for correctness
4. **Testability**: Easy to mock storage and network layers
5. **Performance**: Zero-cost abstractions with Rust's trait system
6. **Extensibility**: Easy to add new features without modifying core logic

## 📝 Comparison with Navidrome

| Feature | Navidrome | Reverie |
|---------|-----------|---------|
| Language | Go | Rust |
| Storage | SQLite | Abstracted (multiple backends) |
| Network | Built-in HTTP | Abstracted (pluggable servers) |
| API | Subsonic API | Custom REST API |
| Transcoding | FFmpeg | Planned |
| Federation | No | Planned |

## 🛣️ Roadmap

### Completed ✅

- [x] Core domain models (Track, Album, Artist, User, Playlist)
- [x] Subsonic-compatible data structures (MediaFile, SubsonicAlbum, SubsonicArtist, etc.)
- [x] Storage abstraction layer (all Storage traits)
- [x] SubsonicStorage trait (complete Subsonic API storage interface)
- [x] Memory storage implementation (MemoryStorage)
- [x] Filesystem storage implementation (FileSystemStorage)
  - VFS-inspired architecture (Inode, DirEntry, FileHandle)
  - Directory traversal with iterative scanning
  - Metadata caching with HashMap-based cache
  - Support for common audio formats (MP3, FLAC, M4A, OGG, etc.)
- [x] Axum HTTP server implementation
- [x] Subsonic API basic endpoints (ping, license, folders, genres, scan)
- [x] Subsonic API browse endpoints (directory, artist, album, song)
- [x] Subsonic API search endpoints (search2, search3)
- [x] Subsonic API playlist endpoints (CRUD)
- [x] Subsonic API user endpoints
- [x] Subsonic API streaming endpoints (stream, download, coverArt)
- [x] Subsonic API star & rating endpoints
- [x] Subsonic API scrobble endpoints
- [x] Complete test coverage (44 tests)

### In Progress 🚧

- [ ] SQLite metadata integration for filesystem storage
- [ ] Library scanner with audio metadata extraction
- [ ] Audio streaming implementation
- [ ] Transcoding support (FFmpeg integration)

### Planned 📋

- [ ] User authentication system
- [ ] Database migrations
- [ ] Configuration file support
- [ ] Docker support
- [ ] Federation/cloud sync
- [ ] Additional storage backends (PostgreSQL, S3)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! The modular architecture makes it easy to contribute:

- Add new storage backends
- Add new network implementations
- Improve existing implementations
- Add tests
- Improve documentation

## 🙏 Acknowledgments

Inspired by [Navidrome](https://github.com/navidrome/navidrome) - an excellent music streaming server that proved the concept.