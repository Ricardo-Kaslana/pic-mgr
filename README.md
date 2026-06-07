# PicMgr — 本地图片管理器

轻量、现代、美观的本地图片管理器。将文件夹设为工作区，为所有图片生成索引，支持标签、评分、备注、全文搜索。

**所有元数据只写 SQLite，永不修改原始图片文件。**

## 技术栈

- **Tauri 2** — 桌面框架 (~10MB)
- **Vue 3** + TypeScript + Pinia — 前端
- **SQLite** + FTS5 — 元数据存储 + 全文搜索
- **image-rs** — WebP 缩略图生成

## 功能

- 🗂️ 工作区管理（创建/删除/选择/重新扫描）
- 📸 图片扫描索引（SHA-256 哈希，增量扫描）
- 🏷️ 标签系统（创建/删除/批量操作）
- ⭐ 评分（0-5 星）
- 📝 备注编辑
- 🔍 全文搜索（FTS5）
- 🖼️ WebP 缩略图缓存（按需生成，S/M/L 三档尺寸）
- 📊 元数据导出（JSON/CSV）
- 🔎 重复图片检测（SHA-256 去重）
- 🖱️ 右键菜单（打开/复制路径/移除）
- 🌐 中英文界面（vue-i18n）
- 🎨 暗色/亮色主题
- ⚡ 虚拟滚动（大量图片高性能）

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 项目结构

```
pic-mgr/
├── src/                    — Vue 前端
│   ├── stores/             — 6 个 Pinia stores
│   ├── views/              — 3 个页面
│   └── components/         — UI 组件
├── src-tauri/              — Rust 后端
│   └── src/
│       ├── commands/       — 17 个 Tauri commands
│       ├── db/             — SQLite schema + queries
│       └── scanner/        — 文件扫描 + 缩略图
├── docs/                   — 设计文档 + ADR
└── CLAUDE.md               — Agent 技能配置
```

## 文档

- [设计规格](docs/superpowers/specs/2026-06-07-pic-mgr-design.md)
- [实现计划](docs/superpowers/plans/2026-06-07-pic-mgr-implementation.md)
- [架构决策](docs/adr/)
- [上下文地图](CONTEXT-MAP.md)
