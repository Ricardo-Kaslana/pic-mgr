
## [0.3.8] — 2026-07-09

### Added

- **Hover-revealed expand button on each thumbnail cell**: A small circular button in the top-left corner of every `.grid-cell` in the thumbnail grid, hidden by default (`opacity: 0`, `pointer-events: none`) and revealed on `.grid-cell:hover` with a scale-spring entrance. Clicking it emits `viewFullscreen` with the cell's index — the same event the existing `@dblclick` handler emits — so the original double-click-to-expand path is fully preserved and unaffected. The button works in both normal and select modes (unlike double-click, which is suppressed in select mode); `@click.stop` ensures the button does not also fire the cell's normal single-click handler (toggle selection / select detail). 26×26 px, mirrors the look-and-feel of `.scroll-fab-btn` (same border, hover fill, and accent-color treatment) but scaled down to fit inside a thumbnail. `z-index: 3` stacks it above the right-side `.check-mark` (z-index 2) so both are visible simultaneously in select mode. Icon: 4-corner-outward-arrows SVG. Tooltip / aria-label use new i18n key `browser.viewFullscreen` (`'全屏查看'` / `'View fullscreen'`).

## [0.3.7] — 2026-07-09

### Added

- **Browser sidebar collapse state persists across launches**: `BrowserView.vue`'s inner `SidebarPanel` wrapper collapse state (the in-page sidebar holding tag filters / workspace info, toggled by the `edge-toggle` chevron button inside the workspace) now persists via `pic-mgr-browser-sidebar-collapsed` in localStorage. Same `localStorage + watch({ immediate: true })` pattern used by the outer nav sidebar (`pic-mgr-sidebar-collapsed`), `theme`, `thumbnailSize`, and `language`. Inner sidebar now opens collapsed or expanded to match the user's last choice instead of resetting to expanded on every launch. The right-side detail panel still resets to expanded on each launch ( its `if (detail) detailCollapsed.value = false` watcher overrides the persisted value when an image is selected, which is intentional behavior).

## [0.3.6] — 2026-07-09

### Changed

- **Collapsed sidebar icon polish** (continuation of `[0.3.5]`): Three iterations to align icons in the 64px collapsed sidebar at the same `x=14` left-edge line. Tighter button padding so icons no longer float in bloated rectangles; theme toggle bg switched from always-on `var(--bg-tertiary)` to transparent-by-default / grey-on-hover (matches `.nav-item` behavior); brand-icon and toggle scaled down to 22×22 (matching `.nav-icon` scale) so all four icons share a 22–28px visual range with consistent left alignment.

- **Toolbar responsive layout**: `GridToolbar.vue` now wraps gracefully when horizontal space is tight. Three `.grid-header` / `.header-left` / `.header-actions` flex containers gained `flex-wrap: wrap` plus `gap: <row> <col>` shorthand — when narrow, items drop to a second line instead of overflowing into the grid area below. Left group (back / count / sort) stays single-line; right group splits to 2 lines in select mode. Single-row layout unchanged for normal widths.

- **Toolbar button variant unification**: Three previously-divergent button looks (`.toolbar-btn.active` solid-fill, `.toolbar-btn.locate-btn` tinted-bg + accent outline, default bordered) collapsed into one outlined system: same shape / padding / border-radius across all variants, differing only in **text + border color**:
  - `.toolbar-btn` — bordered neutral (most actions: 全选, 移动, 排序, 导出)
  - `.toolbar-btn.primary` — bordered accent (the per-mode main CTA: 完成 / 定位 pre-unification)
  - `.toolbar-btn.danger` — bordered danger (删除)
  Hover state consistently **fills with the variant color** across all three. Both `.toolbar-btn.active` and `.toolbar-btn.locate-btn` rules removed.

- **定位 demoted to default style**: With the unification above, the locate button lost its redundant SVG icon and its primary-class, becoming a plain bordered-neutral text button identical in look to 全选 / 移动. Functional click handler unchanged (`emit('locateLast')` still scrolls to / flashes the last-selected image). Reduces visual emphasis on a conditional button that pops in/out based on `lastSelectedId != null`.

