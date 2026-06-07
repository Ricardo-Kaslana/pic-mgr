# Frontend Context (Vue 3)

## Overview

Vue 3 + TypeScript 前端，通过 Pinia stores 调用 Tauri commands，管理 UI 状态。

## Architecture

```
main.ts              — 应用入口，注册 Pinia + Vue Router
App.vue              — AppLayout 包裹 router-view
router/index.ts      — 3 个路由: /, /workspace/:id, /settings
types/index.ts       — TypeScript 接口定义
style.css            — 全局样式 + CSS 变量 (暗色/亮色主题)

stores/
  workspace.ts       — 工作区 CRUD、扫描
  images.ts          — 图片列表、筛选、搜索、分页
  tags.ts            — 标签 CRUD、批量操作
  selection.ts       — 多选状态 (支持 Shift 范围选)
  thumbnail.ts       — 缩略图 blob URL 缓存
  settings.ts        — 主题、缩略图尺寸、语言 (localStorage)

views/
  WorkspaceHome.vue  — 工作区列表页
  BrowserView.vue    — 主浏览页 (Sidebar + Grid + Detail)
  SettingsPage.vue   — 设置页

components/
  layout/AppLayout.vue      — 侧边栏 + 主内容区布局
  workspace/
    WorkspaceCard.vue        — 工作区卡片
    CreateWorkspaceDialog.vue — 新建工作区弹窗
  browser/
    ThumbnailGrid.vue        — 虚拟滚动缩略图网格
    SidebarPanel.vue         — 搜索 + 标签筛选侧栏
    SearchBar.vue            — 防抖搜索输入
    TagFilter.vue            — 标签筛选器
    DetailPanel.vue          — 图片详情面板
  detail/
    ImagePreview.vue         — 图片预览区
    MetadataEditor.vue       — 评分/标签/备注编辑
    ImageInfo.vue            — 只读文件信息
  common/
    TagBadge.vue             — 标签胶囊组件
```

## Key patterns

- **Tauri invoke** — 所有后端调用通过 `invoke<ReturnType>('command_name', { params })`
- **响应式缓存** — thumbnail store 用 `ref<Map>` 缓存 blob URL，grid 用 `reactive<Map>` 触发重渲染
- **虚拟滚动** — 自适应网格布局，按行计算可见区域，动态加载缩略图
- **分页追加** — `fetchImages(id, append=true)` 翻页时追加而非替换
- **主题** — CSS 变量 + `[data-theme]` 属性切换
