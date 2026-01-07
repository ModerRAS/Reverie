# Reverie Architecture

This document provides a detailed overview of Reverie's architecture, focusing on the abstractions for storage and network systems.

## Table of Contents

1. [Overview](#overview)
2. [Design Principles](#design-principles)
3. [Module Architecture](#module-architecture)
4. [Storage Abstraction](#storage-abstraction)
5. [Network Abstraction](#network-abstraction)
6. [Data Flow](#data-flow)
7. [Extending the System](#extending-the-system)

## Overview

Reverie is a music streaming server built with Rust, designed from the ground up with flexible, trait-based abstractions. The core principle is **separation of concerns** - the business logic is completely independent of storage and network implementation details.

### Key Goals

1. **Flexibility**: Easy to swap storage backends (filesystem, database, cloud) without changing core logic
2. **Testability**: All components can be easily mocked and tested in isolation
3. **Performance**: Zero-cost abstractions leveraging Rust's trait system
4. **Maintainability**: Clear module boundaries and responsibilities

## Design Principles

### 1. Trait-Based Abstraction

All major subsystems are defined as traits, allowing multiple implementations:

```rust
// Storage abstraction example
#[async_trait]
pub trait TrackStorage: Send + Sync {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>>;
    async fn save_track(&self, track: &Track) -> Result<()>;
    // ... more methods
}
```

### 2. Dependency Injection

Components receive their dependencies through constructors:

```rust
// HTTP server receives storage implementation
pub struct AxumServer<S> {
    storage: Arc<S>,
    config: NetworkConfig,
}
```

### 3. Async-First

All I/O operations are asynchronous using Tokio:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    storage.initialize().await?;
    // ...
}
```

## Module Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    reverie-server                       │
│         (Application Entry Point & Orchestration)       │
│                                                          │
│  - Initializes storage and network components           │
│  - Wires dependencies together                          │
│  - Handles application lifecycle                        │
└────────────────────┬──────────────────┬─────────────────┘
                     │                  │
         ┌───────────▼──────────┐   ┌──▼──────────────────┐
         │  reverie-network     │   │  reverie-storage    │
         │  (HTTP/Networking)   │   │  (Data Persistence) │
         │                      │   │                     │
         │  - HTTP server trait │   │  - Storage traits   │
         │  - Axum impl         │   │  - Memory impl      │
         │  - API handlers      │   │  - File impl (TODO) │
         │  - Media streaming   │   │  - DB impl (TODO)   │
         └───────────┬──────────┘   └──┬──────────────────┘
                     │                  │
                     └──────────┬───────┘
                                │
                    ┌───────────▼──────────┐
                    │    reverie-core      │
                    │  (Domain Models)     │
                    │                      │
                    │  - Track, Album      │
                    │  - Artist, User      │
                    │  - Playlist          │
                    │  - Error types       │
                    └──────────────────────┘
```

### reverie-core

**Purpose**: Define domain models and business logic types.

**Key Components**:
- `models.rs`: Data structures (Track, Album, Artist, User, Playlist)
- `error.rs`: Domain-specific error types

**Dependencies**: Minimal - only serialization and basic utilities.

**Design Philosophy**: This crate should be pure - no I/O, no external services. It represents the "what" of the application.

### reverie-storage

**Purpose**: Provide storage abstraction and implementations.

**Key Components**:
- `traits.rs`: Storage trait definitions
  - `TrackStorage`: Track CRUD operations
  - `AlbumStorage`: Album CRUD operations
  - `ArtistStorage`: Artist CRUD operations
  - `UserStorage`: User management
  - `PlaylistStorage`: Playlist operations
  - `FileStorage`: File operations
  - `Storage`: Combined storage interface
- `memory.rs`: In-memory implementation (for testing)
- `filesystem.rs`: Filesystem implementation (planned)
- `database.rs`: Database implementation (planned)

**Design Philosophy**: Storage implementations are completely isolated from business logic. New backends can be added without touching other code.

### reverie-network

**Purpose**: Provide network abstraction and HTTP server implementations.

**Key Components**:
- `traits.rs`: Network trait definitions
  - `HttpServer`: HTTP server lifecycle
  - `MediaStreamer`: Audio streaming
  - `ExternalConnection`: External connectivity
- `axum_server.rs`: Axum-based HTTP server
- `dto.rs`: Data Transfer Objects for API
- `error.rs`: Network-specific errors

**Design Philosophy**: The network layer is responsible for handling HTTP, but delegates all data operations to storage implementations.

### reverie-server

**Purpose**: Main application entry point.

**Key Components**:
- `main.rs`: Application orchestration

**Responsibilities**:
1. Initialize logging and configuration
2. Create storage backend
3. Create HTTP server
4. Wire dependencies together
5. Start the application

## Storage Abstraction

### Storage Traits Hierarchy

```rust
// Specialized storage traits
TrackStorage    ──┐
AlbumStorage    ──┤
ArtistStorage   ──┤
UserStorage     ──┼──> Storage (combined trait)
PlaylistStorage ──┤
FileStorage     ──┘
```

### Implementation Strategy

Each storage trait is independent and can be implemented separately. The `Storage` trait combines all of them:

```rust
#[async_trait]
pub trait Storage: 
    TrackStorage + 
    AlbumStorage + 
    ArtistStorage + 
    UserStorage + 
    PlaylistStorage + 
    FileStorage 
{
    async fn initialize(&self) -> Result<()>;
    async fn close(&self) -> Result<()>;
    async fn health_check(&self) -> Result<bool>;
}
```

### Current Implementations

#### MemoryStorage

- **Use Case**: Testing, development
- **Storage**: In-memory HashMaps with RwLocks
- **Characteristics**: Fast, ephemeral, thread-safe
- **Limitations**: Data lost on restart

#### Future Implementations

1. **FilesystemStorage** (Planned)
   - Music files on disk
   - SQLite for metadata
   - Efficient for single-server deployments

2. **DatabaseStorage** (Planned)
   - PostgreSQL or MySQL
   - Scalable metadata storage
   - Support for multiple instances

3. **CloudStorage** (Planned)
   - S3-compatible storage for files
   - Cloud database for metadata
   - Distributed deployments

## Network Abstraction

### Network Traits

```rust
HttpServer          -> Server lifecycle management
MediaStreamer       -> Audio file streaming
ExternalConnection  -> Federation, cloud sync
RequestHandler      -> Generic request handling
```

### Axum Implementation

The Axum implementation demonstrates how to build on the network abstraction:

```rust
pub struct AxumServer<S> {
    storage: Arc<S>,  // Generic storage
    config: NetworkConfig,
}

impl<S> AxumServer<S>
where
    S: TrackStorage + AlbumStorage + ... + Clone + 'static
{
    fn create_router(&self) -> Router {
        // Create routes using storage abstraction
    }
}
```

### API Design

REST API endpoints follow resource-oriented design:

```
GET    /api/tracks           -> List tracks
GET    /api/tracks/:id       -> Get track
GET    /api/tracks/search    -> Search tracks
GET    /api/albums           -> List albums
GET    /api/albums/:id       -> Get album
GET    /api/artists          -> List artists
GET    /api/playlists/:id    -> Get playlist
```

## Data Flow

### Request Flow Example: Get Track

```
1. HTTP Request
   ↓
2. Axum Router
   ↓
3. get_track_handler<S>
   ↓
4. TrackStorage::get_track (trait method)
   ↓
5. MemoryStorage::get_track (implementation)
   ↓
6. Return Track model
   ↓
7. Convert to TrackResponse DTO
   ↓
8. Serialize to JSON
   ↓
9. HTTP Response
```

### Key Points

- **Handler is generic**: Works with any `TrackStorage` implementation
- **DTO separation**: API types (TrackResponse) separate from domain types (Track)
- **Type safety**: Rust's type system ensures correct usage

## Extending the System

### Adding a New Storage Backend

Example: PostgreSQL storage

```rust
// 1. Create new module
// reverie-storage/src/postgres.rs

use sqlx::PgPool;

pub struct PostgresStorage {
    pool: PgPool,
}

// 2. Implement storage traits
#[async_trait]
impl TrackStorage for PostgresStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        // PostgreSQL implementation
        let track = sqlx::query_as!(
            Track,
            "SELECT * FROM tracks WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(track)
    }
    // ... implement other methods
}

// 3. Update Cargo.toml features
[features]
postgres = ["sqlx/postgres"]

// 4. Use in server
let storage = Arc::new(PostgresStorage::new(pool));
```

### Adding a New HTTP Server Implementation

Example: Actix-Web server

```rust
// 1. Create new module
// reverie-network/src/actix_server.rs

pub struct ActixServer<S> {
    storage: Arc<S>,
    config: NetworkConfig,
}

// 2. Implement HttpServer trait
#[async_trait]
impl<S> HttpServer for ActixServer<S>
where
    S: TrackStorage + AlbumStorage + ...
{
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        // Actix implementation
    }
    // ... implement other methods
}

// 3. Update Cargo.toml features
[features]
actix-server = ["actix-web"]

// 4. Use in server
let server = ActixServer::new(storage, config);
```

### Adding New API Endpoints

```rust
// 1. Add handler function
async fn create_track_handler<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateTrackRequest>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    // Implementation
}

// 2. Add route in create_router
.route("/api/tracks", post(create_track_handler::<S>))
```

## Testing Strategy

### Unit Tests

- Test storage implementations independently
- Test domain models
- Test utility functions

### Integration Tests

- Test storage with real databases (Docker containers)
- Test API endpoints with test storage
- Test complete request/response cycles

### Example Test

```rust
#[tokio::test]
async fn test_track_crud() {
    let storage = MemoryStorage::new();
    storage.initialize().await.unwrap();
    
    let track = Track { /* ... */ };
    storage.save_track(&track).await.unwrap();
    
    let retrieved = storage.get_track(track.id).await.unwrap();
    assert_eq!(retrieved.unwrap().id, track.id);
}
```

## Performance Considerations

### Storage

- **Memory**: Fast but limited by RAM
- **Filesystem**: Balance between speed and persistence
- **Database**: Connection pooling, query optimization
- **Caching**: Consider adding a cache layer

### Network

- **Streaming**: Efficient file streaming without loading entire files
- **Transcoding**: On-demand transcoding for bandwidth optimization
- **Connection pooling**: Reuse database connections

### Concurrency

- All storage operations are thread-safe
- Use of `Arc` for shared storage access
- `RwLock` for read-heavy workloads

## Security Considerations

### Storage

- Input validation before storage operations
- SQL injection prevention (parameterized queries)
- File path validation to prevent directory traversal

### Network

- Authentication middleware (planned)
- Rate limiting (planned)
- CORS configuration
- Input validation on all endpoints

## Conclusion

Reverie's architecture prioritizes flexibility and extensibility through trait-based abstractions. The clean separation between domain logic, storage, and network layers makes it easy to:

1. **Test**: Mock implementations for unit tests
2. **Extend**: Add new storage/network implementations
3. **Maintain**: Clear module boundaries
4. **Scale**: Swap implementations based on deployment needs

The design follows Rust best practices and leverages the type system for correctness and safety.
