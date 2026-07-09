# PicMgr — 本地图片管理器

轻量、现代、美观的本地图片管理器。将文件夹设为工作区，为所有图片生成索引，支持标签、评分、备注、全文搜索。

**所有元数据只写 SQLite，永不修改原始图片文件。**

## 技术栈

- **Tauri 2** — 桌面框架 (~10MB)
- **Vue 3** + TypeScript + Pinia — 前端
- **SQLite** + FTS5 — 元数据存储 + 全文搜索
- **image-rs** — JPEG 缩略图生成

## 功能

- 🗂️ 工作区管理（创建/删除/选择/重新扫描）
- 📸 图片扫描索引（SHA-256 哈希，增量扫描）
- 🏷️ 标签系统（创建/删除/批量操作）
- ⭐ 评分（0-5 星）
- 📝 备注编辑
- 🔍 全文搜索（FTS5）
- 🖼️ JPEG 缩略图缓存（按需生成，S/M/L 三档尺寸）
- 📊 元数据导出（JSON/CSV）
- 🔎 重复图片检测（SHA-256 去重）
- 📅 文件日期追踪（创建时间/修改时间/收录时间）
- 🔄 重新扫描自动清除已删除的图片记录
- 📋 一键复制文件名
- 🗑️ 删除图片（移入回收站，支持批量）
- 📁 移动图片（选择目标文件夹，支持批量）
- 🖱️ 右键菜单（打开/复制路径/移动/移除/删除）
- 🖼️ 全屏查看（双击缩略图或 hover 显形的左上角展开按钮，← → 翻页）
- 🌐 中英文界面（vue-i18n）
- 🎨 暗色/亮色主题（Digital Darkroom 美学）
- 🎛️ 统一按钮系统（6 种语义变体 + 3 档尺寸）
- ⚡ 虚拟滚动（大量图片高性能）
- 🔃 排序控制（按文件名/收录时间/修改时间/创建时间，升降序切换；选择跨会话持久化）
- 当前工作区名称显示在工具栏（返回按钮与图片计数之间，长名称自动省略）
- 📍 快速定位（详情面板打开后自动滚动到并高亮选中图片，手动定位按钮）
- ◀▶ 侧栏可折叠为 64px 图标列（chevron 切换，hover 显示 tooltip，localStorage 持久化，主题切换按钮在收起态默认透明，hover 才显灰）；工作区侧栏折叠状态同样跨会话持久化（useLocalStorage 统一管理所有偏好项）

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
│   ├── views/              — 4 个页面
│   ├── components/         — UI 组件
│   └── i18n/               — vue-i18n 国际化
├── src-tauri/              — Rust 后端
│   └── src/
│       ├── commands/       — 23 个 Tauri commands
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
