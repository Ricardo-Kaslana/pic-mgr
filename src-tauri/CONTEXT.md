# Backend Context (Rust / Tauri)

## Overview

Tauri 2 桌面应用后端。通过 Tauri commands 暴露 24 个 API，负责文件系统访问、SQLite 读写和缩略图生成。

## Architecture

```
main.rs          — Tauri 入口，注册 commands，管理 AppState + ThumbnailState
lib.rs           — 模块声明，AppState 定义
models.rs        — 14 个数据模型 (ImageRecord, ImageSummary, ImageDetail, ImagePage, ImageSummaryPage, ...)
db/
  schema.rs      — SQLite DDL (workspaces, images, tags, image_tags, images_fts) + 复合索引
  queries.rs     — 查询层 (CRUD, 分页, FTS 搜索, 批量标签, N+1-free 实现)
scanner/
  mod.rs         — 目录递归扫描，SHA-256 哈希，文件元数据提取 (mtime + birth time)
  thumbnail.rs   — JPEG 缩略图生成 (image crate, quality 300px)
commands/
  workspace.rs   — create_workspace, list_workspaces, delete_workspace
  scan.rs        — scan_workspace, rescan_workspace
  images.rs      — get_images, get_images_light, get_image_detail, search_images, find_duplicates
  metadata.rs    — update_notes, update_rating, batch_update_tags, open_in_explorer, delete_image, trash_images, move_images, read_file_bytes
  tags.rs        — get_tags, create_tag, delete_tag
  thumbnail.rs   — get_thumbnail, get_thumbnails_batch, generate_thumbnail, generate_thumbnails_batch
  export.rs      — export_metadata (JSON/CSV, 支持 image_ids 筛选)
```

## Key decisions

- **AppState** 通过 `app.manage()` 注入，commands 通过 `State<AppState>` 获取
- **Database** 使用 `Mutex<Connection>` 保护并发访问
- **ThumbnailState** 独立管理，包含 `Mutex<ThumbnailCache>`
- **SQLite PRAGMAs** 启动时 (`db/mod.rs`) 设置：
  - `journal_mode=WAL` — 读不阻塞写
  - `synchronous=NORMAL` — WAL 模式下与 FULL 同样安全，写入快 ~2x
  - `cache_size=-20000` — 20 MB 页缓存
  - `temp_store=MEMORY` — 临时索引放内存
  - `mmap_size=268435456` — 256 MB 内存映射 I/O
  - `ANALYZE` — 启动时收集行数统计让查询规划器选对索引
- **复合索引** (`db/schema.rs`) — `(workspace_id, sort_column)` 覆盖分页 + 排序的所有热路径：
  - `idx_images_workspace_indexed/updated/name/size/rating/created/file_created`
  - `idx_images_workspace_format`、`idx_images_workspace_hash`
  - `idx_image_tags_image_id`（反向查询）
  - 单列 `idx_images_workspace_id`、`idx_images_hash`、`idx_image_tags_tag_id` 保留
- **N+1-free queries** (`db/queries.rs`)：
  - `get_tags` 用单条 `GROUP BY` 替代每 tag 相关子查询
  - `find_duplicate_groups` 用 CTE + JOIN 替代 1+N 循环
  - `get_image_paths` 用 `WHERE id IN (...)` 单次查询拉所有图片路径
  - `get_images` / `get_images_light` 共享 `build_filter_clause` 生成参数化的 WHERE/ORDER BY 片段
- **轻量 grid 投影** — `ImageSummary` (~80 B/行) 仅含 grid 所需字段；`get_images_light` 回传它；`get_image_detail` 仍返回完整 `ImageRecord`
- **扫描器** 不使用 image crate 读取尺寸（避免栈缓冲区溢出），只做文件系统元数据 + SHA-256
- **缩略图** 按需生成，使用 `catch_unwind` 保护，根据文件头魔数判断格式（非扩展名）
- **批量缩略图** `get_thumbnails_batch` / `generate_thumbnails_batch` 用单次 `WHERE id IN (...)` 查所有图片的 (id, workspace_id, relative_path)，再加一遍 `cache_path(id).exists()` 过滤已缓存条目
- **Tauri 插件** — tauri-plugin-dialog (文件夹选择/保存对话框), tauri-plugin-fs (文件写入)
- **批量插入** 每 50 条释放一次数据库锁
- **工作区删除** 不使用 `ON DELETE CASCADE`，改为手动事务：先删 `image_tags`，再删 `images`（FTS 触发器自动清理），最后删 `workspace`。避免级联链过深触发 `SQLITE_CORRUPT`
- **FTS5 触发器** `'delete'` 命令只用 rowid 参数，不加内容列 — 内容校验在级联删除时会因不匹配导致 `SQLITE_ERROR`
- **数据库迁移** 无框架，在 `mod.rs` 的 `Database::new()` 中通过 `DROP TRIGGER IF EXISTS` + `CREATE TRIGGER IF NOT EXISTS` 更新触发器；`ALTER TABLE ADD COLUMN` 用 `.ok()` 忽略已存在错误

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
| trash | 跨平台回收站 (文件删除) |