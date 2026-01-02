# Reverie

A modern, lightweight music streaming server written in Rust, inspired by [Navidrome](https://github.com/navidrome/navidrome). Reverie features a fully abstracted storage and network layer, making it easy to swap implementations and extend functionality.

## 🎯 Key Features

- **Abstracted Storage Layer**: Pluggable storage backends (filesystem, database, cloud storage)
- **Abstracted Network Layer**: Flexible HTTP server implementations and external connectivity
- **Type-Safe**: Built with Rust for memory safety and performance
- **Async-First**: Fully asynchronous using Tokio
- **Modular Architecture**: Clean separation between domain logic, storage, and network layers

## 🏗️ Architecture

Reverie is organized into multiple crates with clear responsibilities:

### Core Modules

```
reverie/
├── reverie-core/       # Domain models and business logic
├── reverie-storage/    # Storage abstraction layer
├── reverie-network/    # Network abstraction layer
└── reverie-server/     # Main application server
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
- 🚧 Filesystem + SQLite (planned)
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

### API Endpoints

- `GET /health` - Health check
- `GET /api/tracks` - List tracks (with pagination)
- `GET /api/tracks/:id` - Get track by ID
- `GET /api/tracks/search?q=query` - Search tracks
- `GET /api/albums` - List albums
- `GET /api/albums/:id` - Get album by ID
- `GET /api/albums/:id/tracks` - Get tracks in album
- `GET /api/artists` - List artists
- `GET /api/artists/:id` - Get artist by ID
- `GET /api/artists/:id/albums` - Get albums by artist
- `GET /api/playlists/:id` - Get playlist by ID

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

- [x] Core domain models
- [x] Storage abstraction layer
- [x] Network abstraction layer
- [x] In-memory storage implementation
- [x] Axum HTTP server implementation
- [ ] Filesystem + SQLite storage
- [ ] Library scanner
- [ ] Audio streaming
- [ ] Transcoding support
- [ ] User authentication
- [ ] Subsonic API compatibility
- [ ] Database migrations
- [ ] Configuration file support
- [ ] Docker support
- [ ] Federation/cloud sync

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