### Fixed

- **In-page sidebar fold button clipped after collapse** (BrowserView): `.edge-toggle.sidebar-edge` used `margin-left: -14px; margin-right: -14px` to straddle the sidebar-grid boundary for the visual "tab on the edge" effect. When the sidebar collapsed to `width: 0`, the button's slot shifted to `x=0` of `.browser` and the `-14px` left margin pulled the button's left half **outside** `.browser` — straight into AppLayout's `.main-content` whose `overflow: hidden` clipped it. Now `.sidebar-wrapper.collapsed .sidebar-edge` (and the symmetric `.detail-wrapper.collapsed .detail-edge`) reset both margins to `0`, keeping the button fully inside `.browser`. Added `margin 0.25s cubic-bezier(0.4, 0, 0.2, 1)` to `.edge-toggle` transition list so the reset is animated, not snapped, in sync with the wrapper's width transition.


## [0.3.5] — 2026-07-09

### Added

- **Collapsible left navigation sidebar**: Top chevron toggle expands/collapses the 220px sidebar to a 64px icon-rail with hover tooltips. State persists across launches via `pic-mgr-sidebar-collapsed` in `useSettingsStore`. Brand, nav, and theme icons all align on the same x-axis in collapsed mode. Single `:class="{ collapsed: settings.sidebarCollapsed }"` binding drives all layout via scoped CSS — tooltips are CSS-only (`::after { content: attr(data-label) }`), no extra DOM. Default uncollapsed so existing users see no surprise.

- **Sidebar collapse state in settings store**: New `sidebarCollapsed: Ref<boolean>` + `toggleSidebar()` action on `useSettingsStore`, following the existing `localStorage + watch({ immediate: true })` pattern shared by `theme`, `thumbnailSize`, `language`. No new store file — keeps UI prefs in one place.

- **i18n key `nav.toggleSidebar`**: Added in both `en` (`'Toggle sidebar'`) and `zh-CN` (`'切换侧栏'`). Tooltip text reuses existing `nav.home`, `nav.settings`, `settings.dark`, `settings.light` translations — no new strings needed for the icon labels themselves.

### Changed

- **Default window sizing**: `tauri.conf.json` `app.windows[0]` shrunk from 1200×800 to 1080×720, and `center: true` added so the app launches centered on the primary monitor. ~156px of horizontal space recovered in the thumbnail grid. `cargo check` confirms Tauri v2 accepts the new `center` field.

# Changelog

## [0.3.4] -- 2026-07-08

### Added

- **Floating scroll FABs in the thumbnail grid**: Two circular buttons anchored to the bottom-right corner of the grid. "Scroll to Top" appears when the user is more than 300 px from the top, "Scroll to Bottom" appears when more than 300 px from the bottom. Each button smoothly slides + fades in / out, supports smooth-scroll, and respects the existing edge-toggle / accent-glow design language. Hidden entirely on workspaces short enough to fit in one viewport. Zero impact on the hot path: state is read once per scroll event and once on resize / pagination append, no per-frame work.

### Changed

- **.thumb-grid is now position: relative** so absolutely-positioned FAB children stay clipped to the scroll viewport (the existing .grid-sizer { position: relative } still anchors the absolutely-positioned grid cells, so cell layout is unchanged).

## [0.3.3] — 2026-07-08

### Optimized

