# CLAUDE.md

Project-level guidance for AI agents.

## 项目概览

轻量、现代、美观的本地图片管理器。将文件夹设为工作区，为所有图片生成索引，支持标签、评分、备注、全文搜索。

**所有元数据只写 SQLite，永不修改原始图片文件。**

## 项目结构

```
pic-mgr/
├── src/                          — Vue 前端
│   ├── stores/                   — 6 个 Pinia stores + 3 个 composables
│   ├── views/                    — 4 个页面
│   ├── components/               — UI 组件
│   ├── i18n/                     — vue-i18n (zh-CN, en)
│   └── types/index.ts            — ImageRecord / ImageSummary / ImagePage / ImageSummaryPage
├── src-tauri/                    — Rust 后端
│   └── src/
│       ├── commands/             — 24 个 Tauri commands（含 get_images_light）
│       ├── db/                   — SQLite schema + queries（含 PRAGMAs + 复合索引）
│       └── scanner/              — 文件扫描 + 缩略图
├── docs/                         — 设计文档
│   ├── adr/                      — 架构决策记录
│   ├── agents/                   — Agent 技能配置
│   └── superpowers/              — 设计规格 + 实现计划
├── CHANGELOG.md
├── CONTEXT-MAP.md
└── README.md
```

## 技术栈

| 层 | 选型 |
|---|------|
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 + TypeScript + Pinia + Vue Router 4 |
| 国际化 | vue-i18n (zh-CN, en) |
| 构建 | Vite |
| 数据库 | SQLite (rusqlite, WAL, FTS5, 复合索引) |
| 图片处理 | image-rs (JPEG 缩略图) |
| 哈希 | sha2 (SHA-256) |
| 回收站 | trash (跨平台) |

## 开发命令

```bash
npm run tauri dev          # 开发模式
npm run tauri build        # 构建
cd src-tauri && cargo check # Rust 编译检查
```

## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Uses the default canonical triage role strings (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Multi-context — `CONTEXT-MAP.md` at root points to per-context `CONTEXT.md` files. See `docs/agents/domain.md`.

## 关键设计决策

- **AppState** 通过 `app.manage()` 注入，commands 通过 `State<AppState>` 获取
- **Database** 使用 `Mutex<Connection>` 保护并发访问
- **SQLite PRAGMAs** 启动时设置：`synchronous=NORMAL`、`cache_size=-20000`（20 MB）、`temp_store=MEMORY`、`mmap_size=256 MB`，加 `ANALYZE` 让查询规划器选对索引
- **复合索引** `(workspace_id, sort_column)` 覆盖按工作区分页 + 排序的所有热路径，外加 `(workspace_id, file_hash)` 给去重、`(workspace_id, format)` 给筛选
- **N+1 查询消除** — `get_tags` 单条 GROUP BY；`find_duplicate_groups` 用 CTE + JOIN；批量缩略图用 `WHERE id IN (...)` 单次查询
- **轻量 grid 投影** — `ImageSummary`（~80 B/行）只含 grid 渲染所需的字段，`get_images_light` 命令回传它；`ImageRecord`（~250 B/行）只在打开详情面板时通过 `get_image_detail` 拉取
- **扫描器** 不使用 image crate 读取尺寸（避免栈溢出），只做文件系统元数据 + SHA-256
- **缩略图** 按需生成，包裹在 `catch_unwind` 中
- **缩略图 batch** 单次 IPC 处理最多 60 个 ID（`BATCH_CHUNK_SIZE`），后端用 `WHERE id IN (...)` 一次查完，剩余的走 4 路并发 fallback；下方 6 行预取区防止滚动时出现骨架屏
- **虚拟滚动** 自适应网格布局，scroll 在 65% 触发下一页（避免"到底才加载"造成的卡顿）
- **LRU 缩略图缓存** `thumbStore.cache` 最多 500 个 blob URL，`shallowRef<Map>` + `triggerRef` 触发渲染更新；滚动远离 viewport 时调用 `thumbStore.revoke` 由 LRU 负责生命周期
- **thumbUrls 双层结构** — `thumbStore.cache` 是 blob URL 唯一持有者，ThumnbnailGrid 的 `thumbUrls` 是组件本地的渲染投影；`flushBatch` 必须从 cache 拉所有 ID 的 URL（不只是 `loaded`），`evictAllThumbUrls` 不能 revoke 仍在 cache 里的 URL
- **分页** 追加模式（`append=true`），不替换已有数据；默认 `pageSize = 100`
- **DB 索引** 除复合索引外保留单列 `idx_images_workspace_id`、`idx_image_tags_tag_id` 处理纯前缀筛选
- **DB 事务** 扫描插入和批量标签操作包裹 `BEGIN`/`COMMIT`，避免逐条 fsync
- **FTS5 安全** 用户搜索输入经 `sanitize_fts_query` 双引号包裹 + 转义，防止 MATCH 语法注入
- **行映射辅助** `row_to_image_record` / `row_to_image_summary` / `row_to_workspace` 统一在 `db/queries.rs`
- **Stores 缓存** — `workspaceStore.fetchAll` 单飞 + 缓存；`tagStore.fetchTags` 单飞 + 按 workspace 缓存；避免重复 IPC
- **BrowserView 拆分** 主文件只做布局编排；GridToolbar 组件处理排序/选择/批量/导出；useToast composable 管理通知
- **i18n 完整** 所有 UI 字符走 `t()` / `$t()`，无硬编码中文
- **ARIA 无障碍** 下拉菜单 `aria-haspopup`/`aria-expanded`、右键菜单 `role="menu"`、评分 `role="radiogroup"`、弹窗 `role="dialog"`、toast `role="alert"`
- **缩略图格式检测** 使用 `with_guessed_format()` 从文件头魔数判断，不依赖扩展名
- **DedupView** 内嵌为 BrowserView 子组件，非独立路由（避免 Vue Router 嵌套黑屏）
- **文件日期** `created_at` 存 filesystem mtime；`file_created_at` 存 birth time；`indexed_at` 存入库时间
- **DB 迁移** 启动时 `ALTER TABLE ADD COLUMN`，`.ok()` 忽略已存在错误
- **删除图片** 使用 `trash` crate 移入系统回收站（安全），同时清理 DB 记录 + 缩略图缓存
- **重新扫描** `workspaceStore.scan()` 调用 `rescan_workspace`（先 `DELETE FROM images WHERE workspace_id = ?`，再扫描），确保已删除文件的孤立 DB 记录被清除；进度事件无缝有效
- **移动图片** `fs::rename` 移动文件，跨盘 fallback 到 copy+delete；文件名冲突自动加后缀 `_1`/`_2`，更新 DB 记录
- **从工作区移除** 仅删除 DB 记录，不碰磁盘文件（原有行为不变）
- **大图查看** 双击缩略图或点击 hover 显形的展开按钮 → LightboxViewer 全屏，`read_file_bytes` 命令读原文件 → blob URL 显示；← → 键翻页，Escape 关闭
- **按钮系统** 统一 btn-primary/secondary/ghost/danger + btn-sm/md/lg + btn-icon/pill 体系，全局一致 hover/active/disabled 三态反馈

