# Backend Context (Rust / Tauri)

## Overview

Tauri 2 桌面应用后端。通过 Tauri commands 暴露 17 个 API，负责文件系统访问、SQLite 读写和缩略图生成。

## Architecture

```
main.rs          — Tauri 入口，注册 commands，管理 AppState + ThumbnailState
lib.rs           — 模块声明，AppState 定义
models.rs        — 11 个数据模型 (serde Serialize/Deserialize)
db/
  schema.rs      — SQLite DDL (workspaces, images, tags, image_tags, images_fts)
  queries.rs     — 查询层 (CRUD, 分页, FTS 搜索, 批量标签)
scanner/
  mod.rs         — 目录递归扫描，SHA-256 哈希，文件元数据提取
  thumbnail.rs   — WebP 缩略图生成 (image crate)
commands/
  workspace.rs   — create_workspace, list_workspaces, delete_workspace
  scan.rs        — scan_workspace, rescan_workspace
  images.rs      — get_images, get_image_detail, search_images
  metadata.rs    — update_notes, update_rating, batch_update_tags
  tags.rs        — get_tags, create_tag, delete_tag
  thumbnail.rs   — get_thumbnail, generate_thumbnail
  export.rs      — export_metadata (JSON/CSV)
```

## Key decisions

- **AppState** 通过 `app.manage()` 注入，commands 通过 `State<AppState>` 获取
- **Database** 使用 `Mutex<Connection>` 保护并发访问
- **ThumbnailState** 独立管理，包含 `Mutex<ThumbnailCache>`
- **扫描器** 不使用 image crate 读取尺寸（避免栈缓冲区溢出），只做文件系统元数据 + SHA-256
- **缩略图** 按需生成，使用 `catch_unwind` 保护 image crate 调用
- **批量插入** 每 50 条释放一次数据库锁

## Dependencies

| Crate | 用途 |
|-------|------|
| tauri 2 | 桌面框架 |
| rusqlite (bundled) | SQLite 数据库 |
| image 0.25 | 缩略图生成 |
| sha2 | SHA-256 文件哈希 |
| walkdir | 目录遍历（已弃用，改用 std::fs） |
| serde / serde_json | 序列化 |
| tokio | 异步运行时 |