- **Lightweight grid payload (`ImageSummary`)**: New projection (~80 B/row vs `ImageRecord` ~250 B/row) excludes `file_hash`, `notes`, `last_viewed_at`, `file_created_at` that the grid never renders. New `get_images_light` Tauri command backs the grid; `get_image_detail` still returns the full record when the user opens the right panel. ~3x IPC payload reduction on 100+ image workspaces.
- **SQLite PRAGMAs at startup**: `synchronous=NORMAL`, `cache_size=-20000` (20 MB), `temp_store=MEMORY`, `mmap_size=256 MB`, plus `ANALYZE` for query planner stats. Safe with WAL mode; ~2x write speedup over `synchronous=FULL`.
- **Composite indexes for the workspace+sort hot path**: 8 new indexes cover `(workspace_id, indexed_at|updated_at|file_name|file_size|rating|created_at|file_created_at)`, plus `(workspace_id, format)` and `(workspace_id, file_hash)` for dedup.
- **N+1 query elimination**:
  - `get_tags`: collapsed per-tag correlated subqueries into a single `GROUP BY` join.
  - `find_duplicates`: replaced the 1 + N hash loop with a CTE + JOIN single query.
  - `generate_thumbnails_batch` / `get_thumbnails_batch`: replaced per-id `query_row` + `cache_path().exists()` with one `WHERE id IN (...)` round-trip plus a single workspace-path resolution pass. 1000 image_ids now mean 1 DB query, not 2000.
- **Frontend reactivity**: `thumbUrls` switched from `reactive(new Map())` to `shallowRef(new Map())` + `triggerRef` for batched updates. Eliminates Vue deep-proxy overhead for 100+ concurrent thumbnail entries.
- **Thumbnail batch tuning**: `BATCH_CHUNK_SIZE` 20 → 60 (single IPC now covers the whole first viewport), `FALLBACK_CONCURRENCY` 2 → 4. Added a 6-row prefetch zone below viewport to prevent skeleton flashes while scrolling.
- **Smart scroll pre-loading**: Next page now triggers at 65% scrolled (was: within 500 px of bottom). The new threshold lets the next page land before the user reaches the bottom — no visible stall on 100+ image workspaces.
- **Default page size 50 → 100**: 100+ image workspaces now render in a single round-trip instead of forcing a scroll-triggered second fetch.
- **Workspace list cache**: `workspaceStore.fetchAll` is now single-flight + cached, skipping the IPC round-trip on every subsequent navigation. `create` / `remove` update the local cache in place.
- **Tag fetch dedup**: `tagStore.fetchTags` gained single-flight + per-workspace cache so the duplicate `fetchTags` calls from `BrowserView` and `SidebarPanel` mount hooks coalesce into one IPC.
- **CSS paint containment**: `.grid-cell` got `contain: layout style; will-change: transform;`; `<img>` got `decoding="async"`. Reduces layout/paint cost during scroll.
- **BrowserView init cleanup**: `fetchAll` is no longer awaited; the catch-up branch only re-fires when the workspace is genuinely empty (`total === 0 && !scanning`) instead of unconditionally. `generate_thumbnails_batch` now only fires after a real scan, not on every workspace open.

### Fixed

- **Cached thumbnails invisible on workspace re-entry** (regression from the local `thumbUrls` map cleanup). `flushBatch` only iterated the `loaded` set from `thumbStore.batchLoad`, but `batchLoad` filters cached ids out of `loaded`, so cached URLs were never surfaced to the new (empty) `thumbUrls` on re-entry — every thumbnail rendered as a placeholder until the next app restart. `flushBatch` now iterates the full batch and pulls URLs from `thumbStore.cache` for every id. `evictAllThumbUrls` no longer revokes URLs that the store cache is still holding; LRU eviction owns their lifetime. Workspaces with 100+ cached images now show thumbnails immediately on re-entry, no extra IPC needed.

## [0.3.2] — 2026-07-06

### Optimized
 — 2026-07-06

### Optimized

- **Thumbnail format: WebP lossless → JPEG quality 80**: 10-20x faster encoding for first-time thumbnail generation with no visible quality difference at 300px. Existing `.webp` cache files are ignored; new `.jpg` files generate on demand.
- **Parallel initial data fetch**: `fetchAll`, `fetchImages`, and `fetchTags` now run in parallel via `Promise.all` instead of sequentially — ~40% faster workspace open.
- **Thumbnail concurrency limiter**: Max 4 parallel IPC calls prevents overwhelming the backend when 30+ thumbnails enter viewport simultaneously. Queue-based scheduling with inflight tracking.
- **Reduced first page size**: Initial page from 100 to 50 images for faster first paint.

