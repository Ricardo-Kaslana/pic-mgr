# Frontend Context (Vue 3)

## Overview

Vue 3 + TypeScript 前端，通过 Pinia stores 调用 Tauri commands，管理 UI 状态。

## Architecture

```
main.ts              — 应用入口，注册 Pinia + Vue Router
App.vue              — AppLayout 包裹 router-view
router/index.ts      — 3 个路由: /, /workspace/:id, /settings
i18n/                — vue-i18n 国际化 (zh-CN.ts, en.ts, index.ts)
types/index.ts       — TypeScript 接口定义 (ImageRecord / ImageSummary / ImagePage / ImageSummaryPage)
style.css            — 全局样式 + CSS 变量 + 统一按钮系统 (暗色/亮色主题)

stores/
  workspace.ts       — 工作区 CRUD、扫描；单飞 + 缓存的 fetchAll
  images.ts          — 图片列表（ImageSummary[]）、筛选、搜索、分页（默认 pageSize=100）
  tags.ts            — 标签 CRUD、批量操作；fetchTags 单飞 + per-workspace 缓存
  selection.ts       — 多选状态 (支持 Shift 范围选)
  thumbnail.ts       — 缩略图 blob URL 缓存 (shallowRef<Map>, LRU 500)
  settings.ts        — 主题、缩略图尺寸、语言 (localStorage)

views/
  WorkspaceHome.vue  — 工作区列表页
  BrowserView.vue    — 主浏览页 (Sidebar + Grid + Detail + 内嵌 DedupView；排序/定位/面板折叠)
  DedupView.vue      — 重复图片检测 (作为 BrowserView 子组件)
  SettingsPage.vue   — 设置页

components/
  layout/AppLayout.vue      — 侧边栏 + 主内容区布局
  workspace/
    WorkspaceCard.vue        — 工作区卡片
    CreateWorkspaceDialog.vue — 新建工作区弹窗
  browser/
    ThumbnailGrid.vue        — 虚拟滚动缩略图网格 (shallowRef thumbUrls, smart scroll)
    SidebarPanel.vue         — 搜索 + 标签筛选侧栏
    SearchBar.vue            — 防抖搜索输入
    TagFilter.vue            — 标签筛选器
    DetailPanel.vue          — 图片详情面板
  detail/
    ImagePreview.vue         — 图片预览区
    MetadataEditor.vue       — 评分/标签/备注编辑
    ImageInfo.vue            — 只读文件信息（3 个日期 + 复制文件名）—— 接收 ImageRecord
  common/
    TagBadge.vue             — 标签胶囊组件
    LightboxViewer.vue       — 全屏大图查看（双击触发，← → 翻页，Escape 关闭）—— 接收 ImageSummary[]
```

## Key patterns

- **Tauri invoke** — 所有后端调用通过 `invoke<ReturnType>('command_name', { params })`
- **Grid / Detail 分层数据** — grid 用 `ImageSummary`（轻量，~80 B/行）通过 `get_images_light` 拉取；detail panel 用 `ImageRecord`（完整）通过 `get_image_detail` 按需拉取
- **响应式缓存** — `thumbStore.cache` 是 blob URL 唯一持有者（`shallowRef<Map>` + LRU 500），grid 内的 `thumbUrls` 是组件级渲染投影（也是 `shallowRef<Map>`），通过 `triggerRef` 触发整批更新
- **内存管理** — 滚动离开可视区后调用 `thumbStore.revoke` 让 LRU 决定是否 revoke；`evictAllThumbUrls` 只 revoke 不在 store cache 里的孤儿 URL
- **缩略图 batch 调度** — 视口内前 60 个 ID 进 priority batch（单 IPC），其余进 4 路并发 fallback；视口下方 6 行预取区防止滚动时出现骨架屏
- **虚拟滚动** — 自适应网格布局，按行计算可见区域；`onScroll` 在 65% 处预加载下一页
- **悬浮滚动按钮** - 缩略图网格右下角浮动回到顶部 / 回到底部圆形按钮；位置阈值 300px (常量 SCROLL_FAB_THRESHOLD)；visible 状态由单个 ref({ canScroll, scrollTop, distanceFromBottom }) 反应式袋子驱动，在 onScroll / ResizeObserver / watch(totalHeight) 三个时机由 updateScrollInfo() 助手更新；位于 .thumb-grid-outer 内、.thumb-grid 外，position: absolute 相对于不滚动的 wrapper，确保按钮始终固定在视口
- **分页追加** — `fetchImages(id, append=true)` 翻页时追加而非替换，默认 `pageSize=100`
- **主题** — CSS 变量 + `[data-theme]` 属性切换
- **i18n** — vue-i18n，语言切换同步到 localStorage
- **右键菜单** — Teleport 到 body 的 context menu，支持打开/复制/移动/移除/删除
- **导出** — JSON/CSV 格式导出，通过 Blob + URL.createObjectURL 下载
- **交互模式** — 单击查看详情；双击查看大图（全屏原图）；"选择"按钮进入选择模式（点击选中，全选/取消全选/删除/移动/导出），Escape 退出
- **工作区删除错误处理** — `WorkspaceHome.vue` 用 `try/catch` 包裹 `store.remove()`，错误通过红色 dismissible banner 显示（i18n key: `workspace.deleteFailed`）
- **Stores 缓存** — `workspaceStore.fetchAll` 单飞 + 缓存；`tagStore.fetchTags` 单飞 + per-workspace 缓存；避免 BrowserView/SidebarPanel 重复 IPC
