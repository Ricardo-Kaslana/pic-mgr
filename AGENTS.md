# AGENTS.md

This file provides guidance to the AI agent when working with code in this repository.

## Invariants

- **Never modify original image files.** All metadata (tags, ratings, notes) goes into SQLite only.
- DedupView is embedded inside BrowserView as a child component, not a separate route (Vue Router nesting causes black screen).
- Image dimensions are null at scan time; they get filled only after thumbnail generation.
- DB migrations run at startup via `ALTER TABLE ADD COLUMN` with `.ok()` to ignore "already exists" errors — no migration framework.
- **Workspace deletion MUST handle cascade manually**: delete `image_tags` → `images` → `workspace` in a single explicit `BEGIN`/`COMMIT` transaction. Do NOT rely on `ON DELETE CASCADE` for workspace deletion — the cascade chain (FTS triggers + FK constraints) can cause `SQLITE_CORRUPT`.
- **FTS5 triggers use rowid-only delete**: `INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id)` — do NOT include content columns on delete, as content verification mismatches silently break cascade operations.
- **Grid uses `ImageSummary`, detail uses `ImageRecord`**: the grid (`images` store, `ThumbnailGrid`, `LightboxViewer`) only ever reads `id`, `workspace_id`, `relative_path`, `file_name`, `rating`, and the thumbnail fields. The right-side `ImageInfo` (in `DetailPanel`) reads the full `ImageRecord` from `imageStore.selectedDetail`. Do NOT widen `ImageSummary` to add fields the grid does not render — every added column is paid for on every workspace open.
- **`thumbUrls` is owned by `thumbStore.cache`, not the grid component**: the local `shallowRef<Map>` in `ThumbnailGrid` is a render-time projection of the store cache. The store cache is the source of truth for blob URL lifetime (LRU). `flushBatch` MUST pull URLs from `thumbStore.cache` for every id in the batch (not just the `loaded` subset that `batchLoad` returns), and `evictAllThumbUrls` MUST NOT revoke URLs still held by `thumbStore.cache` — otherwise cached thumbnails go blank on the next workspace re-entry.

- **Floating scroll FABs are siblings of `.thumb-grid`, not children**: the `.scroll-fab` element must be a direct child of `.thumb-grid-outer` (alongside `.thumb-grid`) — positioned `absolute` relative to the wrapper, which does NOT scroll, so the buttons always stay pinned to the visible viewport. Placing them inside `.thumb-grid` (the scroll container) makes them scroll with the content in some browser configurations; `.thumb-grid-outer` has `position: relative` to anchor the FAB, and `overflow: hidden` to clip overflow.
- **FAB visibility state is one reactive bag updated on scroll + resize + totalHeight changes**: `scrollInfo` (a single `ref({ canScroll, scrollTop, distanceFromBottom })`) is mutated by an `updateScrollInfo()` helper called from `onScroll`, `ResizeObserver`, and a `watch(totalHeight)` callback. Do NOT split this into separate refs (more reactivity overhead) or computed properties over `containerRef.value.clientHeight` (not reactive). Threshold constants (`SCROLL_FAB_THRESHOLD`) live at module top, not inline in the template.

- **Top-of-column areas share an aligned bottom edge in browser view**: the outer sidebar brand area (`.sidebar-brand` in AppLayout.vue), the inner sidebar search section (`.search-section` in SidebarPanel.vue), and the main grid header (`.grid-header` in GridToolbar.vue) must all end at the same y-coordinate, in both expanded and collapsed states. The formula is `total height = padding-top + content + padding-bottom + 1px border-bottom`. The `.search-section` is a `flex-direction: column` wrapper around `<SearchBar>` + `.section-divider`; the divider uses `margin-top: auto` so its 1px line is pinned to the section's bottom (i.e. y = sidebar-panel padding-top + `.search-section` height). When `.app-layout.collapsed` toggles, update grid-header padding and `.search-section` height together - `brand-icon` size shrinks from 34px to 22px and the brand area collapses from 71px to 55px, so the other two areas must scale to match. Current values (post v0.3.13): brand area 71/55, grid-header 67/51 (deliberately ~4px shorter for visual hierarchy), search-section 57/41 (71-14 / 55-14).

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
- Thumbnail format: JPEG quality 80 (was WebP lossless). Cache files stored as `{id}.jpg`.
- Thumbnail format detection uses `with_guessed_format()` (file header magic bytes), not file extension.
- LRU thumbnail cache holds up to 500 blob URLs (`thumbStore.cache`); scrolling out of viewport triggers revoke via `thumbStore.revoke`.
- Thumbnail loading: `BATCH_CHUNK_SIZE = 60` (single IPC for the first viewport), `FALLBACK_CONCURRENCY = 4`, plus a 6-row prefetch zone below viewport. Backend `get_thumbnails_batch` uses a single `WHERE id IN (...)` SQL query.
- `thumbUrls` is `shallowRef(new Map())` + `triggerRef`, NOT `reactive(new Map())`. `reactive` proxies every entry which dominates cost at 100+ entries; `shallowRef` + manual `triggerRef` is O(1) per batch.
- Append-mode pagination (`append=true`): new pages are added to existing data, never replace it. `ThumbnailGrid.onScroll` triggers the next page at 65% scrolled (was: within 500 px of bottom).
- Default `pageSize = 100` (was 50). 100+ image workspaces render in a single round-trip.
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
- **SQLite PRAGMAs at startup** (`db/mod.rs`): `journal_mode=WAL`, `synchronous=NORMAL`, `cache_size=-20000`, `temp_store=MEMORY`, `mmap_size=268435456`, plus `ANALYZE`. These are tuned for our read-heavy browse + occasional bulk thumbnail-batch workload.
- **Composite indexes for the workspace+sort hot path** (`db/schema.rs`): `(workspace_id, indexed_at|updated_at|file_name|file_size|rating|created_at|file_created_at)`, plus `(workspace_id, format)` and `(workspace_id, file_hash)`. The single-column `idx_images_workspace_id` and `idx_image_tags_tag_id` are still there for cases where the leading column is the only filter.
- **N+1-free query layer** (`db/queries.rs`):
  - `get_tags` uses one `GROUP BY` join (no per-tag correlated subquery).
  - `find_duplicate_groups` uses a CTE + JOIN (no per-hash query loop).
  - `get_image_paths` does a single `WHERE id IN (...)` lookup for batch thumbnail generation.
  - `get_images` / `get_images_light` share a `build_filter_clause` helper that produces parameterized fragments for both the paginated SELECT and the COUNT(*) — no dynamic SQL string-replace hacks.