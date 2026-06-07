# pic-mgr 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建一个基于 Tauri 2 + Vue 3 + SQLite 的本地图片管理器，支持工作区管理、图片索引、标签/备注/评分编辑、全文搜索、缩略图浏览和元数据导出。

**Architecture:** Rust 后端通过 Tauri commands 暴露 16 个 API，负责文件系统访问、SQLite 读写和缩略图生成；Vue 前端通过 Pinia stores 管理 UI 状态，通过 Vue Router 管理三个路由页面。Rust 永不修改工作区原始文件——所有元数据只写 SQLite。

**Tech Stack:** Tauri 2, Vue 3 (Composition API), Vite, Pinia, Vue Router 4, rusqlite + FTS5, image-rs, walkdir, tokio, sha2

***

## 文件结构

```
pic-mgr/
├── src/                          — Vue 前端
│   ├── main.ts
│   ├── App.vue
│   ├── style.css                 — 全局样式 + CSS 变量 (暗色/亮色主题)
│   ├── router/index.ts
│   ├── stores/
│   │   ├── workspace.ts
│   │   ├── images.ts
│   │   ├── tags.ts
│   │   ├── selection.ts
│   │   ├── thumbnail.ts
│   │   └── settings.ts
│   ├── types/index.ts
│   ├── views/
│   │   ├── WorkspaceHome.vue
│   │   ├── BrowserView.vue
│   │   └── SettingsPage.vue
│   └── components/
│       ├── layout/AppLayout.vue
│       ├── workspace/
│       │   ├── WorkspaceCard.vue
│       │   └── CreateWorkspaceDialog.vue
│       ├── browser/
│       │   ├── SidebarPanel.vue
│       │   ├── TagFilter.vue
│       │   ├── DateRangeFilter.vue
│       │   ├── SearchBar.vue
│       │   ├── ThumbnailGrid.vue
│       │   ├── ThumbnailCard.vue
│       │   └── DetailPanel.vue
│       ├── detail/
│       │   ├── ImagePreview.vue
│       │   ├── MetadataEditor.vue
│       │   └── ImageInfo.vue
│       └── common/
│           ├── VirtualScroller.vue
│           └── TagBadge.vue
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/                    — 应用图标
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── models.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── workspace.rs
│       │   ├── scan.rs
│       │   ├── images.rs
│       │   ├── metadata.rs
│       │   ├── tags.rs
│       │   ├── thumbnail.rs
│       │   └── export.rs
│       ├── db/
│       │   ├── mod.rs
│       │   ├── schema.rs
│       │   └── queries.rs
│       └── scanner/
│           ├── mod.rs
│           └── thumbnail.rs
├── index.html
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── package.json
└── .gitignore
```

***

## Phase 1: 项目脚手架

### Task 1: 初始化 Tauri + Vue 项目

**Files:**

- Create: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`
- Create: `src/main.ts`, `src/App.vue`, `src/style.css`
- Create: `.gitignore`
- [ ] **Step 1: 使用 create-tauri-app 脚手架**

```bash
cd D:\Development\Complex\pic-mgr
npm create tauri-app@latest . -- --template vue-ts --manager npm
```

- [ ] **Step 2: 验证脚手架生成成功**

```bash
ls src/ src-tauri/ package.json
```

Expected: 所有目录和文件都存在。

- [ ] **Step 3: 安装前端依赖**

```bash
npm install
```

- [ ] **Step 4: 安装额外前端依赖**

```bash
npm install vue-router@4 pinia
```

- [ ] **Step 5: 确认 Rust 依赖已配置——编辑** **`src-tauri/Cargo.toml`**

```toml
[package]
name = "pic-mgr"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "fts5"] }
image = "0.25"
walkdir = "2"
sha2 = "0.10"
tokio = { version = "1", features = ["full"] }

