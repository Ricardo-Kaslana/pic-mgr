# pic-mgr — 本地图片管理器 设计规格说明

**日期:** 2026-06-07
**状态:** 已确认

---

## 1. 概述

一个轻量、现代、美观、高性能的本地图片管理器。用户将某个文件夹设为"工作区"，应用为该文件夹下所有图片生成索引，每张图片可记录标签、创建时间、更新时间、备注等信息。所有对工作区图片的处理都不实际影响工作区里的原始图片文件。

### 1.1 核心原则

1. **工作区图片只读** — Rust 后端只读取文件系统里的图片，所有元数据修改（标签、备注、评分）只写入 SQLite 数据库，永不触碰原始文件。
2. **Rust 管数据，Vue 管展示** — 图片列表、搜索、缩略图生成全部通过 Tauri command 由 Rust 处理。Vue 负责筛选状态、排序、UI 交互。
3. **事件驱动缩略图进度** — 后台预生成缩略图通过 Tauri events 推送进度到前端，前端只展示不控制生成逻辑。

---

## 2. 技术栈

| 层 | 选型 | 理由 |
|---|------|------|
| 桌面框架 | **Tauri 2.x** | 轻量(~10MB)、Rust 后端、原生文件系统访问 |
| 前端框架 | **Vue 3** + Composition API | 中文社区强、单文件组件直观、生态成熟 |
| 构建工具 | **Vite** | 快速 HMR、原生 ES 模块 |
| 状态管理 | **Pinia** | Vue 3 官方推荐、TypeScript 友好 |
| 路由 | **Vue Router 4** | SPA 路由标准方案 |
| 数据库 | **SQLite** (rusqlite + FTS5) | 单文件、零配置、全文搜索支持 |
| 图片处理 | **image-rs** (Rust) | 主流格式全覆盖、缩略图生成 |
| 文件遍历 | **walkdir** (Rust) | 递归目录遍历 |
| 哈希 | **sha2** (Rust) | SHA-256 文件去重/变更检测 |
| 异步 | **tokio** (Rust) | 后台缩略图队列 |
| 序列化 | **serde + serde_json** | Tauri command 参数/返回值 |

---

## 3. 架构

```
┌─────────────────────────────────────────────────────────┐
│                    Vue Frontend                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐ │
│  │ Thumbnail │ │  Detail  │ │  Search  │ │ Workspace │ │
│  │  Grid     │ │  Panel   │ │  Bar     │ │  Manager  │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └─────┬─────┘ │
│       └─────────────┴────────────┴─────────────┘       │
│                         │                               │
│              Pinia Stores (images, tags, ui)            │
└─────────────────────────┬───────────────────────────────┘
                          │  invoke() + listen()
┌─────────────────────────┬───────────────────────────────┐
│                    Rust Backend                         │
│  ┌──────────┐ ┌────────────┐ ┌──────────┐ ┌─────────┐ │
│  │  FS      │ │ Thumbnail  │ │  SQLite  │ │  Event  │ │
│  │  Walker  │ │  Pipeline  │ │  Layer   │ │  Bus    │ │
│  └──────────┘ └────────────┘ └──────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 3.1 Rust 后端模块结构

```
src-tauri/src/
├── main.rs          — Tauri 入口
├── commands/
│   ├── workspace.rs — create/list/delete 工作区
│   ├── scan.rs      — scan_workspace, rescan_workspace
│   ├── images.rs    — get_images, get_image_detail, search_images
│   ├── metadata.rs  — update_notes, update_rating, batch_update_tags
│   ├── tags.rs      — get_tags, create_tag, delete_tag
│   ├── thumbnail.rs — get_thumbnail, generate_thumbnail
│   └── export.rs    — export_metadata
├── db/
│   ├── mod.rs       — 数据库初始化、连接池
│   ├── schema.rs    — DDL 迁移
│   └── queries.rs   — 查询封装
├── scanner/
│   ├── mod.rs       — 目录遍历、增量检测
│   └── thumbnail.rs — 缩略图混合管线
└── models.rs        — 共享数据结构 (Serialize/Deserialize)
```

### 3.2 Vue 前端组件结构

```
src/
├── App.vue
├── router/
│   └── index.ts
├── stores/
│   ├── workspace.ts
│   ├── images.ts
│   ├── tags.ts
│   ├── selection.ts
│   ├── thumbnail.ts
│   └── settings.ts
├── views/
│   ├── WorkspaceHome.vue    — / (选择或创建工作区)
│   ├── BrowserView.vue      — /workspace/:id (主浏览视图)
│   └── SettingsPage.vue     — /settings
├── components/
│   ├── layout/
│   │   └── AppLayout.vue    — 壳：侧栏 + 内容区
│   ├── workspace/
│   │   ├── WorkspaceCard.vue
│   │   └── CreateWorkspaceDialog.vue
│   ├── browser/
│   │   ├── SidebarPanel.vue
│   │   ├── TagFilter.vue
│   │   ├── DateRangeFilter.vue
│   │   ├── SearchBar.vue
│   │   ├── ThumbnailGrid.vue
│   │   ├── ThumbnailCard.vue
│   │   └── DetailPanel.vue
│   ├── detail/
│   │   ├── ImagePreview.vue
│   │   ├── MetadataEditor.vue
│   │   └── ImageInfo.vue
│   └── common/
│       ├── VirtualScroller.vue
│       └── TagBadge.vue
└── types/
    └── index.ts
