# Findings: PicMgr 优化

## 性能分析

### 当前瓶颈
1. **缩略图加载** — `watch(visibleItems)` 每次滚动触发，无去重，可能重复加载
2. **blob URL 泄漏** — `thumbUrls` Map 只增不减，滚动越快内存越大
3. **DB 查询** — `get_images` 每次全量查询，无缓存
4. **缩略图生成** — `catch_unwind` 包裹，串行执行

### 内存热点
- `thumbUrls = reactive<Map<number, string>>` — 无限增长
- `imageStore.images` — 全部在内存中
- `selection.selectedIds` — Set，影响不大

### UI 问题
- 无加载骨架屏，白屏等待
- 空状态只有文字，无视觉引导
- 按钮样式不完全统一
- 详情面板无完整路径显示