[build-dependencies]
tauri-build = { version = "2", features = [] }
```

- [ ] **Step 6: 第一次编译验证**

```bash
cd src-tauri && cargo check
```

Expected: 编译通过，无错误。

- [ ] **Step 7: Commit**

```bash
git init
git add -A
git commit -m "feat: scaffold Tauri + Vue project"
```

***

## Phase 2: Rust 后端核心

### Task 2: 数据模型定义

**Files:**

- Create: `src-tauri/src/models.rs`
- [ ] **Step 1: 编写 models.rs**

```rust
// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: i64,
    pub workspace_id: i64,
    pub relative_path: String,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub file_hash: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub format: Option<String>,
    pub thumbnail_path: Option<String>,
    pub notes: Option<String>,
    pub rating: Option<i32>,
    pub created_at: Option<String>,
    pub updated_at: String,
    pub indexed_at: String,
    pub last_viewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDetail {
    #[serde(flatten)]
    pub record: ImageRecord,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagWithCount {
    #[serde(flatten)]
    pub tag: Tag,
    pub image_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePage {
    pub images: Vec<ImageRecord>,
    pub total: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFilters {
    pub tag_ids: Option<Vec<i64>>,
    pub format: Option<String>,
    pub rating_min: Option<i32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: Option<String>,     // "name" | "date" | "size" | "rating"
    pub sort_dir: Option<String>,    // "asc" | "desc"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub total: i64,
    pub processed: i64,
    pub new_files: i64,
    pub modified_files: i64,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailProgress {
    pub total: i64,
    pub generated: i64,
    pub failed: i64,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: String, // "json" | "csv"
}
```

- [ ] **Step 2: 验证编译**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/models.rs
git commit -m "feat: define data models with serde"
```

***

### Task 3: 数据库初始化与 Schema

**Files:**

- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/schema.rs`
- Modify: `src-tauri/src/lib.rs`
- [ ] **Step 1: 编写 schema.rs**

```rust
// src-tauri/src/db/schema.rs
pub const CREATE_TABLES_SQL: &str = "
CREATE TABLE IF NOT EXISTS workspaces (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS images (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    relative_path   TEXT NOT NULL,
    file_name       TEXT NOT NULL,
    file_size       INTEGER,
    file_hash       TEXT,
    width           INTEGER,
    height          INTEGER,
    format          TEXT,
    thumbnail_path  TEXT,
    notes           TEXT DEFAULT '',
    rating          INTEGER DEFAULT 0 CHECK(rating >= 0 AND rating <= 5),
    created_at      TEXT,
    updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
    indexed_at      TEXT NOT NULL DEFAULT (datetime('now')),
    last_viewed_at  TEXT,
    UNIQUE(workspace_id, relative_path)
);

CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE,
    color TEXT
);

CREATE TABLE IF NOT EXISTS image_tags (
    image_id INTEGER NOT NULL REFERENCES images(id) ON DELETE CASCADE,
    tag_id   INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (image_id, tag_id)
);

CREATE VIRTUAL TABLE IF NOT EXISTS images_fts USING fts5(
    file_name,
    notes,
    content='images',
    content_rowid='id'
);

-- Triggers to keep FTS in sync
CREATE TRIGGER IF NOT EXISTS images_fts_insert AFTER INSERT ON images BEGIN
    INSERT INTO images_fts(rowid, file_name, notes) VALUES (new.id, new.file_name, new.notes);
END;

CREATE TRIGGER IF NOT EXISTS images_fts_delete AFTER DELETE ON images BEGIN
    INSERT INTO images_fts(images_fts, rowid, file_name, notes) VALUES('delete', old.id, old.file_name, old.notes);
END;

CREATE TRIGGER IF NOT EXISTS images_fts_update AFTER UPDATE ON images BEGIN
    INSERT INTO images_fts(images_fts, rowid, file_name, notes) VALUES('delete', old.id, old.file_name, old.notes);
    INSERT INTO images_fts(rowid, file_name, notes) VALUES (new.id, new.file_name, new.notes);
END;
";
```

- [ ] **Step 2: 编写 db/mod.rs**

```rust
// src-tauri/src/db/mod.rs
pub mod schema;

use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: &PathBuf) -> Result<Self> {
        std::fs::create_dir_all(app_data_dir).ok();
        let db_path = app_data_dir.join("pic-mgr.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(schema::CREATE_TABLES_SQL)?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }
}
```

- [ ] **Step 3: 更新 lib.rs 暴露 Database**

```rust
// src-tauri/src/lib.rs
pub mod commands;
pub mod db;
pub mod models;
pub mod scanner;

use db::Database;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub app_data_dir: PathBuf,
}
```

- [ ] **Step 4: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db/ src-tauri/src/lib.rs
git commit -m "feat: database initialization with SQLite schema and FTS5"
```

***

### Task 4: 数据库查询层

**Files:**

- Create: `src-tauri/src/db/queries.rs`
- [ ] **Step 1: 编写 queries.rs**

```rust
// src-tauri/src/db/queries.rs
use rusqlite::{params, Connection, Result};
use crate::models::*;

// ── Workspace ──

pub fn insert_workspace(conn: &Connection, name: &str, path: &str) -> Result<Workspace> {
    conn.execute(
        "INSERT INTO workspaces (name, path) VALUES (?1, ?2)",
        params![name, path],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Workspace {
        id,
        name: name.to_string(),
        path: path.to_string(),
        created_at: chrono_now(conn),
        updated_at: chrono_now(conn),
    })
}

pub fn list_workspaces(conn: &Connection) -> Result<Vec<Workspace>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, created_at, updated_at FROM workspaces ORDER BY updated_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn delete_workspace(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_workspace(conn: &Connection, id: i64) -> Result<Option<Workspace>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, created_at, updated_at FROM workspaces WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Workspace {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

// ── Images ──

pub fn insert_image(conn: &Connection, image: &ImageRecord) -> Result<i64> {
    conn.execute(
        "INSERT INTO images (workspace_id, relative_path, file_name, file_size, file_hash, width, height, format, thumbnail_path, notes, rating, created_at, updated_at, indexed_at, last_viewed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
         ON CONFLICT(workspace_id, relative_path) DO UPDATE SET
         file_size=excluded.file_size, file_hash=excluded.file_hash, width=excluded.width,
         height=excluded.height, format=excluded.format, updated_at=excluded.updated_at",
        params![
            image.workspace_id, image.relative_path, image.file_name, image.file_size,
            image.file_hash, image.width, image.height, image.format, image.thumbnail_path,
            image.notes, image.rating, image.created_at, image.updated_at, image.indexed_at,
            image.last_viewed_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_images(
    conn: &Connection,
    workspace_id: i64,
    filters: &ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImagePage> {
    let mut sql = String::from(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i WHERE i.workspace_id = ?1"
    );
    let mut param_idx = 2;
    let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(workspace_id)];

    if let Some(ref tag_ids) = filters.tag_ids {
        if !tag_ids.is_empty() {
            let placeholders: Vec<String> = tag_ids.iter().enumerate()
                .map(|(i, _)| format!("?{}", param_idx + i))
                .collect();
            sql.push_str(&format!(
                " AND i.id IN (SELECT image_id FROM image_tags WHERE tag_id IN ({}))",
                placeholders.join(",")
            ));
            for &tid in tag_ids {
                param_values.push(Box::new(tid));
                param_idx += 1;
            }
        }
    }

    if let Some(ref format) = filters.format {
        sql.push_str(&format!(" AND i.format = ?{}", param_idx));
        param_values.push(Box::new(format.clone()));
        param_idx += 1;
    }

    if let Some(rating_min) = filters.rating_min {
        sql.push_str(&format!(" AND i.rating >= ?{}", param_idx));
        param_values.push(Box::new(rating_min));
        param_idx += 1;
    }

    if let Some(ref date_from) = filters.date_from {
        sql.push_str(&format!(" AND i.created_at >= ?{}", param_idx));
        param_values.push(Box::new(date_from.clone()));
        param_idx += 1;
    }

    if let Some(ref date_to) = filters.date_to {
        sql.push_str(&format!(" AND i.created_at <= ?{}", param_idx));
        param_values.push(Box::new(date_to.clone()));
        param_idx += 1;
    }

    // Sort
    let sort_col = match filters.sort_by.as_deref() {
        Some("name") => "i.file_name",
        Some("date") => "i.created_at",
        Some("size") => "i.file_size",
        Some("rating") => "i.rating",
        _ => "i.indexed_at",
    };
    let sort_dir = match filters.sort_dir.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_col, sort_dir));

    // Count total
    let count_sql = sql.replacen(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.updated_at, i.indexed_at, i.last_viewed_at",
        "SELECT COUNT(*)",
        1,
    );
    let total: i64 = {
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        conn.query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?
    };

    // Paginate
    let offset = (page - 1) * page_size;
    sql.push_str(&format!(" LIMIT ?{} OFFSET ?{}", param_idx, param_idx + 1));
    param_values.push(Box::new(page_size));
    param_values.push(Box::new(offset));

    let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            relative_path: row.get(2)?,
            file_name: row.get(3)?,
            file_size: row.get(4)?,
            file_hash: row.get(5)?,
            width: row.get(6)?,
            height: row.get(7)?,
            format: row.get(8)?,
            thumbnail_path: row.get(9)?,
            notes: row.get(10)?,
            rating: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            indexed_at: row.get(14)?,
            last_viewed_at: row.get(15)?,
        })
    })?;
    let images: Result<Vec<ImageRecord>> = rows.collect();

    Ok(ImagePage {
        images: images?,
        total,
        page,
        page_size,
    })
}

pub fn get_image_detail(conn: &Connection, image_id: i64) -> Result<Option<ImageRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, relative_path, file_name, file_size, file_hash,
                width, height, format, thumbnail_path, notes, rating,
                created_at, updated_at, indexed_at, last_viewed_at
         FROM images WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![image_id], |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            relative_path: row.get(2)?,
            file_name: row.get(3)?,
            file_size: row.get(4)?,
            file_hash: row.get(5)?,
            width: row.get(6)?,
            height: row.get(7)?,
            format: row.get(8)?,
            thumbnail_path: row.get(9)?,
            notes: row.get(10)?,
            rating: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
            indexed_at: row.get(14)?,
            last_viewed_at: row.get(15)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn search_images(
    conn: &Connection,
    workspace_id: i64,
    query: &str,
    page: i32,
    page_size: i32,
) -> Result<ImagePage> {
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM images_fts WHERE images_fts MATCH ?1 AND rowid IN (SELECT id FROM images WHERE workspace_id = ?2)",
        params![query, workspace_id],
        |row| row.get(0),
    )?;

    let offset = (page - 1) * page_size;
    let mut stmt = conn.prepare(
        "SELECT i.id, i.workspace_id, i.relative_path, i.file_name, i.file_size, i.file_hash,
                i.width, i.height, i.format, i.thumbnail_path, i.notes, i.rating,
                i.created_at, i.updated_at, i.indexed_at, i.last_viewed_at
         FROM images i
         INNER JOIN images_fts fts ON i.id = fts.rowid
         WHERE images_fts MATCH ?1 AND i.workspace_id = ?2
         ORDER BY rank
         LIMIT ?3 OFFSET ?4"
    )?;
    let rows = stmt.query_map(params![query, workspace_id, page_size, offset], |row| {
        Ok(ImageRecord {
            id: row.get(0)?, workspace_id: row.get(1)?, relative_path: row.get(2)?,
            file_name: row.get(3)?, file_size: row.get(4)?, file_hash: row.get(5)?,
            width: row.get(6)?, height: row.get(7)?, format: row.get(8)?,
            thumbnail_path: row.get(9)?, notes: row.get(10)?, rating: row.get(11)?,
            created_at: row.get(12)?, updated_at: row.get(13)?, indexed_at: row.get(14)?,
            last_viewed_at: row.get(15)?,
        })
    })?;
    let images: Result<Vec<ImageRecord>> = rows.collect();

    Ok(ImagePage {
        images: images?,
        total,
        page,
        page_size,
    })
}

// ── Metadata ──

pub fn update_notes(conn: &Connection, image_id: i64, notes: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET notes = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![notes, image_id],
    )?;
    Ok(())
}

pub fn update_rating(conn: &Connection, image_id: i64, rating: i32) -> Result<()> {
    conn.execute(
        "UPDATE images SET rating = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![rating, image_id],
    )?;
    Ok(())
}

pub fn batch_update_tags(
    conn: &Connection,
    image_ids: &[i64],
    tag_ids: &[i64],
    action: &str,
) -> Result<()> {
    match action {
        "add" => {
            for &img_id in image_ids {
                for &tag_id in tag_ids {
                    conn.execute(
                        "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                        params![img_id, tag_id],
                    )?;
                }
            }
        }
        "remove" => {
            for &img_id in image_ids {
                for &tag_id in tag_ids {
                    conn.execute(
                        "DELETE FROM image_tags WHERE image_id = ?1 AND tag_id = ?2",
                        params![img_id, tag_id],
                    )?;
                }
            }
        }
        "set" => {
            for &img_id in image_ids {
                conn.execute("DELETE FROM image_tags WHERE image_id = ?1", params![img_id])?;
                for &tag_id in tag_ids {
                    conn.execute(
                        "INSERT OR IGNORE INTO image_tags (image_id, tag_id) VALUES (?1, ?2)",
                        params![img_id, tag_id],
                    )?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ── Tags ──

pub fn get_tags(conn: &Connection, workspace_id: i64) -> Result<Vec<TagWithCount>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color, COUNT(it.image_id) as cnt
         FROM tags t
         LEFT JOIN image_tags it ON t.id = it.tag_id
         LEFT JOIN images i ON it.image_id = i.id AND i.workspace_id = ?1
         GROUP BY t.id
         HAVING cnt > 0
         ORDER BY cnt DESC"
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(TagWithCount {
            tag: Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            },
            image_count: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn create_tag(conn: &Connection, name: &str, color: Option<&str>) -> Result<Tag> {
    conn.execute(
        "INSERT INTO tags (name, color) VALUES (?1, ?2)",
        params![name, color],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Tag {
        id,
        name: name.to_string(),
        color: color.map(|s| s.to_string()),
    })
}

pub fn delete_tag(conn: &Connection, tag_id: i64) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![tag_id])?;
    Ok(())
}

pub fn get_tags_for_image(conn: &Connection, image_id: i64) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t
         INNER JOIN image_tags it ON t.id = it.tag_id
         WHERE it.image_id = ?1"
    )?;
    let rows = stmt.query_map(params![image_id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    })?;
    rows.collect()
}

// ── Thumbnails ──

pub fn update_thumbnail_path(conn: &Connection, image_id: i64, path: &str) -> Result<()> {
    conn.execute(
        "UPDATE images SET thumbnail_path = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![path, image_id],
    )?;
    Ok(())
}

// ── Helpers ──

fn chrono_now(conn: &Connection) -> String {
    conn.query_row("SELECT datetime('now')", [], |row| row.get(0))
        .unwrap_or_else(|_| "1970-01-01T00:00:00".to_string())
}
```

- [ ] **Step 2: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db/queries.rs
git commit -m "feat: database query layer for workspaces, images, tags, FTS"
```

***

### Task 5: Commands 模块注册

**Files:**

- Create: `src-tauri/src/commands/mod.rs`
- [ ] **Step 1: 编写 mod.rs**

```rust
// src-tauri/src/commands/mod.rs
pub mod workspace;
pub mod scan;
pub mod images;
pub mod metadata;
pub mod tags;
pub mod thumbnail;
pub mod export;
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/commands/mod.rs
git commit -m "feat: commands module registration"
```

***

### Task 6: 工作区 Commands

**Files:**

- Create: `src-tauri/src/commands/workspace.rs`
- Modify: `src-tauri/src/main.rs`
- [ ] **Step 1: 编写 workspace.rs**

```rust
// src-tauri/src/commands/workspace.rs
use tauri::State;
use crate::models::Workspace;
use crate::db::Database;
use crate::AppState;

#[tauri::command]
pub fn create_workspace(state: State<AppState>, name: String, path: String) -> Result<Workspace, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::insert_workspace(&conn, &name, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_workspaces(state: State<AppState>) -> Result<Vec<Workspace>, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::list_workspaces(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_workspace(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_workspace(&conn, id).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 编写 main.rs**

```rust
// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod models;
mod scanner;

use db::Database;
use std::path::PathBuf;

pub struct AppState {
    pub db: Database,
    pub app_data_dir: PathBuf,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));
            let database = Database::new(&app_data_dir)
                .expect("Failed to initialize database");
            app.manage(AppState {
                db: database,
                app_data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::workspace::create_workspace,
            commands::workspace::list_workspaces,
            commands::workspace::delete_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/workspace.rs src-tauri/src/main.rs
git commit -m "feat: workspace commands (create, list, delete)"
```

***

### Task 7: 扫描器

**Files:**

- Create: `src-tauri/src/scanner/mod.rs`
- Create: `src-tauri/src/commands/scan.rs`
- [ ] **Step 1: 编写 scanner/mod.rs**

```rust
// src-tauri/src/scanner/mod.rs
pub mod thumbnail;

use walkdir::WalkDir;
use sha2::{Sha256, Digest};
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;
use crate::models::{ImageRecord, ScanProgress};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif",
    "cr2", "nef", "arw",
];

fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn compute_file_hash(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn file_modified_at(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(UNIX_EPOCH).ok()?;
    let dt = chrono_lite(duration.as_secs() as i64);
    Some(dt)
}

fn chrono_lite(unix_ts: i64) -> String {
    // Simple ISO 8601 from unix timestamp
    let secs_per_day = 86400;
    let days = unix_ts / secs_per_day;
    let remaining = unix_ts % secs_per_day;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Days since epoch → date (approximate for this scope)
    let (y, m, d) = days_since_epoch_to_date(days);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, hours, minutes, seconds)
}

fn days_since_epoch_to_date(days: i64) -> (i64, i64, i64) {
    // Simple Gregorian calendar conversion
    let mut days = days;
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }
    let month_lengths = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for &len in month_lengths.iter() {
        if days < len { break; }
        days -= len;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

pub fn scan_directory(
    workspace_id: i64,
    workspace_path: &Path,
    incremental: bool,
) -> ScanProgress {
    let mut total = 0i64;
    let mut new_files = 0i64;
    let mut modified_files = 0i64;

    for entry in WalkDir::new(workspace_path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() { continue; }
        if !is_image_file(entry.path()) { continue; }
        total += 1;
    }

    // For now, return progress; actual DB insertion is done by the caller
    ScanProgress {
        total,
        processed: 0,
        new_files: 0,
        modified_files: 0,
        finished: false,
    }
}

pub fn scan_and_index(
    workspace_id: i64,
    workspace_path: &Path,
    incremental: bool,
) -> ScanProgress {
    let mut total = 0i64;
    let mut processed = 0i64;
    let mut new_files = 0i64;
    let mut modified_files = 0i64;

    let paths: Vec<_> = WalkDir::new(workspace_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_image_file(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect();

    total = paths.len() as i64;

    for path in &paths {
        let relative = path.strip_prefix(workspace_path).unwrap_or(path);
        let relative_str = relative.to_string_lossy().to_string();
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let format = path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        let file_size = fs::metadata(path).ok().map(|m| m.len() as i64);
        let file_hash = compute_file_hash(path);
        let created_at = file_modified_at(path);
        let now = chrono_lite_now();

        // Try to read dimensions
        let (width, height) = image::image_dimensions(path)
            .map(|(w, h)| (Some(w as i32), Some(h as i32)))
            .unwrap_or((None, None));

        let record = ImageRecord {
            id: 0,
            workspace_id,
            relative_path: relative_str.clone(),
            file_name,
            file_size,
            file_hash,
            width,
            height,
            format,
            thumbnail_path: None,
            notes: None,
            rating: None,
            created_at,
            updated_at: now.clone(),
            indexed_at: now,
            last_viewed_at: None,
        };

        processed += 1;
        // The caller will handle DB insertion using AppState
        // (scan_and_index returns progress, actual DB work is in the command)
    }

    ScanProgress {
        total,
        processed,
        new_files,
        modified_files,
        finished: true,
    }
}

fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    chrono_lite(now.as_secs() as i64)
}
```

- [ ] **Step 2: 编写 commands/scan.rs**

```rust
// src-tauri/src/commands/scan.rs
use tauri::State;
use crate::models::ScanProgress;
use crate::AppState;

#[tauri::command]
pub async fn scan_workspace(state: State<'_, AppState>, id: i64) -> Result<ScanProgress, String> {
    let workspace = {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        crate::db::queries::get_workspace(&conn, id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workspace not found".to_string())?
    };

    let ws_path = std::path::PathBuf::from(&workspace.path);
    if !ws_path.exists() {
        return Err(format!("Path does not exist: {}", workspace.path));
    }

    let progress = crate::scanner::scan_and_index(id, &ws_path, true);

    // Insert scanned images into DB
    use crate::scanner::scan_directory_individually;
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    let image_records = scan_directory_individually(id, &ws_path);
    for record in &image_records {
        crate::db::queries::insert_image(&conn, record).map_err(|e| e.to_string())?;
    }

    Ok(progress)
}

#[tauri::command]
pub async fn rescan_workspace(state: State<'_, AppState>, id: i64) -> Result<ScanProgress, String> {
    // Clear existing images and re-scan
    {
        let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM images WHERE workspace_id = ?1", rusqlite::params![id])
            .map_err(|e| e.to_string())?;
    }
    Box::pin(scan_workspace(state, id)).await
}
```

- [ ] **Step 3: 在 scanner/mod.rs 末尾追加独立扫描函数**

```rust
// 追加到 src-tauri/src/scanner/mod.rs 末尾
pub fn scan_directory_individually(workspace_id: i64, workspace_path: &Path) -> Vec<ImageRecord> {
    use walkdir::WalkDir;
    use std::fs;

    WalkDir::new(workspace_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && is_image_file(e.path()))
        .map(|entry| {
            let path = entry.path();
            let relative = path.strip_prefix(workspace_path).unwrap_or(path);
            let relative_str = relative.to_string_lossy().to_string();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let format = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();
            let file_size = fs::metadata(path).ok().map(|m| m.len() as i64);
            let file_hash = compute_file_hash(path);
            let created_at = file_modified_at(path);
            let now = chrono_lite_now();

            let (width, height) = image::image_dimensions(path)
                .map(|(w, h)| (Some(w as i32), Some(h as i32)))
                .unwrap_or((None, None));

            ImageRecord {
                id: 0,
                workspace_id,
                relative_path: relative_str,
                file_name,
                file_size,
                file_hash,
                width,
                height,
                format,
                thumbnail_path: None,
                notes: None,
                rating: None,
                created_at,
                updated_at: now.clone(),
                indexed_at: now,
                last_viewed_at: None,
            }
        })
        .collect()
}
```

- [ ] **Step 4: 更新 main.rs 注册 scan commands**

```rust
.invoke_handler(tauri::generate_handler![
    commands::workspace::create_workspace,
    commands::workspace::list_workspaces,
    commands::workspace::delete_workspace,
    commands::scan::scan_workspace,
    commands::scan::rescan_workspace,
])
```

- [ ] **Step 5: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/scanner/ src-tauri/src/commands/scan.rs src-tauri/src/main.rs
git commit -m "feat: directory scanner with image detection and indexing"
```

***

### Task 8: 图片查询 Commands

**Files:**

- Create: `src-tauri/src/commands/images.rs`
- [ ] **Step 1: 编写 images.rs**

```rust
// src-tauri/src/commands/images.rs
use tauri::State;
use crate::models::{ImagePage, ImageDetail, ImageFilters, ImageRecord, Tag};
use crate::AppState;

#[tauri::command]
pub fn get_images(
    state: State<AppState>,
    workspace_id: i64,
    filters: ImageFilters,
    page: i32,
    page_size: i32,
) -> Result<ImagePage, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_images(&conn, workspace_id, &filters, page, page_size)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_image_detail(state: State<AppState>, image_id: i64) -> Result<ImageDetail, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;
    let tags = crate::db::queries::get_tags_for_image(&conn, image_id)
        .map_err(|e| e.to_string())?;

    Ok(ImageDetail { record, tags })
}

#[tauri::command]
pub fn search_images(
    state: State<AppState>,
    workspace_id: i64,
    query: String,
    page: i32,
    page_size: i32,
) -> Result<ImagePage, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::search_images(&conn, workspace_id, &query, page, page_size)
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 更新 main.rs 注册 images commands**

```rust
commands::images::get_images,
commands::images::get_image_detail,
commands::images::search_images,
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/images.rs src-tauri/src/main.rs
git commit -m "feat: image query commands with filtering and FTS search"
```

***

### Task 9: 元数据编辑 Commands

**Files:**

- Create: `src-tauri/src/commands/metadata.rs`
- [ ] **Step 1: 编写 metadata.rs**

```rust
// src-tauri/src/commands/metadata.rs
use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn update_notes(state: State<AppState>, image_id: i64, notes: String) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_notes(&conn, image_id, &notes).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_rating(state: State<AppState>, image_id: i64, rating: i32) -> Result<(), String> {
    if rating < 0 || rating > 5 {
        return Err("Rating must be 0-5".to_string());
    }
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_rating(&conn, image_id, rating).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn batch_update_tags(
    state: State<AppState>,
    image_ids: Vec<i64>,
    tag_ids: Vec<i64>,
    action: String,
) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::batch_update_tags(&conn, &image_ids, &tag_ids, &action)
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 更新 main.rs 注册**

```rust
commands::metadata::update_notes,
commands::metadata::update_rating,
commands::metadata::batch_update_tags,
```

- [ ] **Step 3: 编译验证 + Commit**

```bash
cd src-tauri && cargo check
git add src-tauri/src/commands/metadata.rs src-tauri/src/main.rs
git commit -m "feat: metadata commands (notes, rating, batch tags)"
```

***

### Task 10: 标签 Commands

**Files:**

- Create: `src-tauri/src/commands/tags.rs`
- [ ] **Step 1: 编写 tags.rs**

```rust
// src-tauri/src/commands/tags.rs
use tauri::State;
use crate::models::{Tag, TagWithCount};
use crate::AppState;

#[tauri::command]
pub fn get_tags(state: State<AppState>, workspace_id: i64) -> Result<Vec<TagWithCount>, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::get_tags(&conn, workspace_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_tag(
    state: State<AppState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::create_tag(&conn, &name, color.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_tag(state: State<AppState>, tag_id: i64) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::delete_tag(&conn, tag_id).map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 更新 main.rs 注册**

```rust
commands::tags::get_tags,
commands::tags::create_tag,
commands::tags::delete_tag,
```

- [ ] **Step 3: 编译验证 + Commit**

```bash
cd src-tauri && cargo check
git add src-tauri/src/commands/tags.rs src-tauri/src/main.rs
git commit -m "feat: tag commands (list, create, delete)"
```

***

### Task 11: 缩略图管线

**Files:**

- Create: `src-tauri/src/scanner/thumbnail.rs`
- Create: `src-tauri/src/commands/thumbnail.rs`
- [ ] **Step 1: 编写 scanner/thumbnail.rs**

```rust
// src-tauri/src/scanner/thumbnail.rs
use image::GenericImageView;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use crate::models::ThumbnailProgress;

pub struct ThumbnailCache {
    pub cache_dir: PathBuf,
    pub size: u32,
    pub quality: f32,
}

impl ThumbnailCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&cache_dir).ok();
        ThumbnailCache {
            cache_dir,
            size: 300,
            quality: 80.0,
        }
    }

    pub fn cache_path(&self, image_id: i64) -> PathBuf {
        self.cache_dir.join(format!("{}.webp", image_id))
    }

    pub fn generate(&self, source_path: &Path, image_id: i64) -> Result<String, String> {
        let img = image::open(source_path).map_err(|e| format!("Cannot open image: {}", e))?;
        let (w, h) = img.dimensions();
        let aspect = w as f64 / h as f64;

        let (new_w, new_h) = if w > h {
            (self.size, (self.size as f64 / aspect) as u32)
        } else {
            ((self.size as f64 * aspect) as u32, self.size)
        };

        let thumbnail = img.thumbnail(new_w, new_h);
        let cache_path = self.cache_path(image_id);

        // Save as WebP using image crate built-in encoder
        let mut output = Vec::new();
        let encoder = image::codecs::webp::WebPEncoder::new_lossless(&mut output);
        encoder.encode(
            thumbnail.to_rgba8().as_raw(),
            new_w,
            new_h,
            image::ColorType::Rgba8.into(),
        ).map_err(|e| format!("Cannot encode WebP: {}", e))?;

        std::fs::write(&cache_path, &output)
            .map_err(|e| format!("Cannot write thumbnail: {}", e))?;

        Ok(cache_path.to_string_lossy().to_string())
    }

    pub fn get_bytes(&self, image_id: i64) -> Option<Vec<u8>> {
        let path = self.cache_path(image_id);
        std::fs::read(&path).ok()
    }
}
```

- [ ] **Step 2: 编写 commands/thumbnail.rs**

```rust
// src-tauri/src/commands/thumbnail.rs
use tauri::State;
use crate::AppState;
use crate::scanner::thumbnail::ThumbnailCache;
use std::sync::Mutex;

pub struct ThumbnailState {
    pub cache: Mutex<ThumbnailCache>,
}

#[tauri::command]
pub fn get_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<Vec<u8>, String> {
    // Check cache first
    let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
    if let Some(bytes) = cache.get_bytes(image_id) {
        return Ok(bytes);
    }

    // Generate on demand
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;

    let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workspace not found".to_string())?;

    drop(conn);

    let source_path = std::path::PathBuf::from(&workspace.path).join(&record.relative_path);
    let thumb_path = cache.generate(&source_path, image_id)?;

    // Update DB
    let conn2 = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_thumbnail_path(&conn2, image_id, &thumb_path)
        .map_err(|e| e.to_string())?;

    cache.get_bytes(image_id).ok_or_else(|| "Failed to read generated thumbnail".to_string())
}

#[tauri::command]
pub fn generate_thumbnail(
    state: State<AppState>,
    thumb_state: State<ThumbnailState>,
    image_id: i64,
) -> Result<(), String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;
    let record = crate::db::queries::get_image_detail(&conn, image_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Image not found".to_string())?;
    let workspace = crate::db::queries::get_workspace(&conn, record.workspace_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Workspace not found".to_string())?;
    drop(conn);

    let source_path = std::path::PathBuf::from(&workspace.path).join(&record.relative_path);
    let cache = thumb_state.cache.lock().map_err(|e| e.to_string())?;
    let thumb_path = cache.generate(&source_path, image_id)?;

    let conn2 = state.db.conn.lock().map_err(|e| e.to_string())?;
    crate::db::queries::update_thumbnail_path(&conn2, image_id, &thumb_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}
```

- [ ] **Step 3: 更新 main.rs 添加 ThumbnailState 和命令注册**

```rust
use scanner::thumbnail::ThumbnailCache;

// In setup:
let cache_dir = app_data_dir.join("thumbnails");
let thumb_cache = ThumbnailCache::new(cache_dir);
app.manage(ThumbnailState {
    cache: Mutex::new(thumb_cache),
});

// In invoke_handler:
commands::thumbnail::get_thumbnail,
commands::thumbnail::generate_thumbnail,
```

- [ ] **Step 4: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/scanner/thumbnail.rs src-tauri/src/commands/thumbnail.rs src-tauri/src/main.rs
git commit -m "feat: thumbnail generation with WebP caching"
```

***

### Task 12: 导出 Command

**Files:**

- Create: `src-tauri/src/commands/export.rs`
- [ ] **Step 1: 编写 export.rs**

```rust
// src-tauri/src/commands/export.rs
use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn export_metadata(
    state: State<AppState>,
    workspace_id: i64,
    format: String,
) -> Result<String, String> {
    let conn = state.db.conn.lock().map_err(|e| e.to_string())?;

    use crate::db::queries::get_image_detail;

    // Get all images for workspace
    let mut stmt = conn.prepare("SELECT id FROM images WHERE workspace_id = ?1")
        .map_err(|e| e.to_string())?;
    let image_ids: Vec<i64> = stmt.query_map(rusqlite::params![workspace_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut records = Vec::new();
    for id in image_ids {
        if let Ok(Some(record)) = get_image_detail(&conn, id) {
            let tags = crate::db::queries::get_tags_for_image(&conn, id).unwrap_or_default();
            records.push(serde_json::json!({
                "id": record.id,
                "file_name": record.file_name,
                "relative_path": record.relative_path,
                "file_size": record.file_size,
                "width": record.width,
                "height": record.height,
                "format": record.format,
                "notes": record.notes,
                "rating": record.rating,
                "tags": tags.iter().map(|t| &t.name).collect::<Vec<_>>(),
                "created_at": record.created_at,
                "updated_at": record.updated_at,
                "indexed_at": record.indexed_at,
            }));
        }
    }

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&records).map_err(|e| e.to_string()),
        "csv" => {
            let mut csv = String::from("id,file_name,relative_path,format,width,height,file_size,rating,notes,tags,created_at\n");
            for r in &records {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},\"{}\",\"{}\",{}\n",
                    r["id"], r["file_name"].as_str().unwrap_or(""),
                    r["relative_path"].as_str().unwrap_or(""),
                    r["format"].as_str().unwrap_or(""),
                    r["width"].as_i64().unwrap_or(0),
                    r["height"].as_i64().unwrap_or(0),
                    r["file_size"].as_i64().unwrap_or(0),
                    r["rating"].as_i64().unwrap_or(0),
                    r["notes"].as_str().unwrap_or("").replace("\"", "\"\""),
                    r["tags"].as_array().map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(";")).unwrap_or_default(),
                    r["created_at"].as_str().unwrap_or(""),
                ));
            }
            Ok(csv)
        }
        _ => Err("Unsupported format. Use 'json' or 'csv'".to_string()),
    }
}
```

- [ ] **Step 2: 更新 main.rs 注册**

```rust
commands::export::export_metadata,
```

- [ ] **Step 3: 编译验证 + Commit**

```bash
cd src-tauri && cargo check
git add src-tauri/src/commands/export.rs src-tauri/src/main.rs
git commit -m "feat: metadata export (JSON/CSV)"
```

***

## Phase 3: Vue 前端

### Task 13: 前端基础设施

**Files:**

- Create: `src/types/index.ts`
- Create: `src/router/index.ts`
- Modify: `src/main.ts`
- [ ] **Step 1: 编写类型定义 src/types/index.ts**

```typescript
// src/types/index.ts
export interface Workspace {
  id: number
  name: string
  path: string
  created_at: string
  updated_at: string
}

export interface ImageRecord {
  id: number
  workspace_id: number
  relative_path: string
  file_name: string
  file_size: number | null
  file_hash: string | null
  width: number | null
  height: number | null
  format: string | null
  thumbnail_path: string | null
  notes: string | null
  rating: number | null
  created_at: string | null
  updated_at: string
  indexed_at: string
  last_viewed_at: string | null
}

export interface ImageDetail {
  id: number
  workspace_id: number
  relative_path: string
  file_name: string
  file_size: number | null
  file_hash: string | null
  width: number | null
  height: number | null
  format: string | null
  thumbnail_path: string | null
  notes: string | null
  rating: number | null
  created_at: string | null
  updated_at: string
  indexed_at: string
  last_viewed_at: string | null
  tags: Tag[]
}

export interface Tag {
  id: number
  name: string
  color: string | null
}

export interface TagWithCount extends Tag {
  image_count: number
}

export interface ImagePage {
  images: ImageRecord[]
  total: number
  page: number
  page_size: number
}

export interface ImageFilters {
  tag_ids: number[] | null
  format: string | null
  rating_min: number | null
  date_from: string | null
  date_to: string | null
  sort_by: string | null
  sort_dir: string | null
}

export interface ScanProgress {
  total: number
  processed: number
  new_files: number
  modified_files: number
  finished: boolean
}

export interface ThumbnailProgress {
  total: number
  generated: number
  failed: number
  finished: boolean
}
```

- [ ] **Step 2: 编写路由 src/router/index.ts**

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/WorkspaceHome.vue'),
    },
    {
      path: '/workspace/:id',
      name: 'browser',
      component: () => import('../views/BrowserView.vue'),
      props: true,
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsPage.vue'),
    },
  ],
})

export default router
```

- [ ] **Step 3: 更新 src/main.ts**

```typescript
// src/main.ts
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './style.css'

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
```

- [ ] **Step 4: Commit**

```bash
git add src/types/ src/router/ src/main.ts
git commit -m "feat: Vue frontend infrastructure (types, router, Pinia)"
```

***

### Task 14: Pinia Stores

**Files:**

- Create: `src/stores/workspace.ts`
- Create: `src/stores/images.ts`
- Create: `src/stores/tags.ts`
- Create: `src/stores/selection.ts`
- Create: `src/stores/thumbnail.ts`
- Create: `src/stores/settings.ts`
- [ ] **Step 1: workspace store**

```typescript
// src/stores/workspace.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Workspace, ScanProgress } from '../types'

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentId = ref<number | null>(null)
  const scanProgress = ref<ScanProgress | null>(null)

  const current = computed(() =>
    workspaces.value.find(w => w.id === currentId.value) ?? null
  )

  async function fetchAll() {
    workspaces.value = await invoke<Workspace[]>('list_workspaces')
  }

  async function create(name: string, path: string) {
    const ws = await invoke<Workspace>('create_workspace', { name, path })
    await fetchAll()
    return ws
  }

  async function remove(id: number) {
    await invoke('delete_workspace', { id })
    if (currentId.value === id) currentId.value = null
    await fetchAll()
  }

  async function scan(id: number) {
    scanProgress.value = null
    const progress = await invoke<ScanProgress>('scan_workspace', { id })
    scanProgress.value = progress
    return progress
  }

  function select(id: number) {
    currentId.value = id
  }

  return { workspaces, currentId, current, scanProgress, fetchAll, create, remove, scan, select }
})
```

- [ ] **Step 2: images store**

```typescript
// src/stores/images.ts
import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageRecord, ImageDetail, ImagePage, ImageFilters } from '../types'

export const useImageStore = defineStore('images', () => {
  const images = ref<ImageRecord[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(100)
  const loading = ref(false)
  const selectedDetail = ref<ImageDetail | null>(null)

  const filters = reactive<ImageFilters>({
    tag_ids: null,
    format: null,
    rating_min: null,
    date_from: null,
    date_to: null,
    sort_by: 'date',
    sort_dir: 'desc',
  })

  async function fetchImages(workspaceId: number) {
    loading.value = true
    try {
      const result = await invoke<ImagePage>('get_images', {
        workspaceId,
        filters: { ...filters },
        page: page.value,
        pageSize: pageSize.value,
      })
      images.value = result.images
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  async function fetchDetail(imageId: number) {
    selectedDetail.value = await invoke<ImageDetail>('get_image_detail', { imageId })
  }

  async function search(workspaceId: number, query: string) {
    loading.value = true
    try {
      const result = await invoke<ImagePage>('search_images', {
        workspaceId,
        query,
        page: page.value,
        pageSize: pageSize.value,
      })
      images.value = result.images
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  function setPage(p: number) { page.value = p }

  return { images, total, page, pageSize, loading, selectedDetail, filters, fetchImages, fetchDetail, search, setPage }
})
```

- [ ] **Step 3: tags store**

```typescript
// src/stores/tags.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Tag, TagWithCount } from '../types'

export const useTagStore = defineStore('tags', () => {
  const tags = ref<TagWithCount[]>([])
  const loading = ref(false)

  async function fetchTags(workspaceId: number) {
    loading.value = true
    try {
      tags.value = await invoke<TagWithCount[]>('get_tags', { workspaceId })
    } finally {
      loading.value = false
    }
  }

  async function createTag(name: string, color?: string) {
    const tag: Tag = await invoke('create_tag', { name, color })
    return tag
  }

  async function deleteTag(tagId: number) {
    await invoke('delete_tag', { tagId })
  }

  async function batchUpdate(imageIds: number[], tagIds: number[], action: string) {
    await invoke('batch_update_tags', { imageIds, tagIds, action })
  }

  return { tags, loading, fetchTags, createTag, deleteTag, batchUpdate }
})
```

- [ ] **Step 4: selection store**

```typescript
// src/stores/selection.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useSelectionStore = defineStore('selection', () => {
  const selectedIds = ref<Set<number>>(new Set())
  const lastClickedId = ref<number | null>(null)

  const count = computed(() => selectedIds.value.size)
  const hasSelection = computed(() => selectedIds.value.size > 0)
  const all = computed(() => Array.from(selectedIds.value))

  function toggle(id: number, shiftKey: boolean = false) {
    if (shiftKey && lastClickedId.value !== null) {
      // Shift+click: range select (handled by ThumbnailGrid)
    }
    const newSet = new Set(selectedIds.value)
    if (newSet.has(id)) {
      newSet.delete(id)
    } else {
      newSet.add(id)
    }
    selectedIds.value = newSet
    lastClickedId.value = id
  }

  function selectRange(fromId: number, toId: number, allIds: number[]) {
    const from = allIds.indexOf(fromId)
    const to = allIds.indexOf(toId)
    if (from === -1 || to === -1) return
    const [start, end] = from < to ? [from, to] : [to, from]
    const newSet = new Set(selectedIds.value)
    for (let i = start; i <= end; i++) {
      newSet.add(allIds[i])
    }
    selectedIds.value = newSet
  }

  function selectAll(imageIds: number[]) {
    selectedIds.value = new Set(imageIds)
  }

  function clear() {
    selectedIds.value = new Set()
    lastClickedId.value = null
  }

  return { selectedIds, lastClickedId, count, hasSelection, all, toggle, selectRange, selectAll, clear }
})
```

- [ ] **Step 5: thumbnail store**

```typescript
// src/stores/thumbnail.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useThumbnailStore = defineStore('thumbnail', () => {
  const cache = ref<Map<number, string>>(new Map()) // image_id -> object URL
  const loading = ref<Set<number>>(new Set())

  async function load(imageId: number) {
    if (cache.value.has(imageId)) return cache.value.get(imageId)!
    if (loading.value.has(imageId)) return null

    loading.value = new Set([...loading.value, imageId])
    try {
      const bytes = await invoke<number[]>('get_thumbnail', { imageId })
      const uint8 = new Uint8Array(bytes)
      const blob = new Blob([uint8], { type: 'image/webp' })
      const url = URL.createObjectURL(blob)
      const newCache = new Map(cache.value)
      newCache.set(imageId, url)
      cache.value = newCache
      return url
    } catch {
      return null
    } finally {
      const newLoading = new Set(loading.value)
      newLoading.delete(imageId)
      loading.value = newLoading
    }
  }

  function revoke(imageId: number) {
    const url = cache.value.get(imageId)
    if (url) {
      URL.revokeObjectURL(url)
      const newCache = new Map(cache.value)
      newCache.delete(imageId)
      cache.value = newCache
    }
  }

  return { cache, loading, load, revoke }
})
```

- [ ] **Step 6: settings store**

```typescript
// src/stores/settings.ts
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light'>(
    (localStorage.getItem('pic-mgr-theme') as 'dark' | 'light') ?? 'dark'
  )
  const thumbnailSize = ref<number>(
    Number(localStorage.getItem('pic-mgr-thumb-size')) || 200
  )
  const language = ref<string>(
    localStorage.getItem('pic-mgr-lang') ?? 'zh-CN'
  )

  watch(theme, (v) => {
    localStorage.setItem('pic-mgr-theme', v)
    document.documentElement.setAttribute('data-theme', v)
  }, { immediate: true })

  watch(thumbnailSize, (v) => {
    localStorage.setItem('pic-mgr-thumb-size', String(v))
  })

  watch(language, (v) => {
    localStorage.setItem('pic-mgr-lang', v)
  })

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return { theme, thumbnailSize, language, toggleTheme }
})
```

- [ ] **Step 7: Commit**

```bash
git add src/stores/
git commit -m "feat: Pinia stores (workspace, images, tags, selection, thumbnail, settings)"
```

***

### Task 15: 全局样式与 AppLayout

**Files:**

- Modify: `src/style.css`
- Create: `src/components/layout/AppLayout.vue`
- Modify: `src/App.vue`
- [ ] **Step 1: 编写 style.css**

```css
/* src/style.css */
:root {
  --bg-primary: #0f0f14;
  --bg-secondary: #1a1a24;
  --bg-tertiary: #24243a;
  --text-primary: #e8e8ed;
  --text-secondary: #9898b0;
  --text-muted: #686880;
  --accent: #7c6ff7;
  --accent-hover: #9588ff;
  --border: #2a2a3a;
  --danger: #f75c6b;
  --success: #4ade80;
  --warning: #facc15;
  --radius: 8px;
  --transition: 150ms ease;
  --scrollbar-width: 6px;
}

[data-theme="light"] {
  --bg-primary: #fafafa;
  --bg-secondary: #ffffff;
  --bg-tertiary: #f0f0f5;
  --text-primary: #1a1a2e;
  --text-secondary: #6b6b80;
  --text-muted: #9b9bb0;
  --border: #e0e0e8;
}

* { margin: 0; padding: 0; box-sizing: border-box; }

html, body, #app {
  height: 100%;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
  -webkit-font-smoothing: antialiased;
}

::-webkit-scrollbar { width: var(--scrollbar-width); height: var(--scrollbar-width); }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-muted); }

button {
  font-family: inherit;
  cursor: pointer;
  border: none;
  border-radius: var(--radius);
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  transition: all var(--transition);
}

button.primary {
  background: var(--accent);
  color: white;
}
button.primary:hover { background: var(--accent-hover); }

button.secondary {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
}
button.secondary:hover { background: var(--border); }

input, textarea, select {
  font-family: inherit;
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 8px 12px;
  font-size: 13px;
  outline: none;
  transition: border-color var(--transition);
}
input:focus, textarea:focus, select:focus {
  border-color: var(--accent);
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
```

- [ ] **Step 2: 编写 AppLayout.vue**

```vue
<!-- src/components/layout/AppLayout.vue -->
<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings'
import { useRouter, useRoute } from 'vue-router'

const settings = useSettingsStore()
const router = useRouter()
const route = useRoute()

function goHome() { router.push('/') }
function goSettings() { router.push('/settings') }
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-brand" @click="goHome">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="3" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <path d="M21 15l-5-5L5 21" />
        </svg>
        <span class="brand-text">PicMgr</span>
      </div>

      <nav class="sidebar-nav">
        <button
          class="nav-item"
          :class="{ active: route.name === 'home' }"
          @click="goHome"
        >
          工作区
        </button>
        <button
          class="nav-item"
          :class="{ active: route.name === 'settings' }"
          @click="goSettings"
        >
          设置
        </button>
      </nav>

      <div class="sidebar-footer">
        <button class="theme-toggle" @click="settings.toggleTheme()">
          {{ settings.theme === 'dark' ? '☀️' : '🌙' }}
        </button>
      </div>
    </aside>

    <main class="main-content">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  color: var(--accent);
  cursor: pointer;
  border-bottom: 1px solid var(--border);
}

.brand-text {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.sidebar-nav {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  text-align: left;
  background: none;
  color: var(--text-secondary);
  padding: 10px 12px;
  font-size: 14px;
}
.nav-item:hover { background: var(--bg-tertiary); color: var(--text-primary); }
.nav-item.active { background: var(--bg-tertiary); color: var(--accent); }

.sidebar-footer {
  padding: 12px;
  border-top: 1px solid var(--border);
}

.theme-toggle {
  width: 100%;
  background: none;
  font-size: 18px;
  padding: 8px;
}
.theme-toggle:hover { background: var(--bg-tertiary); }

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
```

- [ ] **Step 3: 更新 App.vue**

```vue
<!-- src/App.vue -->
<script setup lang="ts">
import AppLayout from './components/layout/AppLayout.vue'
</script>

<template>
  <AppLayout>
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </router-view>
  </AppLayout>
</template>
```

- [ ] **Step 4: Commit**

```bash
git add src/style.css src/components/layout/AppLayout.vue src/App.vue
git commit -m "feat: global dark theme + AppLayout shell with sidebar navigation"
```

***

### Task 16: WorkspaceHome 视图

**Files:**

- Create: `src/views/WorkspaceHome.vue`
- Create: `src/components/workspace/WorkspaceCard.vue`
- Create: `src/components/workspace/CreateWorkspaceDialog.vue`
- [ ] **Step 1: WorkspaceCard.vue**

```vue
<!-- src/components/workspace/WorkspaceCard.vue -->
<script setup lang="ts">
import type { Workspace } from '../../types'

const props = defineProps<{ workspace: Workspace }>()
const emit = defineEmits<{ select: [id: number]; delete: [id: number] }>()
</script>

<template>
  <div class="card" @click="emit('select', workspace.id)">
    <div class="card-icon">📁</div>
    <div class="card-body">
      <h3>{{ workspace.name }}</h3>
      <p class="path">{{ workspace.path }}</p>
      <p class="date">更新于 {{ workspace.updated_at }}</p>
    </div>
    <button
      class="delete-btn"
      @click.stop="emit('delete', workspace.id)"
      title="删除工作区（不删文件）"
    >×</button>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
}
.card:hover { border-color: var(--accent); transform: translateY(-1px); }
.card-icon { font-size: 32px; }
.card-body { flex: 1; }
.card-body h3 { font-size: 16px; font-weight: 600; margin-bottom: 4px; }
.path { font-size: 12px; color: var(--text-muted); margin-bottom: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 400px; }
.date { font-size: 11px; color: var(--text-muted); }
.delete-btn {
  background: none;
  color: var(--text-muted);
  font-size: 20px;
  padding: 4px 8px;
  border-radius: 4px;
}
.delete-btn:hover { background: var(--danger); color: white; }
</style>
```

- [ ] **Step 2: CreateWorkspaceDialog.vue**

```vue
<!-- src/components/workspace/CreateWorkspaceDialog.vue -->
<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{ create: [name: string, path: string]; close: [] }>()

const name = ref('')
const path = ref('')
const error = ref('')

async function submit() {
  if (!name.value.trim() || !path.value.trim()) {
    error.value = '请填写名称和路径'
    return
  }
  emit('create', name.value.trim(), path.value.trim())
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog">
      <h2>新建工作区</h2>
      <div class="field">
        <label>名称</label>
        <input v-model="name" placeholder="例如：我的照片" @keyup.enter="submit" />
      </div>
      <div class="field">
        <label>文件夹路径</label>
        <input v-model="path" placeholder="D:\Pictures\2024" @keyup.enter="submit" />
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="actions">
        <button class="secondary" @click="emit('close')">取消</button>
        <button class="primary" @click="submit">创建工作区</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 28px;
  width: 440px;
}
.dialog h2 { margin-bottom: 20px; font-size: 18px; }
.field { margin-bottom: 16px; }
.field label { display: block; margin-bottom: 6px; font-size: 13px; color: var(--text-secondary); }
.field input { width: 100%; }
.error { color: var(--danger); font-size: 12px; margin-bottom: 12px; }
.actions { display: flex; gap: 12px; justify-content: flex-end; margin-top: 8px; }
</style>
```

- [ ] **Step 3: WorkspaceHome.vue**

```vue
<!-- src/views/WorkspaceHome.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '../stores/workspace'
import WorkspaceCard from '../components/workspace/WorkspaceCard.vue'
import CreateWorkspaceDialog from '../components/workspace/CreateWorkspaceDialog.vue'

const router = useRouter()
const store = useWorkspaceStore()
const showDialog = ref(false)

onMounted(() => store.fetchAll())

async function handleCreate(name: string, path: string) {
  const ws = await store.create(name, path)
  showDialog.value = false
  await store.scan(ws.id)
  router.push(`/workspace/${ws.id}`)
}

function handleSelect(id: number) {
  router.push(`/workspace/${id}`)
}

async function handleDelete(id: number) {
  if (confirm('确定删除此工作区？不会影响原始图片文件。')) {
    await store.remove(id)
  }
}
</script>

<template>
  <div class="home">
    <header class="home-header">
      <h1>工作区</h1>
      <button class="primary" @click="showDialog = true">+ 新建工作区</button>
    </header>

    <div v-if="store.workspaces.length === 0" class="empty">
      <p>还没有工作区</p>
      <p class="hint">将一个图片文件夹设为工作区，即可开始管理</p>
    </div>

    <div class="grid">
      <WorkspaceCard
        v-for="ws in store.workspaces"
        :key="ws.id"
        :workspace="ws"
        @select="handleSelect"
        @delete="handleDelete"
      />
    </div>

    <CreateWorkspaceDialog
      v-if="showDialog"
      @create="handleCreate"
      @close="showDialog = false"
    />
  </div>
</template>

<style scoped>
.home { padding: 32px; max-width: 900px; margin: 0 auto; height: 100%; overflow-y: auto; }
.home-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.home-header h1 { font-size: 24px; font-weight: 700; }
.empty { text-align: center; padding: 80px 0; color: var(--text-muted); }
.empty .hint { font-size: 14px; margin-top: 8px; }
.grid { display: flex; flex-direction: column; gap: 12px; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/views/WorkspaceHome.vue src/components/workspace/
git commit -m "feat: WorkspaceHome view with create/delete workspace"
```

***

### Task 17: ThumbnailGrid 与 ThumbnailCard

**Files:**

- Create: `src/components/common/VirtualScroller.vue`
- Create: `src/components/browser/ThumbnailCard.vue`
- Create: `src/components/browser/ThumbnailGrid.vue`
- [ ] **Step 1: VirtualScroller.vue**

```vue
<!-- src/components/common/VirtualScroller.vue -->
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  items: any[]
  itemHeight: number
  gap?: number
  buffer?: number
}>(), { gap: 8, buffer: 5 })

const emit = defineEmits<{ loadMore: [] }>()

const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const containerHeight = ref(600)

const totalHeight = computed(() =>
  props.items.length * (props.itemHeight + props.gap)
)

const startIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / (props.itemHeight + props.gap)) - props.buffer)
)

const endIndex = computed(() =>
  Math.min(
    props.items.length,
    Math.ceil((scrollTop.value + containerHeight.value) / (props.itemHeight + props.gap)) + props.buffer
  )
)

const visibleItems = computed(() =>
  props.items.slice(startIndex.value, endIndex.value).map((item, i) => ({
    item,
    index: startIndex.value + i,
    style: {
      position: 'absolute' as const,
      top: `${(startIndex.value + i) * (props.itemHeight + props.gap)}px`,
      height: `${props.itemHeight}px`,
      left: 0,
      right: 0,
    }
  }))
)

function onScroll(e: Event) {
  const el = e.target as HTMLElement
  scrollTop.value = el.scrollTop
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 500) {
    emit('loadMore')
  }
}

onMounted(() => {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight
    const observer = new ResizeObserver(entries => {
      containerHeight.value = entries[0].contentRect.height
    })
    observer.observe(containerRef.value)
  }
})
</script>

<template>
  <div ref="containerRef" class="virtual-scroller" @scroll="onScroll">
    <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
      <div
        v-for="vi in visibleItems"
        :key="vi.index"
        :style="vi.style"
      >
        <slot name="item" :item="vi.item" :index="vi.index" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.virtual-scroller {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}
</style>
```

- [ ] **Step 2: ThumbnailCard.vue**

```vue
<!-- src/components/browser/ThumbnailCard.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useThumbnailStore } from '../../stores/thumbnail'
import type { ImageRecord } from '../../types'

const props = defineProps<{ image: ImageRecord; selected: boolean }>()
const emit = defineEmits<{ click: [id: number, shiftKey: boolean]; dblclick: [] }>()

const thumb = useThumbnailStore()
const thumbUrl = ref<string | null>(null)

onMounted(async () => {
  thumbUrl.value = await thumb.load(props.image.id)
})

onUnmounted(() => {
  thumb.revoke(props.image.id)
})
</script>

<template>
  <div
    class="thumb-card"
    :class="{ selected }"
    @click="emit('click', image.id, $event.shiftKey)"
    @dblclick="emit('dblclick')"
  >
    <div class="thumb-img">
      <img v-if="thumbUrl" :src="thumbUrl" :alt="image.file_name" loading="lazy" />
      <div v-else class="placeholder">
        <span>🖼️</span>
      </div>
    </div>
    <div class="thumb-info">
      <p class="file-name">{{ image.file_name }}</p>
      <div v-if="image.rating" class="rating">
        {{ '★'.repeat(image.rating) }}{{ '☆'.repeat(5 - image.rating) }}
      </div>
    </div>
    <div v-if="selected" class="check-mark">✓</div>
  </div>
</template>

<style scoped>
.thumb-card {
  width: 100%;
  height: 100%;
  background: var(--bg-secondary);
  border: 2px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: border-color var(--transition);
  position: relative;
  display: flex;
  flex-direction: column;
}
.thumb-card:hover { border-color: var(--accent); }
.thumb-card.selected { border-color: var(--accent); background: var(--bg-tertiary); }

.thumb-img {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  min-height: 0;
}
.thumb-img img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.placeholder {
  font-size: 32px;
  color: var(--text-muted);
}

.thumb-info {
  padding: 6px 8px;
  border-top: 1px solid var(--border);
}
.file-name {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rating { font-size: 10px; color: var(--warning); margin-top: 2px; }

.check-mark {
  position: absolute; top: 6px; right: 6px;
  width: 22px; height: 22px;
  background: var(--accent); color: white;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700;
}
</style>
```

- [ ] **Step 3: ThumbnailGrid.vue**

```vue
<!-- src/components/browser/ThumbnailGrid.vue -->
<script setup lang="ts">
import { computed } from 'vue'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useSettingsStore } from '../../stores/settings'
import VirtualScroller from '../common/VirtualScroller.vue'
import ThumbnailCard from './ThumbnailCard.vue'

const props = defineProps<{ workspaceId: number }>()
const emit = defineEmits<{ selectDetail: [id: number] }>()

const imageStore = useImageStore()
const selection = useSelectionStore()
const settings = useSettingsStore()

const itemHeight = computed(() => settings.thumbnailSize + 40)

function handleClick(id: number, shiftKey: boolean) {
  selection.toggle(id, shiftKey)
}

function handleDblClick(id: number) {
  emit('selectDetail', id)
}
</script>

<template>
  <VirtualScroller
    :items="imageStore.images"
    :item-height="itemHeight"
    :gap="12"
    @load-more="imageStore.setPage(imageStore.page + 1); imageStore.fetchImages(workspaceId)"
  >
    <template #item="{ item, index }">
      <div class="grid-cell">
        <ThumbnailCard
          :image="item"
          :selected="selection.selectedIds.has(item.id)"
          @click="handleClick"
          @dblclick="() => handleDblClick(item.id)"
        />
      </div>
    </template>
  </VirtualScroller>
</template>

<style scoped>
.grid-cell { padding: 0 6px; }
</style>
```

- [ ] **Step 4: Commit**

```bash
git add src/components/common/VirtualScroller.vue src/components/browser/ThumbnailCard.vue src/components/browser/ThumbnailGrid.vue
git commit -m "feat: virtual scrolling thumbnail grid with lazy loading"
```

***

### Task 18: SidebarPanel 与筛选组件

**Files:**

- Create: `src/components/browser/TagFilter.vue`
- Create: `src/components/browser/SearchBar.vue`
- Create: `src/components/browser/SidebarPanel.vue`
- [ ] **Step 1: TagFilter.vue**

```vue
<!-- src/components/browser/TagFilter.vue -->
<script setup lang="ts">
import { useTagStore } from '../../stores/tags'
import { useImageStore } from '../../stores/images'

const tagStore = useTagStore()
const imageStore = useImageStore()

function toggleTag(tagId: number) {
  const current = imageStore.filters.tag_ids ?? []
  const idx = current.indexOf(tagId)
  if (idx === -1) {
    imageStore.filters.tag_ids = [...current, tagId]
  } else {
    imageStore.filters.tag_ids = current.filter(id => id !== tagId)
  }
}

function clearTags() {
  imageStore.filters.tag_ids = null
}
</script>

<template>
  <div class="tag-filter">
    <div class="filter-header">
      <span>标签</span>
      <button v-if="imageStore.filters.tag_ids?.length" class="clear-btn" @click="clearTags">清除</button>
    </div>
    <div class="tag-list">
      <button
        v-for="tag in tagStore.tags"
        :key="tag.id"
        class="tag-chip"
        :class="{ active: imageStore.filters.tag_ids?.includes(tag.id) }"
        :style="tag.color ? { '--tag-color': tag.color } : {}"
        @click="toggleTag(tag.id)"
      >
        <span class="tag-dot" :style="tag.color ? { background: tag.color } : {}"></span>
        {{ tag.name }}
        <span class="count">{{ tag.image_count }}</span>
      </button>
    </div>
    <p v-if="tagStore.tags.length === 0" class="empty">暂无标签</p>
  </div>
</template>

<style scoped>
.tag-filter { margin-bottom: 16px; }
.filter-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.filter-header span { font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.clear-btn { background: none; color: var(--accent); padding: 2px 8px; font-size: 11px; }
.tag-list { display: flex; flex-wrap: wrap; gap: 6px; }
.tag-chip {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  padding: 4px 10px;
  border-radius: 100px;
  font-size: 12px;
  display: flex; align-items: center; gap: 6px;
}
.tag-chip:hover { color: var(--text-primary); }
.tag-chip.active { background: var(--accent); color: white; }
.tag-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
.count { font-size: 10px; color: var(--text-muted); }
.tag-chip.active .count { color: rgba(255,255,255,0.7); }
.empty { font-size: 12px; color: var(--text-muted); }
</style>
```

- [ ] **Step 2: SearchBar.vue**

```vue
<!-- src/components/browser/SearchBar.vue -->
<script setup lang="ts">
import { ref } from 'vue'
import { useImageStore } from '../../stores/images'

const props = defineProps<{ workspaceId: number }>()
const imageStore = useImageStore()
const query = ref('')
const debounce = ref<ReturnType<typeof setTimeout> | null>(null)

function onInput() {
  if (debounce.value) clearTimeout(debounce.value)
  debounce.value = setTimeout(() => {
    if (query.value.trim()) {
      imageStore.search(props.workspaceId, query.value.trim())
    } else {
      imageStore.fetchImages(props.workspaceId)
    }
  }, 300)
}
</script>

<template>
  <div class="search-bar">
    <input
      v-model="query"
      placeholder="搜索文件名或备注..."
      @input="onInput"
    />
  </div>
</template>

<style scoped>
.search-bar { margin-bottom: 16px; }
.search-bar input { width: 100%; }
</style>
```

- [ ] **Step 3: SidebarPanel.vue**

```vue
<!-- src/components/browser/SidebarPanel.vue -->
<script setup lang="ts">
import { onMounted } from 'vue'
import { useTagStore } from '../../stores/tags'
import { useImageStore } from '../../stores/images'
import SearchBar from './SearchBar.vue'
import TagFilter from './TagFilter.vue'

const props = defineProps<{ workspaceId: number }>()

const tagStore = useTagStore()
const imageStore = useImageStore()

onMounted(() => {
  tagStore.fetchTags(props.workspaceId)
})

function fetchWithFilters() {
  imageStore.setPage(1)
  imageStore.fetchImages(props.workspaceId)
}

function clearFilters() {
  imageStore.filters.tag_ids = null
  imageStore.filters.format = null
  imageStore.filters.rating_min = null
  imageStore.filters.date_from = null
  imageStore.filters.date_to = null
  imageStore.setPage(1)
  imageStore.fetchImages(props.workspaceId)
}
</script>

<template>
  <div class="sidebar-panel">
    <SearchBar :workspace-id="workspaceId" />
    <TagFilter />
    <button class="secondary" style="width:100%" @click="fetchWithFilters">应用筛选</button>
    <button class="clear-link" @click="clearFilters">清除所有筛选</button>
  </div>
</template>

<style scoped>
.sidebar-panel {
  width: 240px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  padding: 16px;
  overflow-y: auto;
  flex-shrink: 0;
}
.clear-link {
  display: block; width: 100%; margin-top: 8px;
  background: none; color: var(--text-muted);
  font-size: 12px; padding: 4px;
}
.clear-link:hover { color: var(--text-secondary); }
</style>
```

- [ ] **Step 5: Commit**

```bash
git add src/components/browser/
git commit -m "feat: sidebar with tag filter, search bar, filter controls"
```

***

### Task 19: DetailPanel 与详情组件

**Files:**

- Create: `src/components/detail/ImagePreview.vue`
- Create: `src/components/detail/MetadataEditor.vue`
- Create: `src/components/detail/ImageInfo.vue`
- Create: `src/components/common/TagBadge.vue`
- Create: `src/components/browser/DetailPanel.vue`
- [ ] **Step 1: TagBadge.vue**

```vue
<!-- src/components/common/TagBadge.vue -->
<script setup lang="ts">
defineProps<{ name: string; color?: string | null }>()
</script>

<template>
  <span class="tag-badge" :style="color ? { background: color + '22', color: color, borderColor: color } : {}">
    {{ name }}
  </span>
</template>

<style scoped>
.tag-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 100px;
  font-size: 12px;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}
</style>
```

- [ ] **Step 2: ImagePreview\.vue**

```vue
<!-- src/components/detail/ImagePreview.vue -->
<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ fileName: string; thumbnailPath: string | null }>()

// For full preview, we use the original image via a custom protocol or file://
// For now, show thumbnail as placeholder
const displayName = computed(() => props.fileName)
</script>

<template>
  <div class="image-preview">
    <div class="preview-placeholder">
      <span class="icon">🖼️</span>
      <p>{{ displayName }}</p>
      <p class="hint">双击缩略图查看大图</p>
    </div>
  </div>
</template>

<style scoped>
.image-preview {
  background: var(--bg-primary);
  border-radius: var(--radius);
  min-height: 200px;
  display: flex; align-items: center; justify-content: center;
  margin-bottom: 16px;
}
.preview-placeholder { text-align: center; color: var(--text-muted); }
.icon { font-size: 48px; display: block; margin-bottom: 8px; }
.hint { font-size: 11px; margin-top: 4px; }
</style>
```

- [ ] **Step 3: MetadataEditor.vue**

```vue
<!-- src/components/detail/MetadataEditor.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTagStore } from '../../stores/tags'
import type { ImageDetail } from '../../types'
import TagBadge from '../common/TagBadge.vue'

const props = defineProps<{ detail: ImageDetail; workspaceId: number }>()
const emit = defineEmits<{ updated: [] }>()

const tagStore = useTagStore()
const notes = ref(props.detail.notes ?? '')
const rating = ref(props.detail.rating ?? 0)
const saving = ref(false)

watch(() => props.detail.id, () => {
  notes.value = props.detail.notes ?? ''
  rating.value = props.detail.rating ?? 0
})

async function saveNotes() {
  saving.value = true
  try {
    await invoke('update_notes', { imageId: props.detail.id, notes: notes.value })
    emit('updated')
  } finally { saving.value = false }
}

async function setRating(r: number) {
  rating.value = r
  await invoke('update_rating', { imageId: props.detail.id, rating: r })
  emit('updated')
}

async function removeTag(tagId: number) {
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'remove',
  })
  emit('updated')
}

const newTagName = ref('')
async function addTag() {
  if (!newTagName.value.trim()) return
  // Find or create tag
  const existing = tagStore.tags.find(t => t.name === newTagName.value.trim())
  let tagId = existing?.id
  if (!tagId) {
    const tag = await tagStore.createTag(newTagName.value.trim())
    tagId = tag.id
    await tagStore.fetchTags(props.workspaceId)
  }
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'add',
  })
  newTagName.value = ''
  emit('updated')
}
</script>

<template>
  <div class="metadata-editor">
    <!-- Rating -->
    <div class="field">
      <label>评分</label>
      <div class="stars">
        <button
          v-for="n in 5" :key="n"
          class="star" :class="{ active: n <= rating }"
          @click="setRating(n)"
        >{{ n <= rating ? '★' : '☆' }}</button>
      </div>
    </div>

    <!-- Tags -->
    <div class="field">
      <label>标签</label>
      <div class="tags-row">
        <TagBadge
          v-for="tag in detail.tags"
          :key="tag.id"
          :name="tag.name"
          :color="tag.color"
          @click="removeTag(tag.id)"
          title="点击移除"
        />
      </div>
      <div class="add-tag-row">
        <input
          v-model="newTagName"
          placeholder="添加标签..."
          @keyup.enter="addTag"
          style="flex:1"
        />
        <button class="primary" @click="addTag" style="padding:8px 12px">+</button>
      </div>
    </div>

    <!-- Notes -->
    <div class="field">
      <label>备注</label>
      <textarea
        v-model="notes"
        rows="4"
        placeholder="添加备注..."
        @blur="saveNotes"
      ></textarea>
    </div>

    <p v-if="saving" class="saving">保存中...</p>
  </div>
</template>

<style scoped>
.metadata-editor { padding: 8px 0; }
.field { margin-bottom: 20px; }
.field label { display: block; margin-bottom: 8px; font-size: 12px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.stars { display: flex; gap: 4px; }
.star { background: none; font-size: 24px; padding: 0; color: var(--text-muted); }
.star.active { color: var(--warning); }
.tags-row { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.add-tag-row { display: flex; gap: 8px; }
textarea { width: 100%; resize: vertical; min-height: 80px; }
.saving { font-size: 12px; color: var(--accent); }
</style>
```

- [ ] **Step 4: ImageInfo.vue**

```vue
<!-- src/components/detail/ImageInfo.vue -->
<script setup lang="ts">
import type { ImageRecord } from '../../types'

defineProps<{ image: ImageRecord }>()

function formatSize(bytes: number | null): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}
</script>

<template>
  <div class="image-info">
    <div class="info-row">
      <span class="label">文件名</span>
      <span>{{ image.file_name }}</span>
    </div>
    <div class="info-row">
      <span class="label">格式</span>
      <span>{{ image.format?.toUpperCase() ?? '-' }}</span>
    </div>
    <div class="info-row">
      <span class="label">尺寸</span>
      <span>{{ image.width }} × {{ image.height }}</span>
    </div>
    <div class="info-row">
      <span class="label">文件大小</span>
      <span>{{ formatSize(image.file_size) }}</span>
    </div>
    <div class="info-row">
      <span class="label">索引时间</span>
      <span>{{ image.indexed_at }}</span>
    </div>
  </div>
</template>

<style scoped>
.image-info { padding: 8px 0; border-top: 1px solid var(--border); }
.info-row {
  display: flex; justify-content: space-between;
  padding: 6px 0; font-size: 12px;
}
.label { color: var(--text-muted); }
</style>
```

- [ ] **Step 5: DetailPanel.vue**

```vue
<!-- src/components/browser/DetailPanel.vue -->
<script setup lang="ts">
import { useImageStore } from '../../stores/images'
import ImagePreview from '../detail/ImagePreview.vue'
import MetadataEditor from '../detail/MetadataEditor.vue'
import ImageInfo from '../detail/ImageInfo.vue'

const props = defineProps<{ workspaceId: number }>()
const imageStore = useImageStore()

async function handleUpdated() {
  if (imageStore.selectedDetail) {
    await imageStore.fetchDetail(imageStore.selectedDetail.id)
    await imageStore.fetchImages(props.workspaceId)
  }
}
</script>

<template>
  <div class="detail-panel" v-if="imageStore.selectedDetail">
    <div class="panel-header">
      <h3>图片详情</h3>
      <button class="close-btn" @click="imageStore.selectedDetail = null">×</button>
    </div>
    <div class="panel-body">
      <ImagePreview
        :file-name="imageStore.selectedDetail.file_name"
        :thumbnail-path="imageStore.selectedDetail.thumbnail_path"
      />
      <MetadataEditor
        :detail="imageStore.selectedDetail"
        :workspace-id="workspaceId"
        @updated="handleUpdated"
      />
      <ImageInfo :image="imageStore.selectedDetail" />
    </div>
  </div>
</template>

<style scoped>
.detail-panel {
  width: 320px;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.panel-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px; border-bottom: 1px solid var(--border);
}
.panel-header h3 { font-size: 14px; }
.close-btn { background: none; font-size: 18px; color: var(--text-muted); padding: 4px 8px; }
.close-btn:hover { background: var(--bg-tertiary); }
.panel-body { flex: 1; overflow-y: auto; padding: 16px; }
</style>
```

- [ ] **Step 6: Commit**

```bash
git add src/components/common/TagBadge.vue src/components/detail/ src/components/browser/DetailPanel.vue
git commit -m "feat: detail panel with preview, metadata editor, and image info"
```

***

### Task 20: BrowserView 主视图

**Files:**

- Create: `src/views/BrowserView.vue`
- [ ] **Step 1: 编写 BrowserView\.vue**

```vue
<!-- src/views/BrowserView.vue -->
<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useWorkspaceStore } from '../stores/workspace'
import { useImageStore } from '../stores/images'
import { useSelectionStore } from '../stores/selection'
import { useTagStore } from '../stores/tags'
import SidebarPanel from '../components/browser/SidebarPanel.vue'
import ThumbnailGrid from '../components/browser/ThumbnailGrid.vue'
import DetailPanel from '../components/browser/DetailPanel.vue'

const route = useRoute()
const workspaceStore = useWorkspaceStore()
const imageStore = useImageStore()
const selection = useSelectionStore()
const tagStore = useTagStore()

const workspaceId = computed(() => Number(route.params.id))

onMounted(async () => {
  await workspaceStore.fetchAll()
  workspaceStore.select(workspaceId.value)
  await imageStore.fetchImages(workspaceId.value)
  await tagStore.fetchTags(workspaceId.value)
})

// Keyboard shortcuts
function onKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

  switch (e.key) {
    case 'Escape':
      selection.clear()
      imageStore.selectedDetail = null
      break
    case 'a':
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault()
        selection.selectAll(imageStore.images.map(i => i.id))
      }
      break
  }
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onUnmounted(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <div class="browser">
    <SidebarPanel :workspace-id="workspaceId" />
    <div class="grid-area">
      <div class="grid-header">
        <span class="image-count">{{ imageStore.total }} 张图片</span>
        <span v-if="selection.count > 0" class="selection-info">
          已选 {{ selection.count }} 张
        </span>
      </div>
      <ThumbnailGrid
        :workspace-id="workspaceId"
        @select-detail="(id: number) => imageStore.fetchDetail(id)"
      />
    </div>
    <DetailPanel v-if="imageStore.selectedDetail" :workspace-id="workspaceId" />
  </div>
</template>

<style scoped>
.browser {
  display: flex;
  height: 100%;
}
.grid-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.grid-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}
.image-count { font-size: 13px; color: var(--text-secondary); }
.selection-info { font-size: 12px; color: var(--accent); font-weight: 500; }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/BrowserView.vue
git commit -m "feat: BrowserView main view with sidebar + grid + detail panel layout"
```

***

### Task 21: SettingsPage 视图

**Files:**

- Create: `src/views/SettingsPage.vue`
- [ ] **Step 1: 编写 SettingsPage.vue**

```vue
<!-- src/views/SettingsPage.vue -->
<script setup lang="ts">
import { useSettingsStore } from '../stores/settings'

const settings = useSettingsStore()
</script>

<template>
  <div class="settings-page">
    <h1>设置</h1>

    <div class="section">
      <h3>外观</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">主题</span>
          <span class="desc">暗色或亮色界面主题</span>
        </div>
        <select v-model="settings.theme">
          <option value="dark">暗色</option>
          <option value="light">亮色</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-label">
          <span class="title">缩略图尺寸</span>
          <span class="desc">缩略图网格中每张图的高度 (像素)</span>
        </div>
        <div class="size-control">
          <input
            type="range"
            min="120"
            max="400"
            step="20"
            v-model.number="settings.thumbnailSize"
          />
          <span class="size-value">{{ settings.thumbnailSize }}px</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>语言</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">界面语言</span>
          <span class="desc">应用界面显示语言</span>
        </div>
        <select v-model="settings.language">
          <option value="zh-CN">简体中文</option>
          <option value="en">English</option>
        </select>
      </div>
    </div>

    <div class="section">
      <h3>关于</h3>
      <div class="about">
        <p><strong>PicMgr</strong> v0.1.0</p>
        <p class="desc">轻量本地图片管理器</p>
        <p class="desc">Tauri + Vue 3 + SQLite</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 32px;
  max-width: 640px;
  margin: 0 auto;
  height: 100%;
  overflow-y: auto;
}
h1 { font-size: 24px; font-weight: 700; margin-bottom: 28px; }
.section { margin-bottom: 28px; }
.section h3 { font-size: 14px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 16px; }
.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 8px;
}
.setting-label .title { font-size: 14px; font-weight: 500; }
.setting-label .desc { font-size: 12px; color: var(--text-muted); display: block; margin-top: 2px; }
select {
  min-width: 120px;
}
.size-control { display: flex; align-items: center; gap: 12px; }
.size-control input[type="range"] { width: 120px; accent-color: var(--accent); }
.size-value { font-size: 13px; color: var(--text-secondary); min-width: 50px; }
.about p { margin-bottom: 4px; }
.about .desc { font-size: 12px; color: var(--text-muted); }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/views/SettingsPage.vue
git commit -m "feat: settings page with theme, thumbnail size, and language options"
```

***

### Task 22: 最终整合与验证

- [ ] **Step 1: 全量编译 Rust**

```bash
cd src-tauri && cargo build
```

Expected: 编译成功，无错误。

- [ ] **Step 2: 前端 TypeScript 检查**

```bash
npx vue-tsc --noEmit
```

Expected: 无类型错误。如有，修复。

- [ ] **Step 3: 启动开发模式验证**

```bash
npm run tauri dev
```

Expected: 应用启动，显示 WorkspaceHome 页面，可创建工作区、浏览图片。

- [ ] **Step 4: 检查 .gitignore 包含必要条目**

```
node_modules/
dist/
src-tauri/target/
.superpowers/
```

- [ ] **Step 5: 最终 Commit**

```bash
git add -A
git commit -m "feat: complete pic-mgr implementation"
```

***

## 后续迭代 (Scope C+)

| 功能                          | 优先级 |
| --------------------------- | --- |
| 图片去重检测（利用已有 `file_hash` 字段） | 中   |
| 右键菜单（打开文件夹、复制路径）            | 中   |
| 拖拽导入标签                      | 低   |
| 自定义缩略图尺寸滑块                  | 低   |
| 导入/导出工作区配置                  | 低   |
| i18n 多语言支持                  | 低   |

