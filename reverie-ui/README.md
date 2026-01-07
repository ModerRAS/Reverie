# Reverie UI

用 Dioxus 构建的 Reverie 音乐服务器 Web 界面，功能和排版参考 Navidrome。

## 功能特性

### 已实现的页面
- ✅ **主页** - 最近播放、最近添加、快速访问
- ✅ **专辑页面** - 网格视图，支持多种排序方式（最近添加、最近播放、播放最多等）
- ✅ **艺术家页面** - 圆形头像网格视图
- ✅ **歌曲页面** - 表格视图，显示所有曲目
- ✅ **播放列表页面** - 播放列表管理
- ✅ **收藏页面** - 已收藏的歌曲、专辑、艺术家
- ✅ **搜索页面** - 全局搜索
- ✅ **设置页面** - 主题、语言、播放设置
- ✅ **登录页面** - 用户认证

### 详情页面
- ✅ **专辑详情** - 专辑信息 + 曲目列表
- ✅ **艺术家详情** - 艺术家信息 + 热门歌曲 + 专辑列表
- ✅ **播放列表详情** - 播放列表信息 + 曲目列表

### UI 组件
- ✅ **布局组件** - 侧边栏、顶部导航栏、主内容区
- ✅ **播放器栏** - 底部播放控制、进度条、音量控制
- ✅ **卡片组件** - 专辑卡片、艺术家卡片、播放列表卡片
- ✅ **列表组件** - 曲目列表、紧凑列表
- ✅ **通用组件** - 加载动画、空状态、分页、模态框

### 状态管理
- ✅ **认证状态** - 登录/登出
- ✅ **播放器状态** - 播放/暂停、队列、音量、循环模式
- ✅ **UI 状态** - 侧边栏、当前视图、搜索

## 技术栈

- **Dioxus 0.6** - Rust 全栈 UI 框架
- **Tailwind CSS** - 样式
- **Subsonic API** - 与服务器通信

## 开发

### 前置条件

```bash
# 安装 Dioxus CLI
cargo install dioxus-cli
```

### 运行开发服务器

```bash
cd reverie-ui
dx serve
```

这将启动开发服务器，默认地址 `http://localhost:8080`

### 构建生产版本

```bash
dx build --release
```

## 项目结构

```
reverie-ui/
├── Cargo.toml           # 依赖配置
├── Dioxus.toml          # Dioxus 配置
├── assets/
│   └── tailwind.css     # Tailwind 样式
└── src/
    ├── main.rs          # 入口点
    ├── lib.rs           # 库导出
    ├── api/
    │   └── mod.rs       # Subsonic API 客户端
    ├── components/
    │   ├── mod.rs       # 组件导出
    │   ├── layout.rs    # 布局组件
    │   ├── player.rs    # 播放器组件
    │   ├── common.rs    # 通用组件
    │   ├── cards.rs     # 卡片组件
    │   └── lists.rs     # 列表组件
    ├── pages/
    │   ├── mod.rs       # 页面导出
    │   ├── home.rs      # 主页
    │   ├── albums.rs    # 专辑页面
    │   ├── artists.rs   # 艺术家页面
    │   ├── songs.rs     # 歌曲页面
    │   ├── playlists.rs # 播放列表页面
    │   ├── favorites.rs # 收藏页面
    │   ├── search.rs    # 搜索页面
    │   ├── settings.rs  # 设置页面
    │   ├── login.rs     # 登录页面
    │   ├── album_detail.rs   # 专辑详情
    │   ├── artist_detail.rs  # 艺术家详情
    │   └── playlist_detail.rs # 播放列表详情
    ├── routes.rs        # 路由配置
    └── state/
        └── mod.rs       # 全局状态管理
```

## 与 Navidrome UI 对比

| 功能 | Navidrome | Reverie UI |
|------|-----------|------------|
| 框架 | React + react-admin | Dioxus (Rust) |
| 样式 | Material-UI | Tailwind CSS |
| 状态 | Redux | Dioxus Signals |
| 专辑视图 | ✅ | ✅ |
| 艺术家视图 | ✅ | ✅ |
| 歌曲视图 | ✅ | ✅ |
| 播放列表 | ✅ | ✅ |
| 收藏 | ✅ | ✅ |
| 搜索 | ✅ | ✅ |
| 播放器 | ✅ | ✅ |
| 主题切换 | ✅ | ✅ (设置中) |
| 多语言 | ✅ | 🚧 计划中 |
| 拖放排序 | ✅ | 🚧 计划中 |
| 键盘快捷键 | ✅ | 🚧 计划中 |

## 截图

> 注: 需要运行项目后查看

- 深色主题
- 响应式设计（支持移动端）
- 现代化 UI

## License

MIT
