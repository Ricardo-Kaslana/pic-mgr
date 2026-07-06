# AGENTS.md

This file provides guidance to the AI agent when working with code in this repository.

## Invariants

- **Never modify original image files.** All metadata (tags, ratings, notes) goes into SQLite only.
- DedupView is embedded inside BrowserView as a child component, not a separate route (Vue Router nesting causes black screen).
- Image dimensions are null at scan time; they get filled only after thumbnail generation.
- DB migrations run at startup via `ALTER TABLE ADD COLUMN` with `.ok()` to ignore "already exists" errors — no migration framework.
- **Workspace deletion MUST handle cascade manually**: delete `image_tags` → `images` → `workspace` in a single explicit `BEGIN`/`COMMIT` transaction. Do NOT rely on `ON DELETE CASCADE` for workspace deletion — the cascade chain (FTS triggers + FK constraints) can cause `SQLITE_CORRUPT`.
- **FTS5 triggers use rowid-only delete**: `INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id)` — do NOT include content columns on delete, as content verification mismatches silently break cascade operations.

## Build & Check

```bash
npm run tauri dev            # full dev server (frontend + Rust backend)
npm run tauri build          # production build
cd src-tauri && cargo check  # Rust-only compile check (fast feedback)
```

No test suite exists. There is no linter configured beyond vue-tsc.

## Architecture Gotchas

- `AppState` is injected via `app.manage()` and accessed in commands via `State<AppState>`. The DB connection is `Mutex<Connection>`.
- Scanner uses only filesystem metadata + SHA-256 hashes — do NOT use the `image` crate to read dimensions during scan (causes stack overflow on some files).
- Thumbnail generation wraps `image::open()` in `catch_unwind` because `image-rs` can panic on malformed files.
- Thumbnail format detection uses `with_guessed_format()` (file header magic bytes), not file extension.
- LRU thumbnail cache holds up to 500 blob URLs; scrolling out of viewport triggers revoke.
- Append-mode pagination (`append=true`): new pages are added to existing data, never replace it.
- File dates: `created_at` = filesystem mtime; `file_created_at` = birth time; `indexed_at` = time added to DB.
- Delete uses the `trash` crate (system recycle bin), not `fs::remove_file`.
- Move uses `fs::rename`, falling back to copy+delete for cross-device moves; filename conflicts get `_1`, `_2` suffixes.
- "Remove from workspace" deletes only the DB record — it does not touch the file on disk.

## i18n

All user-facing strings go through vue-i18n. Add both `zh-CN` and `en` translations in `src/i18n/`.

## Rust Backend (src-tauri)

- Edition 2021, Tauri v2 (uses `@tauri-apps/api` v2 on the frontend, `tauri` v2 crate on the backend).
- `rusqlite` with `bundled` feature (compiles SQLite from source).
- Commands are in `src/commands/` split by domain: `images.rs`, `metadata.rs`, `export.rs`.
