# Changelog

## [0.2.0] — 2026-06-07

### Added

- **Multi-size thumbnail presets**: S (120px) / M (200px) / L (320px) buttons in settings
- **Workspace rescan button**: 🔄 button on workspace cards to re-scan for new images
- **Right-click context menu**: Open in Explorer, copy file path, remove from workspace
- **Export metadata**: Export workspace metadata to JSON or CSV with browser download
- **Image deduplication**: SHA-256 based duplicate detection page (`/workspace/:id/dedup`)
- **i18n support**: vue-i18n with Chinese (zh-CN) and English (en) translations
- **Image preview**: Detail panel now shows actual thumbnail image instead of placeholder
- **Backend commands**: `open_in_explorer`, `delete_image`, `find_duplicates`

### Changed

- Settings thumbnail size control changed from range slider to S/M/L preset buttons
- `ImagePreview` component now displays real thumbnail via `useThumbnailStore`

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
- 修复分页时图片列表被替换而非追加
- 修复虚拟滚动缩略图不可见（使用 reactive Map 触发 Vue 重渲染）
- 修复扫描器使用流式 SHA-256 避免大文件内存问题
- 修复缩略图生成 panic 未捕获（添加 `catch_unwind`）

### Changed

- 扫描器从 `WalkDir` 改为 `std::fs::read_dir` 递归遍历
- 扫描器不再依赖 `image` crate，仅在缩略图生成时使用
- `scan_workspace` 从 async 改为 sync，数据库锁按 50 条一批释放
