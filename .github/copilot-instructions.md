# Reverie 项目 AI 编码指南

## 项目概览

Reverie 是一个用 Rust 编写的音乐流媒体服务器，兼容 Subsonic API (版本 1.16.1)。项目采用 **trait-based 抽象** 架构，将存储和网络层完全解耦。目标是实现与 [navidrome](https://github.com/navidrome/navidrome) 相同的功能，但支持多种存储后端。

## 架构核心

### Workspace 结构 (4 个 crate)

```
reverie-core      → 纯领域模型 (Track, Album, Artist, User, Playlist, Subsonic 类型)，无 I/O
reverie-storage   → 存储抽象层 + VFS (OpenDAL) + SQLite 实现
reverie-network   → HTTP 服务层 (Axum 实现 + Subsonic API 端点)
reverie-server    → 应用入口，组装各组件
```

**依赖方向**: `server` → `network`/`storage` → `core`

### 存储层设计 (双层架构)

```
┌─────────────────────────────────────────────────────────────┐
│                    DatabaseStorage                           │
│  (SQLite 元数据存储: tracks, albums, artists, users, etc.)  │
├─────────────────────────────────────────────────────────────┤
│                    VFS (OpenDAL)                             │
│  (媒体文件存储: 本地文件系统/S3/Azure/GCS/WebDAV/SFTP)       │
└─────────────────────────────────────────────────────────────┘
```

#### VFS 配置示例

```rust
// 本地文件系统
let vfs = VfsConfig::local("./music");

// S3 兼容存储 (AWS S3, MinIO, etc.)
let vfs = VfsConfig::s3("bucket", "us-east-1", Some("endpoint"), Some("key"), Some("secret"));

// Azure Blob Storage
let vfs = VfsConfig::azblob("container", "account", Some("key"), None);
```

### 关键设计模式

1. **Trait 抽象** - 所有存储操作通过 trait 定义 ([reverie-storage/src/traits.rs](reverie-storage/src/traits.rs))
2. **VFS 抽象** - 媒体文件操作通过 OpenDAL 实现 ([reverie-storage/src/vfs.rs](reverie-storage/src/vfs.rs))
3. **依赖注入** - 服务器通过泛型接收存储实现: `AxumServer<S: TrackStorage + AlbumStorage + ...>`
4. **Arc 共享** - 存储实例用 `Arc<S>` 包装后传入网络层

```rust
// 典型组装模式 (见 reverie-server/src/main.rs)
let config = DatabaseConfig::new("reverie.db", VfsConfig::local("./music"));
let storage = Arc::new(DatabaseStorage::new(config).await?);
storage.initialize().await?;
let server = AxumServer::new(storage.clone(), network_config);
server.start(addr).await?;
```

## 开发命令

```bash
cargo build                    # 构建所有 crate
cargo test                     # 运行所有测试
cargo run -p reverie-server    # 启动服务器 (端口 4533)
cargo run --example simple_server  # 带示例数据启动
```

## Subsonic API 实现状态

### 已实现 (36 端点)
- System: `/ping`, `/getLicense`
- Browsing: `/getMusicFolders`, `/getGenres`, `/getIndexes`, `/getArtists`, `/getMusicDirectory`, `/getArtist`, `/getAlbum`, `/getSong`, `/getAlbumInfo`, `/getArtistInfo`
- Search: `/search2`, `/search3`
- Playlists: `/getPlaylists`, `/getPlaylist`, `/createPlaylist`, `/updatePlaylist`, `/deletePlaylist`
- Users: `/getUser`, `/getUsers`
- Media: `/stream`, `/download`, `/getCoverArt`, `/getAvatar`
- Annotation: `/getStarred`, `/getStarred2`, `/star`, `/unstar`, `/setRating`, `/scrobble`, `/getNowPlaying`, `/getRandomSongs`, `/getLyrics`
- Scanning: `/getScanStatus`, `/startScan`

### 待实现 (参考 navidrome, ~25 端点)
- AlbumList: `/getAlbumList`, `/getAlbumList2`, `/getSongsByGenre`
- Extended: `/getTopSongs`, `/getSimilarSongs`, `/getSimilarSongs2`, `/getAlbumInfo2`, `/getArtistInfo2`
- Bookmarks: `/getBookmarks`, `/createBookmark`, `/deleteBookmark`, `/getPlayQueue`, `/savePlayQueue`
- Radio: `/getInternetRadioStations`, `/createInternetRadioStation`, `/updateInternetRadioStation`, `/deleteInternetRadioStation`
- Sharing: `/getShares`, `/createShare`, `/updateShare`, `/deleteShare`
- OpenSubsonic: `/getOpenSubsonicExtensions`, `/getLyricsBySongId`

## 添加新功能指南

### 新增 VFS 后端

1. 在 `Cargo.toml` 添加 feature: `vfs-xxx = ["opendal/services-xxx"]`
2. 在 [reverie-storage/src/vfs.rs](reverie-storage/src/vfs.rs) 的 `VfsConfig` 添加构造函数
3. 在 `OpendalVfs::build_operator()` 添加对应的 builder

### 新增 Subsonic API 端点

1. 在 [reverie-network/src/subsonic/mod.rs](reverie-network/src/subsonic/mod.rs) 添加 handler + route
2. 必要时在 [reverie-storage/src/traits.rs](reverie-storage/src/traits.rs) 的 `SubsonicStorage` trait 添加方法
3. 在各存储实现 (memory.rs, database.rs) 中实现新方法
4. 测试在 [reverie-network/src/subsonic/tests.rs](reverie-network/src/subsonic/tests.rs) 添加

### 新增领域模型

1. 在 [reverie-core/src/models.rs](reverie-core/src/models.rs) 定义 struct (需 `Serialize`, `Deserialize`)
2. 在 `reverie-storage/traits.rs` 添加对应 trait
3. 在 database.rs 的 migrations 添加表结构
4. 在各存储实现中实现新 trait

## 错误处理约定

- 每个 crate 有独立的 `error.rs`，使用 `thiserror` 定义错误类型
- 导出 `Result<T>` 类型别名: `pub type Result<T> = std::result::Result<T, XxxError>;`
- 跨 crate 边界使用 `anyhow` 转换

## 测试模式

```rust
// 存储测试模式 (见 reverie-storage/tests/)
#[tokio::test]
async fn test_xxx() {
    let storage = DatabaseStorage::new(DatabaseConfig::memory()).await.unwrap();
    storage.initialize().await.unwrap();
    // ... 测试操作
}

// API 测试模式 (见 reverie-network/src/subsonic/tests.rs)
#[tokio::test]
async fn test_endpoint() {
    let router = create_subsonic_router();
    let response = router
        .oneshot(Request::builder().uri("/ping").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), 200);
}
```

## 重要约定

- 所有 I/O 操作必须 `async`，使用 `#[async_trait]` 宏
- 存储 trait 必须包含 `Send + Sync` bounds
- UUID 使用 `uuid::Uuid`，时间戳使用 `chrono::DateTime<Utc>`
- Subsonic API 响应格式要求同时支持 XML 和 JSON，版本号 `1.16.1`
- VFS 路径使用 Unix 风格 (正斜杠)，无论底层操作系统
- 数据库使用 SQLite，表名使用复数形式 (tracks, albums, artists)
- 工具调用修改代码时要分步骤多次调用工具修改，而不是一次修改几千行，因为你总是很容易的一个文件修改几百行几千行而导致超出最大输出长度。

## 参考项目

- `temp/navidrome/` - Go 实现的 Subsonic 服务器，功能参考，如果不存在可以从https://github.com/navidrome/navidrome.git重新clone过来
- `temp/opendal/` - Rust 存储抽象层，VFS 实现参考，如果不存在可以从https://github.com/apache/opendal.git重新clone过来
- `temp/docsite/` - Reverie 文档站点，架构和设计文档参考，如果不存在可以从https://github.com/DioxusLabs/docsite.git重新clone过来
- `temp/dioxus/` - Dioxus UI 框架，Reverie UI 实现参考，如果不存在可以从https://github.com/DioxusLabs/dioxus.git重新clone过来