```

---

## 4. 数据模型 (SQLite)

### 4.1 表结构

```sql
CREATE TABLE workspaces (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE images (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    relative_path   TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    file_size       INTEGER,          -- 字节
    file_hash       TEXT,             -- SHA-256 hex
    width           INTEGER,          -- 像素
    height          INTEGER,          -- 像素
    format          TEXT,             -- jpg/png/gif/webp/bmp/tiff/raw
    thumbnail_path  TEXT,             -- 缓存路径, NULL = 未生成
    notes           TEXT DEFAULT '',
    rating          INTEGER DEFAULT 0 CHECK(rating >= 0 AND rating <= 5),
    created_at      TEXT,             -- 文件修改时间 (fallback)
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    indexed_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_viewed_at  TEXT,
    UNIQUE(workspace_id, relative_path)
);

CREATE TABLE tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE,
    color TEXT  -- hex color, e.g. '#ff6b6b'
);

CREATE TABLE image_tags (
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (image_id, tag_id)
);

-- FTS5 全文搜索虚拟表 (搜索 file_name + notes)
CREATE VIRTUAL TABLE images_fts USING fts5(
    file_name,
    notes,
    content='images',
    content_rowid='id'
);
```

### 4.2 关键设计决策

| 决策 | 说明 |
|------|------|
| `relative_path` 而非绝对路径 | 工作区文件夹移动后只需更新 `workspaces.path`，不用逐行更新 `images` 表 |
| `file_hash` SHA-256 | 去重检测 + 检测文件是否被外部替换 |
| `thumbnail_path` 指向应用缓存目录 | 缩略图不在工作区里，与原始图片完全隔离 |
| `image_tags` 联结表 | 标准化多对多，避免逗号分隔字符串的查询问题 |
| `images_fts` FTS5 表 | 支持文件名和备注的全文搜索，性能优于 `LIKE '%...%'` |
| `indexed_at` 与文件时间解耦 | 索引时间与 EXIF/文件系统的创建时间分离，避免混淆 |

---

## 5. Tauri Commands API

### 5.1 工作区

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `create_workspace` | name, path | Workspace | 创建新工作区，触发初始扫描 |
| `list_workspaces` | — | Vec\<Workspace\> | 列出所有已创建的工作区 |
| `delete_workspace` | id | () | 删除工作区及其索引（不删原始文件） |

### 5.2 索引

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `scan_workspace` | id | ScanProgress | 增量扫描新增/修改的文件 |
| `rescan_workspace` | id | ScanProgress | 强制全量重新扫描 |

### 5.3 图片查询

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `get_images` | workspace_id, filters, sort, page | ImagePage | 分页查询，支持按标签/时间/格式/评分筛选 |
| `get_image_detail` | image_id | ImageDetail | 单张图片完整信息 |
| `search_images` | workspace_id, query, page | ImagePage | FTS5 全文搜索文件名和备注 |

### 5.4 元数据编辑

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `update_notes` | image_id, notes | () | 更新备注 |
| `update_rating` | image_id, rating | () | 更新评分 (1-5) |
| `batch_update_tags` | image_ids, tag_ids, action | () | 批量添加/移除/设置标签 |

### 5.5 标签

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `get_tags` | workspace_id | Vec\<TagWithCount\> | 标签列表 + 使用计数 |
| `create_tag` | workspace_id, name, color | Tag | 创建新标签 |
| `delete_tag` | tag_id | () | 删除标签 |

### 5.6 缩略图

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `get_thumbnail` | image_id | Vec\<u8\> | 返回已缓存的缩略图字节 (WebP) |
| `generate_thumbnail` | image_id | () | 按需立即生成缩略图 |

### 5.7 导出

| Command | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `export_metadata` | workspace_id, format | String | 导出为 JSON 或 CSV |

---

## 6. 缩略图混合管线

```
scan_workspace()
      │
      ▼
┌──────────────────────────┐
│  找到所有图片路径         │
│  (过滤: jpg/png/gif/    │
│   webp/bmp/tiff/raw)    │
└──────────┬───────────────┘
           │
           ▼
┌──────────────────────────────────────┐
│  分两路：                             │
│                                      │
│  A. 后台队列 (tokio::spawn)           │
│  ┌────────────────────────────────┐  │
│  │ 逐批生成缩略图 (每批16张,       │  │
│  │ 间隔50ms)                      │  │
│  │ emit thumbnail_progress event  │  │
│  │ 写入 app_data/thumbnails/      │  │
│  └────────────────────────────────┘  │
│                                      │
│  B. 按需优先                         │
│  ┌────────────────────────────────┐  │
│  │ 用户浏览到某张图 → 跳到队列最前 │  │
│  │ → 立即生成 → 返回给前端         │  │
│  └────────────────────────────────┘  │
└──────────┬───────────────────────────┘
           │
           ▼
