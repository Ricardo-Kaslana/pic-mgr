<div align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="112" alt="PicMgr logo" />
  <h1>PicMgr</h1>
  <p>轻量、现代、完全本地化的桌面图片管理器。</p>

  <p>
    <a href="https://github.com/Ricardo-Kaslana/pic-mgr/actions/workflows/ci.yml"><img src="https://github.com/Ricardo-Kaslana/pic-mgr/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
    <a href="https://github.com/Ricardo-Kaslana/pic-mgr/releases"><img src="https://img.shields.io/github/v/release/Ricardo-Kaslana/pic-mgr?include_prereleases" alt="Latest release" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
    <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
    <img src="https://img.shields.io/badge/Vue-3-42B883?logo=vue.js&logoColor=white" alt="Vue 3" />
  </p>

  <p>
    <a href="README.md">English</a> |
    <a href="README.zh-CN.md">简体中文</a>
  </p>
</div>

> **原始图片永不修改。** PicMgr 将本地文件夹设为工作区并为图片建立索引，标签、评分和备注全部保存在 SQLite 中。除非你明确操作，否则它不会修改、移动或重写任何原始图片。

## 功能

- **工作区管理** — 将任意文件夹添加为工作区，随时重新扫描或同步磁盘变更。
- **快速索引** — 基于 SHA-256 的增量扫描与重复图片检测，底层使用 SQLite + FTS5。
- **标签系统** — 全局共享的标签库，可跨工作区复用；支持标签筛选、批量挂载/移除、侧栏右键删除。
- **评分与备注** — 每张图片支持 0–5 星评分和自由文本备注。
- **全文搜索** — 通过 SQLite FTS5 搜索文件名、路径与备注。
- **缩略图缓存** — 按需生成三档尺寸的 JPEG 缩略图，磁盘缓存 + 内存 LRU blob URL。
- **安全的文件操作** — 批量删除进入系统回收站；批量移动支持跨盘移动与重名自动加后缀。
- **重复检测** — 按 SHA-256 分组展示重复图片，方便清理。
- **元数据导出** — 将图库导出为 JSON 或 CSV。
- **舒适浏览** — 虚拟滚动、键盘操作的全屏查看、排序持久化、可折叠侧栏与快速定位。
- **中英双语界面** — 完整的 English / 简体中文翻译。
- **Digital Darkroom 主题** — 精心调校的暗色与亮色主题。

## 技术栈

| 层 | 选型 |
| --- | --- |
| 桌面框架 | [Tauri 2](https://v2.tauri.app/) |
| 前端 | Vue 3 + TypeScript + Pinia + Vue Router |
| 存储 | SQLite（`rusqlite` bundled）+ FTS5 |
| 缩略图 | [`image`](https://crates.io/crates/image) — JPEG quality 80 |
| 哈希 | `sha2`（SHA-256） |
| 国际化 | `vue-i18n`（`en`、`zh-CN`） |

## 快速开始

### 环境要求

- [Node.js](https://nodejs.org/) 20+
- Rust stable
- Tauri 2 所需的系统依赖 — 参见官方[环境准备](https://v2.tauri.app/start/prerequisites/)

### 开发

```bash
npm install
npm run tauri dev
```

### 生产构建

```bash
npm run tauri build
```

### 检查

```bash
npm run typecheck              # vue-tsc
cd src-tauri && cargo fmt --check
cd src-tauri && cargo test
```

## 工作原理

- 文件夹被添加为**工作区**；每次扫描都会把文件元数据与 SHA-256 哈希写入 SQLite。
- 所有元数据（标签、评分、备注、缩略图）都与原始文件分离存放。
- Rust 后端按领域拆分为 29 个 Tauri commands（`commands/`、`db/`、`scanner/`）。
- 网格只渲染 `ImageSummary`；详情面板按需加载完整的 `ImageRecord`。
- 缩略图惰性生成，并用 `catch_unwind` 包裹，避免损坏文件导致崩溃。

更多细节见[架构设计](docs/design/architecture.md)与 [ADR](docs/README.md)。

## 项目结构

```text
pic-mgr/
├── src/                    # Vue 3 前端
│   ├── components/         # UI 组件
│   ├── composables/        # 组合式函数
│   ├── i18n/               # en + zh-CN 文案
│   ├── router/             # Vue Router
│   ├── stores/             # Pinia stores
│   ├── types/              # TypeScript 类型
│   └── views/              # 工作区首页、浏览、去重、设置
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands/       # Tauri commands
│       ├── db/             # Schema、迁移与查询
│       └── scanner/        # 文件扫描 + 缩略图缓存
├── docs/                   # 设计记录 + ADR
├── AGENTS.md               # 开发约定与架构约束
└── CHANGELOG.md
```

## 文档

- [架构概览](docs/design/architecture.md)
- [设计记录](docs/design/)
- [架构决策记录](docs/adr/)
- [更新日志](CHANGELOG.md)
- [贡献指南](CONTRIBUTING.md)

## 参与贡献

欢迎提交 Bug、功能建议与 Pull Request。提交 PR 前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，本项目遵循[贡献者公约](CODE_OF_CONDUCT.md)。

如果发现安全问题，请按照 [SECURITY.md](SECURITY.md) 私下报告，不要公开提交 issue。

## 许可证

基于 [MIT License](LICENSE) 开源。

Copyright © 2026 Dawn.Wang.
