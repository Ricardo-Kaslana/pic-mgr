# PicMgr — 本地图片管理器

Tauri 2 + Vue 3 + SQLite 本地图片管理器。所有元数据只写数据库，永不修改原始图片文件。

## 项目结构

```
pic-mgr/
├── src/                          — Vue 前端
│   ├── stores/                   — 6 个 Pinia stores
│   ├── views/                    — 5 个页面
│   ├── components/               — UI 组件
│   ├── i18n/                     — vue-i18n 国际化 (zh-CN, en)
│   ├── types/index.ts            — TypeScript 接口
│   └── router/index.ts           — Vue Router
├── src-tauri/                    — Rust 后端
│   └── src/
│       ├── commands/             — 20 个 Tauri commands
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
- **虚拟滚动** 自适应网格布局，`reactive<Map>` 触发缩略图重渲染
- **LRU 缩略图缓存** 最多 500 个 blob URL，超出淘汰最久未访问的；滚动离开可视区自动 revoke
- **缩略图格式检测** 使用 `with_guessed_format()` 从文件头魔数判断，不依赖扩展名
- **分页** 追加模式（`append=true`），不替换已有数据
- **DedupView** 内嵌为 BrowserView 子组件，非独立路由（避免 Vue Router 嵌套黑屏）

## 已知限制

- 图片尺寸在扫描时为 null，生成缩略图后才填充
- Windows 上 WebView2 需禁用 GPU 加速（`--disable-gpu`）
- RAW 格式（CR2/NEF/ARW）缩略图可能失败（已 catch）
