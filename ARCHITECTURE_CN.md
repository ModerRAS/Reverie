# Reverie 架构

本文档提供了 Reverie 架构的详细概述，重点关注存储和网络系统的抽象设计。

## 目录

1. [概述](#概述)
2. [设计原则](#设计原则)
3. [模块架构](#模块架构)
4. [存储抽象](#存储抽象)
5. [网络抽象](#网络抽象)
6. [数据流](#数据流)
7. [扩展系统](#扩展系统)

## 概述

Reverie 是一个用 Rust 构建的音乐流媒体服务器，从头开始设计，具有灵活的、基于特征（trait）的抽象。核心原则是**关注点分离**——业务逻辑完全独立于存储和网络实现细节。

### 核心目标

1. **灵活性**：轻松更换存储后端（文件系统、数据库、云）而不改变核心逻辑
2. **可测试性**：所有组件都可以轻松模拟和独立测试
3. **性能**：利用 Rust 的特征系统实现零成本抽象
4. **可维护性**：清晰的模块边界和职责

## 设计原则

### 1. 基于特征的抽象

所有主要子系统都定义为特征，允许多种实现：

```rust
// 存储抽象示例
#[async_trait]
pub trait TrackStorage: Send + Sync {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>>;
    async fn save_track(&self, track: &Track) -> Result<()>;
    // ... 更多方法
}
```

### 2. 依赖注入

组件通过构造函数接收依赖：

```rust
// HTTP 服务器接收存储实现
pub struct AxumServer<S> {
    storage: Arc<S>,
    config: NetworkConfig,
}
```

### 3. 异步优先

所有 I/O 操作都使用 Tokio 是异步的：

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let storage = Arc::new(MemoryStorage::new());
    storage.initialize().await?;
    // ...
}
```

## 模块架构

```
┌─────────────────────────────────────────────────────────┐
│                    reverie-server                       │
│         (应用程序入口点和编排)                            │
│                                                          │
│  - 初始化存储和网络组件                                   │
│  - 连接依赖项                                            │
│  - 处理应用程序生命周期                                   │
└────────────────────┬──────────────────┬─────────────────┘
                      │                  │
          ┌───────────▼──────────┐   ┌──▼──────────────────┐
          │  reverie-network     │   │  reverie-storage    │
          │  (HTTP/网络)         │   │  (数据持久化)       │
          │                      │   │                     │
          │  - HTTP 服务器特征   │   │  - 存储特征         │
          │  - Axum 实现         │   │  - Memory 实现      │
          │  - API 处理器        │   │  - File 实现 (待完成)│
          │  - 媒体流媒体        │   │  - DB 实现 (待完成)  │
          └───────────┬──────────┘   └──┬──────────────────┘
                      │                  │
                      └──────────┬───────┘
                                 │
                     ┌───────────▼──────────┐
                     │    reverie-core      │
                     │  (域模型)            │
                     │                      │
                     │  - Track, Album      │
                     │  - Artist, User      │
                     │  - Playlist          │
                     │  - 错误类型           │
                     └──────────────────────┘
```

### reverie-core

**目的**：定义域模型和业务逻辑类型。

**核心组件**：
- `models.rs`：数据结构（Track、Album、Artist、User、Playlist）
- `error.rs`：特定领域的错误类型

**依赖项**：最小 - 只有序列化和基本工具。

**设计理念**：这个 crate 应该是纯粹的 - 没有 I/O，没有外部服务。它代表应用程序的"什么"。

### reverie-storage

**目的**：提供存储抽象和实现。

**核心组件**：
- `traits.rs`：存储特征定义
  - `TrackStorage`：曲目 CRUD 操作
  - `AlbumStorage`：专辑 CRUD 操作
  - `ArtistStorage`：艺术家 CRUD 操作
  - `UserStorage`：用户管理
  - `PlaylistStorage`：播放列表操作
  - `FileStorage`：文件操作
  - `Storage`：组合存储接口
- `memory.rs`：内存中实现（用于测试）
- `filesystem.rs`：文件系统实现（计划中）
- `database.rs`：数据库实现（计划中）

**设计理念**：存储实现与业务逻辑完全隔离。新后端可以在不接触其他代码的情况下添加。

### reverie-network

**目的**：提供网络抽象和 HTTP 服务器实现。

**核心组件**：
- `traits.rs`：网络特征定义
  - `HttpServer`：HTTP 服务器生命周期
  - `MediaStreamer`：音频流媒体
  - `ExternalConnection`：外部连接
- `axum_server.rs`：基于 Axum 的 HTTP 服务器
- `dto.rs`：用于 API 的数据传输对象
- `error.rs`：网络特定错误

**设计理念**：网络层负责处理 HTTP，但将所有数据操作委托给存储实现。

### reverie-server

**目的**：主应用程序入口点。

**核心组件**：
- `main.rs`：应用程序编排

**职责**：
1. 初始化日志和配置
2. 创建存储后端
3. 创建 HTTP 服务器
4. 连接依赖项
5. 启动应用程序

## 存储抽象

### 存储特征层次结构

```rust
// 专业化存储特征
TrackStorage    ──┐
AlbumStorage    ──┤
ArtistStorage   ──┤
UserStorage     ──┼──> Storage（组合特征）
PlaylistStorage ──┤
FileStorage     ──┘
```

### 实现策略

每个存储特征都是独立的，可以单独实现。`Storage` 特征将它们全部组合：

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

### 当前实现

#### MemoryStorage

- **用例**：测试、开发
- **存储**：使用 RwLock 的内存中 HashMap
- **特性**：快速、临时、线程安全
- **限制**：重启后数据丢失

### 未来实现

1. **FilesystemStorage**（计划中）
   - 磁盘上的音乐文件
   - SQLite 用于元数据
   - 适用于单服务器部署

2. **DatabaseStorage**（计划中）
   - PostgreSQL 或 MySQL
   - 可扩展的元数据存储
   - 支持多个实例

3. **CloudStorage**（计划中）
   - S3 兼容存储用于文件
   - 云数据库用于元数据
   - 分布式部署

## 网络抽象

### 网络特征

```rust
HttpServer          -> 服务器生命周期管理
MediaStreamer       -> 音频文件流媒体
ExternalConnection  -> 联合、云同步
RequestHandler      -> 通用请求处理
```

### Axum 实现

Axum 实现演示了如何在网络抽象上构建：

```rust
pub struct AxumServer<S> {
    storage: Arc<S>,  // 通用存储
    config: NetworkConfig,
}

impl<S> AxumServer<S>
where
    S: TrackStorage + AlbumStorage + ... + Clone + 'static
{
    fn create_router(&self) -> Router {
        // 使用存储抽象创建路由
    }
}
```

### API 设计

REST API 端点遵循面向资源的设计：

```
GET    /api/tracks           -> 列出曲目
GET    /api/tracks/:id       -> 获取曲目
GET    /api/tracks/search    -> 搜索曲目
GET    /api/albums           -> 列出专辑
GET    /api/albums/:id       -> 获取专辑
GET    /api/artists          -> 列出艺术家
GET    /api/playlists/:id    -> 获取播放列表
```

## 数据流

### 请求流程示例：获取曲目

```
1. HTTP 请求
   ↓
2. Axum 路由器
   ↓
3. get_track_handler<S>
   ↓
4. TrackStorage::get_track（特征方法）
   ↓
5. MemoryStorage::get_track（实现）
   ↓
6. 返回 Track 模型
   ↓
7. 转换为 TrackResponse DTO
   ↓
8. 序列化为 JSON
   ↓
9. HTTP 响应
```

### 关键点

- **处理器是泛型的**：适用于任何 `TrackStorage` 实现
- **DTO 分离**：API 类型（TrackResponse）与域类型（Track）分离
- **类型安全**：Rust 的类型系统确保正确使用

## 扩展系统

### 添加新的存储后端

示例：PostgreSQL 存储

```rust
// 1. 创建新模块
// reverie-storage/src/postgres.rs

use sqlx::PgPool;

pub struct PostgresStorage {
    pool: PgPool,
}

// 2. 实现存储特征
#[async_trait]
impl TrackStorage for PostgresStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        // PostgreSQL 实现
        let track = sqlx::query_as!(
            Track,
            "SELECT * FROM tracks WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(track)
    }
    // ... 实现其他方法
}

// 3. 更新 Cargo.toml 特性
[features]
postgres = ["sqlx/postgres"]

// 4. 在服务器中使用
let storage = Arc::new(PostgresStorage::new(pool));
```

### 添加新的 HTTP 服务器实现

示例：Actix-Web 服务器

```rust
// 1. 创建新模块
// reverie-network/src/actix_server.rs

pub struct ActixServer<S> {
    storage: Arc<S>,
    config: NetworkConfig,
}

// 2. 实现 HttpServer 特征
#[async_trait]
impl<S> HttpServer for ActixServer<S>
where
    S: TrackStorage + AlbumStorage + ...
{
    async fn start(&self, addr: SocketAddr) -> Result<()> {
        // Actix 实现
    }
    // ... 实现其他方法
}

// 3. 更新 Cargo.toml 特性
[features]
actix-server = ["actix-web"]

// 4. 在服务器中使用
let server = ActixServer::new(storage, config);
```

### 添加新的 API 端点

```rust
// 1. 添加处理器函数
async fn create_track_handler<S>(
    State(state): State<AppState<S>>,
    Json(request): Json<CreateTrackRequest>,
) -> impl IntoResponse
where
    S: TrackStorage + Clone + Send + Sync + 'static,
{
    // 实现
}

// 2. 在 create_router 中添加路由
.route("/api/tracks", post(create_track_handler::<S>))
```

## 测试策略

### 单元测试

- 独立测试存储实现
- 测试域模型
- 测试工具函数

### 集成测试

- 使用真实数据库测试存储（Docker 容器）
- 使用测试存储测试 API 端点
- 测试完整的请求/响应周期

### 测试示例

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

## 性能注意事项

### 存储

- **内存**：快速但受 RAM 限制
- **文件系统**：平衡速度和持久性
- **数据库**：连接池、查询优化
- **缓存**：考虑添加缓存层

### 网络

- **流媒体**：高效的文件流媒体，无需加载整个文件
- **转码**：按需转码以优化带宽
- **连接池**：重用数据库连接

### 并发

- 所有存储操作都是线程安全的
- 使用 `Arc` 进行共享存储访问
- `RwLock` 用于读取密集型工作负载

## 安全注意事项

### 存储

- 存储操作前的输入验证
- SQL 注入预防（参数化查询）
- 文件路径验证以防止目录遍历

### 网络

- 身份验证中间件（计划中）
- 速率限制（计划中）
- CORS 配置
- 所有端点的输入验证

## 结论

Reverie 的架构通过基于特征的抽象优先考虑灵活性和可扩展性。域逻辑、存储和网络层之间的清晰分离使其易于：

1. **测试**：模拟实现以进行单元测试
2. **扩展**：添加新的存储/网络实现
3. **维护**：清晰的模块边界
4. **扩展**：根据部署需求更换实现

该设计遵循 Rust 最佳实践，并利用类型系统确保正确性和安全性。