- **工作区名称** GridToolbar 在返回按钮和图片计数之间以粗体显示当前工作区名称（workspaceName prop，来自 workspaceStore.current?.name），最大宽度 260px 超出省略号截断；GridToolbar 不与 workspace store 耦合

- **排序控制** 网格头部下拉菜单，支持文件名/收录时间/修改时间/创建时间；点击切换升降序；`queries.rs` 的 `sort_col` match 动态映射；排序列与方向跨会话持久化到 `useSettingsStore.sortBy` / `sortDir` (`pic-mgr-sort-by` / `pic-mgr-sort-dir`)
- **快速定位** 详情面板打开时自动滚动网格到选中图片（nextTick + 80ms 等待布局回流），accent 发光边框高亮；详情面板头部 ⌖ 按钮手动触发
- **面板折叠** 侧栏和详情面板可完全收起（width→0），28px 圆形切换按钮浮在面板边缘；hover 浮现 + accent 发光；SVG chevron 图标；选中图片时自动展开详情面板；工作区内层 SidebarPanel 折叠状态跨会话持久化（通过 `useSettingsStore.browserSidebarCollapsed`，由 `useLocalStorage` 统一写入 `pic-mgr-browser-sidebar-collapsed`）
- **悬浮滚动按钮** 缩略图网格右下角浮动 `回到顶部` / `回到底部` 圆形按钮（v0.3.4+）。
  - .scroll-fab 与 .grid-sizer 同级，是 .thumb-grid 的直接子元素；.thumb-grid 设了 position: relative 才能让绝对定位 FAB 钉在可视区域，被 overflow: auto 边界裁剪。
  - 单个 ref({ canScroll, scrollTop, distanceFromBottom }) 反应式袋子，由 updateScrollInfo() 助手在 onScroll / ResizeObserver / watch(totalHeight) 三个时机更新；不拆成多个 ref，避免多余 reactivity 开销。
  - 阈值 300px 通过模块顶部常量 SCROLL_FAB_THRESHOLD 暴露给模板（不在模板里写魔法数字）

- **useLocalStorage composable** 泛型封装 `ref(getItem)+watch(setItem)` 模式，支持 custom read/write/apply hooks；settings store 的 7 个持久化字段均通过该 composable 实现，消除了重复代码
- **storageKeys 常量** 所有 localStorage 键名集中在 `src/composables/storageKeys.ts`，避免魔法字符串分散在各处
- **workspace.scan() 合并** 移除无进度监听的 `scan()` 变体，保留 always-subscribe 的单一实现；`startScan` 已从 store 导出中删除

## 已知限制

- 图片尺寸在扫描时为 null，生成缩略图后才填充
- Windows 上 WebView2 需禁用 GPU 加速（`--disable-gpu`）
- RAW 格式（CR2/NEF/ARW）缩略图可能失败（已 catch）
