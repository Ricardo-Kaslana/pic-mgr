# Frontend Context (Vue 3)

## Overview

Vue 3 + TypeScript 前端，通过 Pinia stores 调用 Tauri commands，管理 UI 状态。

## Architecture

```
main.ts              — 应用入口，注册 Pinia + Vue Router
App.vue              — AppLayout 包裹 router-view
router/index.ts      — 3 个路由: /, /workspace/:id, /settings
i18n/                — vue-i18n 国际化 (zh-CN.ts, en.ts, index.ts)
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
  BrowserView.vue    — 主浏览页 (Sidebar + Grid + Detail + 内嵌 DedupView)
  DedupView.vue      — 重复图片检测 (作为 BrowserView 子组件)
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
- **响应式缓存** — thumbnail store 用 LRU 缓存 (MAX 500)，grid 用 `reactive<Map>` 触发重渲染
- **内存管理** — 滚动离开可视区后 revoke blob URL，LRU 淘汰最久未访问的缩略图
- **虚拟滚动** — 自适应网格布局，按行计算可见区域，动态加载缩略图
- **分页追加** — `fetchImages(id, append=true)` 翻页时追加而非替换
- **主题** — CSS 变量 + `[data-theme]` 属性切换
- **i18n** — vue-i18n，语言切换同步到 localStorage
- **右键菜单** — Teleport 到 body 的 context menu，支持打开/复制/移除
- **导出** — JSON/CSV 格式导出，通过 Blob + URL.createObjectURL 下载
- **交互模式** — 单击查看图片详情；"选择"按钮进入选择模式（点击选中，全选/取消全选），Escape 退出
