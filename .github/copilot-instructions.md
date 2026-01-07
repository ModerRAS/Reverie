# Reverie 项目 AI 编码指南

## 项目概览

Reverie 是一个用 Rust 编写的音乐流媒体服务器，兼容 Subsonic API。项目采用 **trait-based 抽象** 架构，将存储和网络层完全解耦。

## 架构核心

### Workspace 结构 (4 个 crate)

```
reverie-core      → 纯领域模型 (Track, Album, Artist, User, Playlist)，无 I/O
reverie-storage   → 存储抽象层 (traits.rs 定义接口，memory.rs 为测试实现)
reverie-network   → HTTP 服务层 (Axum 实现 + Subsonic API 端点)
reverie-server    → 应用入口，组装各组件
```

**依赖方向**: `server` → `network`/`storage` → `core`

### 关键设计模式

1. **Trait 抽象** - 所有存储操作通过 trait 定义 (`TrackStorage`, `AlbumStorage` 等在 [reverie-storage/src/traits.rs](reverie-storage/src/traits.rs))
2. **依赖注入** - 服务器通过泛型接收存储实现: `AxumServer<S: TrackStorage + AlbumStorage + ...>`
3. **Arc 共享** - 存储实例用 `Arc<S>` 包装后传入网络层

```rust
// 典型组装模式 (见 reverie-server/src/main.rs)
let storage = Arc::new(MemoryStorage::new());
storage.initialize().await?;
let server = AxumServer::new(storage.clone(), network_config);
server.start(addr).await?;
```

## 开发命令

```bash
cargo build                    # 构建所有 crate
cargo test                     # 运行所有测试 (44 tests)
cargo run -p reverie-server    # 启动服务器 (端口 4533)
cargo run --example simple_server  # 带示例数据启动
```

## 添加新功能指南

### 新增存储类型 (如 PostgreSQL)

1. 在 `reverie-storage/src/` 创建新模块 (如 `postgres.rs`)
2. 实现所有 `*Storage` traits (`TrackStorage`, `AlbumStorage` 等)
3. 实现 `Storage` trait 的 `initialize()` 方法
4. 在 `lib.rs` 导出新模块

### 新增 API 端点

1. Subsonic API: 在 [reverie-network/src/subsonic/mod.rs](reverie-network/src/subsonic/mod.rs) 添加 handler + route
2. REST API: 在 [reverie-network/src/axum_server.rs](reverie-network/src/axum_server.rs) 的 `create_router()` 添加
3. 测试在对应的 `tests.rs` 中添加

### 新增领域模型

1. 在 [reverie-core/src/models.rs](reverie-core/src/models.rs) 定义 struct (需 `Serialize`, `Deserialize`)
2. 在 `reverie-storage/traits.rs` 添加对应 trait
3. 在各存储实现中实现新 trait

## 错误处理约定

- 每个 crate 有独立的 `error.rs`，使用 `thiserror` 定义错误类型
- 导出 `Result<T>` 类型别名: `pub type Result<T> = std::result::Result<T, XxxError>;`
- 跨 crate 边界使用 `anyhow` 转换

## 测试模式

```rust
// 存储测试模式 (见 reverie-storage/tests/)
#[tokio::test]
async fn test_xxx() {
    let storage = MemoryStorage::new();
    storage.initialize().await.unwrap();
    // ... 测试操作
}

// API 测试模式 (见 reverie-network/src/subsonic/tests.rs)
#[tokio::test]
async fn test_endpoint() {
    let router = create_router();
    let response = router
        .oneshot(Request::builder().uri("/endpoint").body(Body::empty()).unwrap())
        .await.unwrap();
    assert_eq!(response.status(), 200);
}
```

## 重要约定

- 所有 I/O 操作必须 `async`，使用 `#[async_trait]` 宏
- 存储 trait 必须包含 `Send + Sync` bounds
- UUID 使用 `uuid::Uuid`，时间戳使用 `chrono::DateTime<Utc>`
- Subsonic API 响应格式为 XML，版本号 `1.16.1`
