# PicMgr — 本地图片管理器

Tauri 2 + Vue 3 + SQLite 本地图片管理器。所有元数据只写数据库，永不修改原始图片文件。

## 项目结构

```
pic-mgr/
├── src/                          — Vue 前端
│   ├── stores/                   — 6 个 Pinia stores
│   ├── views/                    — 5 个页面
│   ├── components/               — UI 组件（browser/ 含 GridToolbar, ThumbnailGrid 等）
│   ├── composables/              — Vue composables（useToast 等）
│   ├── i18n/                     — vue-i18n 国际化 (zh-CN, en)
│   ├── types/index.ts            — TypeScript 接口
│   └── router/index.ts           — Vue Router
├── src-tauri/                    — Rust 后端
│   └── src/
│       ├── commands/             — 23 个 Tauri commands
│       ├── db/                   — SQLite schema + queries
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
| 数据库 | SQLite (rusqlite, WAL, FTS5) |
| 图片处理 | image-rs (WebP 缩略图) |
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
- **扫描器** 不使用 image crate 读取尺寸（避免栈溢出），只做文件系统元数据 + SHA-256
- **缩略图** 按需生成，包裹在 `catch_unwind` 中
- **虚拟滚动** 自适应网格布局，`shallowRef<Map>` + `reactive<Map>` 双层缓存触发缩略图重渲染
- **LRU 缩略图缓存** 最多 500 个 blob URL，`shallowRef` 原地 mutation + `triggerRef` 避免 O(n) 复制；滚动离开可视区自动 revoke
- **DB 索引** `idx_images_hash`（查重）和 `idx_image_tags_tag_id`（标签 JOIN）
- **DB 事务** 扫描插入和批量标签操作包裹 `BEGIN`/`COMMIT`，避免逐条 fsync
- **FTS5 安全** 用户搜索输入经 `sanitize_fts_query` 双引号包裹 + 转义，防止 MATCH 语法注入
- **行映射辅助** `row_to_image_record` / `row_to_workspace` 统一在 `db/queries.rs`，消除 6 处重复
- **BrowserView 拆分** 主文件只做布局编排；GridToolbar 组件处理排序/选择/批量/导出；useToast composable 管理通知
- **i18n 完整** 所有 UI 字符走 `t()` / `$t()`，无硬编码中文
- **ARIA 无障碍** 下拉菜单 `aria-haspopup`/`aria-expanded`、右键菜单 `role="menu"`、评分 `role="radiogroup"`、弹窗 `role="dialog"`、toast `role="alert"`
- **缩略图格式检测** 使用 `with_guessed_format()` 从文件头魔数判断，不依赖扩展名
- **分页** 追加模式（`append=true`），不替换已有数据
- **DedupView** 内嵌为 BrowserView 子组件，非独立路由（避免 Vue Router 嵌套黑屏）
- **文件日期** `created_at` 存 filesystem mtime；`file_created_at` 存 birth time；`indexed_at` 存入库时间
- **DB 迁移** 启动时 `ALTER TABLE ADD COLUMN`，`.ok()` 忽略已存在错误
- **删除图片** 使用 `trash` crate 移入系统回收站（安全），同时清理 DB 记录 + 缩略图缓存
- **移动图片** `fs::rename` 移动文件，跨盘 fallback 到 copy+delete；文件名冲突自动加后缀 `_1`/`_2`，更新 DB 记录
- **从工作区移除** 仅删除 DB 记录，不碰磁盘文件（原有行为不变）
- **大图查看** 双击缩略图 → LightboxViewer 全屏，`read_file_bytes` 命令读原文件 → blob URL 显示；← → 键翻页，Escape 关闭
- **按钮系统** 统一 btn-primary/secondary/ghost/danger + btn-sm/md/lg + btn-icon/pill 体系，全局一致 hover/active/disabled 三态反馈
- **排序控制** 网格头部下拉菜单，支持文件名/收录时间/修改时间/创建时间；点击切换升降序；`queries.rs` 的 `sort_col` match 动态映射
- **快速定位** 详情面板打开时自动滚动网格到选中图片（nextTick + 80ms 等待布局回流），accent 发光边框高亮；详情面板头部 ⌖ 按钮手动触发
- **面板折叠** 侧栏和详情面板可完全收起（width→0），28px 圆形切换按钮浮在面板边缘；hover 浮现 + accent 发光；SVG chevron 图标；选中图片时自动展开详情面板

## 已知限制

- 图片尺寸在扫描时为 null，生成缩略图后才填充
- Windows 上 WebView2 需禁用 GPU 加速（`--disable-gpu`）
- RAW 格式（CR2/NEF/ARW）缩略图可能失败（已 catch）

## Behavioral guidelines

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

### 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

### 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

### 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.