┌──────────────────────────┐
│ image-rs crate           │
│ 统一尺寸: 300px (可配置) │
│ 格式: WebP (质量 80%)    │
│ 缓存到 app_data/thumbnails/│
└──────────────────────────┘
```

### 6.1 扫描时支持的图片格式

| 格式 | 扩展名 | 说明 |
|------|--------|------|
| JPEG | .jpg, .jpeg | 最常用 |
| PNG | .png | 透明支持 |
| GIF | .gif | 取首帧做缩略图 |
| WebP | .webp | 现代格式 |
| BMP | .bmp | 传统格式 |
| TIFF | .tiff, .tif | 专业摄影 |
| RAW (基础) | .cr2, .nef, .arw | 缩略图生成可能需要平台相关库 |

---

## 7. 前端路由 & 状态管理

### 7.1 路由

| 路径 | 视图 | 说明 |
|------|------|------|
| `/` | WorkspaceHome | 默认页：选择或创建工作区 |
| `/workspace/:id` | BrowserView | 主浏览视图（核心页面） |
| `/settings` | SettingsPage | 应用设置 |

### 7.2 Pinia Stores

| Store | 职责 |
|-------|------|
| `useWorkspaceStore` | 当前工作区 ID、名称、路径、图片总数 |
| `useImageStore` | 图片列表（分页/虚拟滚动）、筛选条件、排序方式 |
| `useTagStore` | 标签库、颜色、使用计数 |
| `useSelectionStore` | 当前选中图片集合（多选批量操作） |
| `useThumbnailStore` | 缩略图生成进度、已缓存列表 |
| `useSettingsStore` | 用户偏好（主题、缩略图大小、语言等） |

### 7.3 关键 UI 行为

- **虚拟滚动** — 图片上万时只渲染可视区域的缩略图卡片，保持 60fps 滚动
- **渐进加载** — 先显示占位符，缩略图加载完成后替换；浏览到的图片优先
- **DetailPanel 可调** — 右侧详情面板可拖拽调整宽度或完全收起
- **暗色主题默认** — 图片管理器天然适合暗色背景，让图片更突出
- **键盘快捷键** — 方向键浏览、Enter 全屏预览、T 编辑标签、N 编辑备注、Ctrl+A 全选、Escape 取消

---

## 8. 功能范围 (Scope B — 进阶版)

| 功能 | 描述 |
|------|------|
| ✅ 工作区管理 | 创建/删除/切换多个工作区 |
| ✅ 增量索引 | 首次全量扫描，后续增量检测新增/修改文件 |
| ✅ 缩略图浏览 | 虚拟滚动网格，性能不受图片数量影响 |
| ✅ 标签系统 | 创建/删除/批量编辑标签，标签颜色，按标签筛选 |
| ✅ 备注编辑 | 每张图片可写备注，支持全文搜索 |
| ✅ 评分系统 | 1-5 星评分，支持按评分筛选 |
| ✅ 高级搜索 | 组合条件筛选（标签 + 时间 + 格式 + 评分）+ 全文搜索 |
| ✅ 批量操作 | 多选图片，批量设置/移除标签 |
| ✅ 导出 | 元数据导出为 JSON/CSV |
| ✅ 多工作区 | 同时管理多个文件夹，快速切换 |
| ❌ 图片去重 | 不在当前范围（数据结构已预留 file_hash 字段） |
| ❌ 相似图片查找 | 不在当前范围 |
| ❌ 图片编辑 | 不在当前范围（只管理元数据，不编辑图片内容） |

---

## 9. 非功能需求

| 需求 | 目标 |
|------|------|
| 启动速度 | 冷启动 < 2 秒 |
| 索引性能 | 10000 张图片全量扫描 < 30 秒 |
| 浏览性能 | 虚拟滚动 60fps，100000+ 图片无卡顿 |
| 内存占用 | 空闲 < 150MB |
| 应用体积 | 安装包 < 15MB |
| 主题 | 默认暗色，提供亮色切换 |
| 语言 | 中文界面 |

---

## 10. 项目结构

```
pic-mgr/
├── src/                    — Vue 前端源码
│   ├── App.vue
│   ├── main.ts
│   ├── router/
│   ├── stores/
│   ├── views/
│   ├── components/
│   └── types/
├── src-tauri/              — Rust 后端源码
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   ├── db/
│   │   ├── scanner/
│   │   └── models.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/                 — 静态资源
├── index.html
├── vite.config.ts
├── tsconfig.json
├── package.json
├── CLAUDE.md               — Agent skills 配置
├── docs/
│   ├── agents/             — 技能配置
│   ├── superpowers/specs/  — 设计规格说明
│   └── adr/                — 架构决策记录
└── .scratch/               — 本地 issue tracker
```
