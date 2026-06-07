# ADR-0001: Tauri 2 作为桌面框架

**日期:** 2026-06-07
**状态:** 已接受

## 背景

需要一个桌面应用框架来构建本地图片管理器。

## 决策

选择 Tauri 2.x。

## 理由

- 二进制体积小 (~10MB)，相比 Electron (~150MB)
- Rust 后端天然支持文件系统操作、SQLite、图片处理
- WebView2 (Windows) / WebKit (macOS/Linux) 复用系统浏览器引擎
- 前端可自由选择 Vue/React/Svelte

## 后果

- 需要 Rust 工具链
- Windows 上依赖 WebView2 运行时
- 调试需要同时关注 Rust 和前端