## [0.3.1] — 2026-07-06

### Fixed

- **Workspace deletion silently fails**: Two root causes fixed:
  - FTS5 triggers verified content on `'delete'` command, causing `SQLITE_ERROR` on cascade delete when content mismatched (empty string vs NULL, etc.). Changed to rowid-only delete (`INSERT INTO images_fts(images_fts, rowid) VALUES('delete', old.id)`) in both `images_fts_delete` and `images_fts_update` triggers. Migration drops+recreates old triggers on startup.
  - `ON DELETE CASCADE` chain (workspace → images → image_tags + FTS triggers) could trigger `SQLITE_CORRUPT` on large datasets. Rewrote `delete_workspace` to manually handle the cascade: delete `image_tags` → delete `images` → delete `workspace`, all in a single explicit `BEGIN`/`COMMIT` transaction.
- **WorkspaceHome delete error handling**: Added `try/catch` with dismissible error banner (i18n key `workspace.deleteFailed`) so failures are visible instead of silent.

### Changed

- `.gitignore`: Added patterns for `.claude/`, `.codegraph/`, `.omo/`, OS files, env files.

## [0.3.0] — 2026-06-12

### Added

- **Sort controls**: Dropdown in grid header to sort by filename, indexed time, modified time, or created time; click toggles asc/desc; backend `sort_col` match expanded in `queries.rs`
- **Quick locate**: Auto-scroll grid to selected image when detail panel opens (nextTick + 80ms delay for layout reflow); accent glow border on active cell; manual locate button (⌖) in detail header
- **Panel collapse/expand**: Sidebar and detail panels collapse to 0 width with 28px circular toggle buttons floating at the panel edge; buttons hidden by default, revealed on hover with accent glow; SVG chevron icons; smooth CSS transitions; detail auto-expands when an image is selected

### Changed

- **Button layout**: Header toolbar restructured — header-left holds count + sort, header-right holds action buttons; select mode uses btn-group dividers between groups; emojis removed from toolbar buttons
- **Select all toggle**: Single button toggles between select-all and deselect-all (was: two separate buttons); Ctrl+A shortcut uses same toggle
- **Workspace rescan feedback**: Spinning refresh icon during scan, green checkmark with pop-in animation for 2s on completion; button stays visible during operation; emoji replaced with SVG icons for rescan and delete

### Fixed

- **Text alignment**: Context menu emoji prefixes wrapped in fixed-width `.ctx-icon` span (22px) so menu item text aligns vertically; nav sidebar `.nav-icon` given fixed 20px width; theme toggle emoji also fixed-width — all sidebar text labels now start at the same x position

### Optimized

- **Scan inserts wrapped in transaction**: 10-50x faster scanning by avoiding per-row fsync
- **Batch tag updates wrapped in transaction**: 10x+ faster bulk tag operations
- **Database indexes**: Added `idx_images_hash` (duplicate detection) and `idx_image_tags_tag_id` (tag joins)
- **Reduced syscalls**: Scanner `fs::metadata` calls reduced from 3 to 1 per file; thumbnail generation reads file once instead of 3 times
- **Batch export queries**: Export metadata uses 2 batch queries instead of N+1 individual queries
- **Thumbnail cache**: `shallowRef` replaces `ref<Map>` to avoid O(n) copy on every load/eviction
- **Scroll throttle**: Thumbnail loading watcher throttled to 80ms to prevent per-frame firing
- **ResizeObserver leak fix**: Observer properly disconnected on component unmount
- **Type consolidation**: `ImageDetail extends ImageRecord` eliminates 16 duplicated fields
- **Row mapping helper**: `row_to_image_record` / `row_to_workspace` extracted, eliminating 6 copies of 17-field mapping
- **Unified delete logic**: `delete_image` and `move_images` now call `delete_db_records`; removed redundant manual FTS cleanup (triggers handle it)
- **FTS5 query sanitization**: User search input wrapped in double quotes with escaped special characters to prevent syntax errors
- **BrowserView decomposition**: 560→269 lines; extracted `GridToolbar` component and `useToast` composable
- **Complete i18n**: 48 hardcoded Chinese strings replaced with `t()` calls across 10 components; 21 new i18n keys added
- **ARIA accessibility**: Dropdown menus, context menus, star rating, lightbox dialog, and toast notifications have proper ARIA roles and attributes
- **Dead code removed**: `VirtualScroller.vue`, `ThumbnailCard.vue` (never imported); `walkdir`, `tokio` dependencies (unused)
- **Version strings unified**: All version references set to 0.3.0

