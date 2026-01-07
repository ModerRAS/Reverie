# Reverie

一个用 Rust 编写的现代、轻量级音乐流媒体服务器，灵感来自 [Navidrome](https://github.com/navidrome/navidrome)。Reverie 具有完全抽象的存储和网络层，可以轻松更换实现并扩展功能。

## 核心特性

- **抽象存储层**：可插拔的存储后端（文件系统、数据库、云存储）
- **抽象网络层**：灵活的 HTTP 服务器实现和外部连接
- **类型安全**：使用 Rust 构建，确保内存安全和性能
- **异步优先**：完全异步，使用 Tokio
- **模块化架构**：域逻辑、存储和网络层之间清晰分离
- **Subsonic API 兼容**：完全兼容 Subsonic API，可与任何 Subsonic 客户端配合使用
- **Web UI（可选）**：`reverie-ui` 提供类 Navidrome 的 Web 界面（Dioxus 构建）

## 当前开发状态

### 已实现的功能

**核心功能**
- ✅ 核心域模型（Track、Album、Artist、User、Playlist）
- ✅ 存储抽象层（SubsonicStorage trait）
- ✅ 内存存储实现
- ✅ Axum HTTP 服务器
- ✅ Subsonic API 端点（36 个端点，44 个测试）

**Subsonic API 端点（已完成）**

基础端点：
- ✅ `/ping` - 健康检查
- ✅ `/getLicense` - 获取许可证信息
- ✅ `/getMusicFolders` - 获取音乐文件夹
- ✅ `/getGenres` - 获取流派列表
- ✅ `/getScanStatus` - 获取扫描状态
- ✅ `/startScan` - 开始扫描

浏览端点：
- ✅ `/getIndexes` - 获取艺术家索引
- ✅ `/getArtists` - 获取艺术家列表
- ✅ `/getMusicDirectory` - 获取音乐目录
- ✅ `/getArtist` - 获取艺术家详情
- ✅ `/getAlbum` - 获取专辑详情
- ✅ `/getSong` - 获取歌曲详情
- ✅ `/getAlbumInfo` - 获取专辑信息
- ✅ `/getArtistInfo` - 获取艺术家信息

搜索端点：
- ✅ `/search2` - 搜索（返回 Artist/Album/Child）
- ✅ `/search3` - 搜索（返回 ID3 版本）

播放列表端点：
- ✅ `/getPlaylists` - 获取播放列表
- ✅ `/getPlaylist` - 获取播放列表详情
- ✅ `/createPlaylist` - 创建播放列表
- ✅ `/updatePlaylist` - 更新播放列表
- ✅ `/deletePlaylist` - 删除播放列表

用户端点：
- ✅ `/getUser` - 获取用户信息
- ✅ `/getUsers` - 获取所有用户

流媒体端点：
- ✅ `/stream` - 流媒体传输
- ✅ `/download` - 下载
- ✅ `/getCoverArt` - 获取封面图
- ✅ `/getAvatar` - 获取用户头像

收藏与评分：
- ✅ `/getStarred` - 获取收藏内容
- ✅ `/getStarred2` - 获取收藏内容（ID3 版本）
- ✅ `/star` - 收藏
- ✅ `/unstar` - 取消收藏
- ✅ `/setRating` - 设置评分

播放记录：
- ✅ `/scrobble` - 播放记录
- ✅ `/getNowPlaying` - 当前播放
- ✅ `/getRandomSongs` - 随机歌曲
- ✅ `/getLyrics` - 获取歌词

### 测试覆盖

- ✅ 36 个 Subsonic API 测试
- ✅ 8 个内存存储测试
- ✅ 总计 44 个测试全部通过

## 架构

Reverie 由多个职责明确的 crate 组成：

### 核心模块

```
reverie/
├── reverie-core/       # 域模型和业务逻辑
├── reverie-storage/    # 存储抽象层
├── reverie-network/    # 网络抽象层
└── reverie-server/     # 主应用程序服务器
└── reverie-ui/         # 可选 Web UI（Dioxus）
```

### 架构图

```
┌─────────────────────────────────────────────────────────┐
│                    reverie-server                       │
│              (应用程序编排)                              │
└────────────────────┬──────────────────┬─────────────────┘
                      │                  │
          ┌───────────▼──────────┐   ┌──▼──────────────────┐
          │  reverie-network     │   │  reverie-storage    │
          │  (HTTP/网络)         │   │  (数据持久化)       │
          └───────────┬──────────┘   └──┬──────────────────┘
                      │                  │
                      └──────────┬───────┘
                                 │
                     ┌───────────▼──────────┐
                     │    reverie-core      │
                     │  (域模型)            │
                     └──────────────────────┘
```

### 存储抽象

存储层为不同类型的操作提供特征：

**通用存储特征：**
- **TrackStorage**：管理音乐曲目
- **AlbumStorage**：管理专辑
- **ArtistStorage**：管理艺术家
- **UserStorage**：管理用户
- **PlaylistStorage**：管理播放列表
- **FileStorage**：管理文件操作（音频文件、封面艺术）
- **Storage**：组合所有存储特征

**Subsonic API 存储特征：**
- **SubsonicStorage**：Subsonic API 所需的全部存储操作
  - `get_music_folders()` - 获取音乐文件夹
  - `get_artist_indexes()` - 获取艺术家索引
  - `get_directory()` - 获取目录
  - `get_artist()`, `get_album()`, `get_song()` - 获取详情
  - `search()` - 搜索
  - `get_playlists()`, `create_playlist()`, `delete_playlist()` - 播放列表管理
  - `get_starred()`, `star()`, `unstar()` - 收藏功能
  - `scrobble()` - 播放记录
  - 以及更多...

**可用的实现：**
- ✅ 内存中（用于测试/开发）
- ✅ 文件系统存储（带元数据缓存）
- 🚧 SQLite 集成（计划中）
- 🚧 PostgreSQL（计划中）
- 🚧 S3 兼容存储（计划中）

### 网络抽象

网络层提供 HTTP 服务和外部连接的特征：

**HTTP 服务器特征：**
- **HttpServer**：HTTP 服务器实现
- **MediaStreamer**：带可选转码的音频流媒体
- **ExternalConnection**：外部网络连接（联合、云同步）

**Subsonic API 端点：**
网络层实现所有 Subsonic API 端点，返回标准 XML 格式响应：
- 基础信息端点（ping、license、folders、genres）
- 浏览端点（directory、artist、album、song）
- 搜索端点（search2、search3）
- 播放列表端点（playlist CRUD）
- 流媒体端点（stream、download、coverArt）
- 用户端点（getUser、getUsers）
- 收藏端点（star、unstar、getStarred）
- 播放记录端点（scrobble、getNowPlaying）

**可用的实现：**
- ✅ 基于 Axum 的 HTTP 服务器（支持 Subsonic API）
- 🚧 转码支持（计划中）
- 🚧 联合协议（计划中）

## 快速开始

### 前置条件

- Rust 1.70 或更高版本
- Cargo

### 构建

```bash
# 构建所有 crate
cargo build --release

# 仅构建服务器
cargo build --release -p reverie-server
```

### 运行

```bash
# 运行服务器
cargo run --release -p reverie-server

# 或运行编译后的二进制文件
./target/release/reverie
```

服务器默认在 `http://127.0.0.1:4533` 启动。

### 运行 Web UI（可选）

Web UI 是独立 crate（`reverie-ui`），通过 Subsonic API（`/rest`）与后端通信。

前置条件：

```bash
cargo install dioxus-cli
```

启动：

```bash
cd reverie-ui
dx serve
```

然后打开 `http://localhost:8080`。开发模式下，UI 会将 `/rest` 代理到 `http://127.0.0.1:4533/rest`。

### 更多文档

更详细的架构/总结等文档在 `Docs/` 目录下。

### Subsonic API 端点

Reverie 完全兼容 Subsonic API（版本 1.16.1），可与任何 Subsonic 客户端配合使用。

**基础端点：**
- `GET /rest/ping` - 健康检查
- `GET /rest/getLicense` - 获取许可证信息
- `GET /rest/getMusicFolders` - 获取音乐文件夹列表
- `GET /rest/getGenres` - 获取流派列表
- `GET /rest/getScanStatus` - 获取扫描状态
- `GET /rest/startScan` - 开始扫描

**浏览端点：**
- `GET /rest/getIndexes` - 获取艺术家索引
- `GET /rest/getArtists` - 获取完整艺术家列表
- `GET /rest/getMusicDirectory` - 获取音乐目录
- `GET /rest/getArtist` - 获取艺术家详情
- `GET /rest/getAlbum` - 获取专辑详情
- `GET /rest/getSong` - 获取歌曲详情
- `GET /rest/getAlbumInfo` - 获取专辑信息
- `GET /rest/getArtistInfo` - 获取艺术家信息

**搜索端点：**
- `GET /rest/search2` - 搜索（返回 Artist/Album/Child）
- `GET /rest/search3` - 搜索（返回 ID3 版本）

**播放列表端点：**
- `GET /rest/getPlaylists` - 获取所有播放列表
- `GET /rest/getPlaylist` - 获取播放列表详情
- `GET /rest/createPlaylist` - 创建播放列表
- `GET /rest/updatePlaylist` - 更新播放列表
- `GET /rest/deletePlaylist` - 删除播放列表

**用户端点：**
- `GET /rest/getUser` - 获取用户信息
- `GET /rest/getUsers` - 获取所有用户

**流媒体端点：**
- `GET /rest/stream` - 流媒体传输
- `GET /rest/download` - 下载音频文件
- `GET /rest/getCoverArt` - 获取封面图片
- `GET /rest/getAvatar` - 获取用户头像

**收藏与评分：**
- `GET /rest/getStarred` - 获取收藏内容
- `GET /rest/getStarred2` - 获取收藏内容（ID3 版本）
- `GET /rest/star` - 收藏
- `GET /rest/unstar` - 取消收藏
- `GET /rest/setRating` - 设置评分

**播放记录：**
- `GET /rest/scrobble` - 记录播放
- `GET /rest/getNowPlaying` - 获取当前播放
- `GET /rest/getRandomSongs` - 获取随机歌曲
- `GET /rest/getLyrics` - 获取歌词

## 开发

### 项目结构

每个 crate 都有特定用途：

**reverie-core**：域模型和共享类型
- 无外部依赖（除了序列化）
- 纯数据结构
- 业务逻辑类型

**reverie-storage**：存储抽象和实现
- 存储操作的特征定义
- 多个后端实现
- 异步优先 API

**reverie-network**：网络抽象和 HTTP 服务器
- HTTP 服务器特征和实现
- API 处理器和路由
- 外部连接抽象

**reverie-server**：主应用程序
- 将存储和网络层连接起来
- 配置管理
- 应用程序启动和生命周期

### 添加新的存储后端

1. 在 `reverie-storage/src/` 中实现存储特征
2. 在 `reverie-storage/Cargo.toml` 的特性中添加你的实现
3. 更新服务器以使用你的存储后端

示例：

```rust
use async_trait::async_trait;
use reverie_storage::{TrackStorage, Result};

pub struct MyCustomStorage { /* ... */ }

#[async_trait]
impl TrackStorage for MyCustomStorage {
    async fn get_track(&self, id: Uuid) -> Result<Option<Track>> {
        // 你的实现
    }
    // ... 实现其他方法
}
```

### 添加新的 HTTP 服务器

1. 在 `reverie-network/src/` 中实现 `HttpServer` 特征
2. 将你的实现添加到特性中
3. 更新服务器以使用你的 HTTP 实现

## 设计原则

1. **抽象优先**：核心逻辑独立于实现细节
2. **依赖注入**：实现时在运行时注入
3. **类型安全**：利用 Rust 的类型系统确保正确性
4. **可测试性**：轻松模拟存储和网络层
5. **性能**：Rust 特征系统的零成本抽象
6. **可扩展性**：无需修改核心逻辑即可轻松添加新功能

## 与 Navidrome 的比较

| 特性 | Navidrome | Reverie |
|---------|-----------|---------|
| 语言 | Go | Rust |
| 存储 | SQLite | 抽象（多后端） |
| 网络 | 内置 HTTP | 抽象（可插拔服务器） |
| API | Subsonic API | 自定义 REST API |
| 转码 | FFmpeg | 计划中 |
| 联合 | 否 | 计划中 |

## 路线图

### 已完成 ✅

- [x] 核心域模型（Track、Album、Artist、User、Playlist）
- [x] Subsonic 兼容的数据结构（MediaFile、SubsonicAlbum、SubsonicArtist 等）
- [x] 存储抽象层（所有 Storage traits）
- [x] SubsonicStorage trait（完整的 Subsonic API 存储接口）
- [x] 内存存储实现（MemoryStorage）
- [x] 文件系统存储实现（FileSystemStorage）
  - VFS 启发的架构（Inode、DirEntry、FileHandle）
  - 使用迭代扫描的目录遍历
  - 基于 HashMap 的元数据缓存
  - 支持常见音频格式（MP3、FLAC、M4A、OGG 等）
- [x] Axum HTTP 服务器实现
- [x] Subsonic API 基础端点（ping、license、folders、genres、scan）
- [x] Subsonic API 浏览端点（directory、artist、album、song）
- [x] Subsonic API 搜索端点（search2、search3）
- [x] Subsonic API 播放列表端点（CRUD）
- [x] Subsonic API 用户端点
- [x] Subsonic API 流媒体端点（stream、download、coverArt）
- [x] Subsonic API 收藏与评分端点
- [x] Subsonic API 播放记录端点
- [x] 完整的测试覆盖（44 个测试）

### 开发中 🚧

- [ ] SQLite 元数据集成（用于文件系统存储）
- [ ] 音乐库扫描器（音频元数据提取）
- [ ] 音频流媒体实现
- [ ] 转码支持（FFmpeg 集成）

### 计划中 📋

- [ ] 用户认证系统
- [ ] 数据库迁移
- [ ] 配置文件支持
- [ ] Docker 支持
- [ ] 联合/云同步
- [ ] 更多存储后端（PostgreSQL、S3）

## 贡献

欢迎贡献！模块化架构使其易于贡献：

- 添加新的存储后端
- 添加新的网络实现
- 改进现有实现
- 添加测试
- 改进文档

## 许可证

本项目基于 MIT 许可证授权 - 有关详细信息，请参阅 [LICENSE](LICENSE) 文件。

## 致谢

灵感来自 [Navidrome](https://github.com/navidrome/navidrome) - 一个出色的音乐流媒体服务器，证明了这一概念。