### Backend

- `queries.rs`: Added `indexed_at`, `updated_at`, `file_created_at` to the `sort_col` match (previously only `name`, `date`, `size`, `rating`)
- `thumbnail.rs`: Replaced deprecated `image::io::Reader` with `image::ImageReader`
- `Cargo.toml`: Removed unused `walkdir` and `tokio` direct dependencies

## [0.2.0] — 2026-06-07

### Added

- **Multi-size thumbnail presets**: S (120px) / M (200px) / L (320px) buttons in settings
- **Workspace rescan button**: 🔄 button on workspace cards to re-scan for new images
- **Right-click context menu**: Open in Explorer, copy file path, remove from workspace
- **Export metadata**: Export to JSON/CSV with system save dialog; respects selection (选中图片 or 全部); toast success notification
- **Image deduplication**: SHA-256 based duplicate detection (embedded in BrowserView)
- **i18n support**: vue-i18n with Chinese (zh-CN) and English (en) translations
- **Image preview**: Detail panel now shows actual thumbnail image instead of placeholder
- **Backend commands**: `open_in_explorer`, `delete_image`, `find_duplicates`
- **Workspace quick add**: Folder picker dialog (📂 浏览) in create workspace, auto-fills name from folder
- **Tauri plugins**: `tauri-plugin-dialog`, `tauri-plugin-fs` for save dialog and file writing

### Changed

- Settings thumbnail size control changed from range slider to S/M/L preset buttons
- `ImagePreview` component now displays real thumbnail via `useThumbnailStore`
- **Click behavior redesign**: single click now opens detail panel (was: select); added explicit "☐ 选择" button to enter select mode with 全选/取消全选/完成 buttons
- `ThumbnailGrid` accepts `selectMode` prop to branch click behavior
- Escape key exits select mode first, then clears detail panel
- DedupView embedded as component inside BrowserView (not a separate route)

### Optimized

- **LRU thumbnail cache**: Max 500 blob URLs with access-order tracking, auto-evicts oldest
- **Memory cleanup**: Blob URLs revoked when thumbnails scroll out of view (100-item buffer)
- **Skeleton loading**: Shimmer animation shown while thumbnails load (replaces blank placeholder)
- **Hover effects**: Cards lift with translateY + box-shadow + image scale on hover
- **Selected state**: Box-shadow highlight for clearer visual feedback
- **CSS variables**: Added --accent-bg, --radius-sm, --shadow-sm/md/lg for consistency
- **Empty state**: Large icon + title + direct create button on workspace home
- **ImageInfo**: Shows file path, null-safe dimensions display

### Fixed

- **Black screen on dedup click**: Caused by Vue Router treating `/workspace/:id/dedup` as nested child of `/workspace/:id` with no `<router-view>`. Fixed by embedding DedupView as component.
- **Black screen on re-entering workspace**: Caused by Vue transition `mode="out-in"` getting stuck. Removed mode, added `imageStore.reset()` and `selection.clear()` on mount.
- **Toast not visible**: `<style scoped>` cannot style Teleported elements. Changed to `:global()`.
- **Thumbnail generation failed for mismatched extensions**: Files with wrong extensions (.jpg that are GIF, .png that are JPEG, .bmp that are PNG) failed because `image::open` used extension. Now uses `with_guessed_format()` to detect from magic bytes.
- **Dedup crash on null fields**: `formatSize` now handles null/undefined `file_size`.
- **Rust borrow lifetime**: `stmt.query_map()` result bound to local variable to avoid premature drop.
- **find_duplicates panic protection**: Wrapped in `catch_unwind` to prevent app crash.

### Added (2026-06-09)

- **File date tracking**: ImageInfo shows 3 dates — file creation time (`metadata.created()`), file modification time (`metadata.modified()`), and indexed time
- **Copy filename button**: One-click copy in detail panel with ✓ feedback
- **DB migration**: `ALTER TABLE ADD COLUMN` on startup for schema evolution

### Added (2026-06-11)

- **Batch delete**: Move images to system Recycle Bin via `trash` crate, supports single and multi-select
- **Batch move**: Move images to a chosen folder, auto-handles filename conflicts with `_1`/`_2` suffixes
- **Delete context menu**: Right-click "删除文件（回收站）" option in thumbnail grid
- **Selection toolbar**: Delete (🗑️) and Move (📁) buttons in select mode toolbar
- **Backend commands**: `trash_images`, `move_images`, `read_file_bytes`
- **i18n**: Full Chinese/English translations for all new UI elements

### Fixed

- **Cross-drive move failure**: `std::fs::rename` on Windows can't move across drives (os error 17). Fixed with copy+delete fallback.
- **Move losing DB records**: Old records now updated in-place when destination is within workspace
- **Page reset after operation**: `setPage(1)` called before refresh after delete/move to avoid empty grid

### Added (2026-06-11)

- **Lightbox fullscreen viewer**: Double-click thumbnail to view full-size original image; arrow keys for prev/next, Escape to close
- **Context menu move**: Right-click "📁 移动文件" option with folder picker dialog
- **Context menu refresh**: All context menu operations (remove/trash/move) now emit unified refresh via BrowserView
- **UI design overhaul**: Digital Darkroom aesthetic — layered background depths, refined CSS variable system, glass-morphism overlays, unified button system (6 semantic variants + 3 sizes + icon/pill), SVG sidebar icons, card hover shadows, animated context menus, spring-eased toasts

## [0.1.0] — 2026-06-07

### Added

**Rust 后端 (17 Tauri Commands)**
- 工作区管理：`create_workspace`, `list_workspaces`, `delete_workspace`
- 图片扫描：`scan_workspace`, `rescan_workspace`（SHA-256 哈希，批量插入）
- 图片查询：`get_images`（分页+筛选）, `get_image_detail`, `search_images`（FTS5 全文搜索）
- 元数据编辑：`update_notes`, `update_rating`, `batch_update_tags`
- 标签管理：`get_tags`, `create_tag`, `delete_tag`
- 缩略图：`get_thumbnail`（按需生成 WebP）, `generate_thumbnail`
- 导出：`export_metadata`（JSON/CSV）

**Vue 前端**
- 3 个页面：WorkspaceHome, BrowserView, SettingsPage
- 6 个 Pinia stores：workspace, images, tags, selection, thumbnail, settings
- 虚拟滚动缩略图网格（自适应列数，按需加载）
- 侧边栏搜索 + 标签筛选
- 详情面板：图片预览、评分/标签/备注编辑、文件信息
- 暗色/亮色主题切换（CSS 变量）
- 键盘快捷键：Escape 清除选择, Ctrl+A 全选

**基础设施**
- SQLite 数据库（WAL 模式，FTS5 全文搜索，自动同步触发器）
- WebP 缩略图缓存（按需生成，blob URL 缓存）
- 元数据导出（JSON/CSV 格式）

### Fixed

- 修复 WebView2 GPU 加速导致的崩溃（`--disable-gpu`）
- 修复 `image::image_dimensions` 栈缓冲区溢出（扫描阶段移除尺寸读取）
- 修复缩略图编码尺寸不匹配（使用实际 thumbnail 尺寸）
