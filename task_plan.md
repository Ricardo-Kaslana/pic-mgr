# Task Plan: PicMgr 性能/细节/美观/内存优化

## Goal
对 PicMgr 进行全面优化，涵盖性能、细节体验、UI 美观、内存管理四个维度。

## Phases

### Phase 1: 性能优化 `complete`
- [x] 缩略图懒加载优化：只在可见区域加载，离开后释放 blob URL
- [x] 图片列表虚拟滚动性能：减少 reactive 开销
- [x] DB 查询优化：确保索引存在，避免全表扫描
- [x] 缩略图生成并发：批量生成时使用线程池而非串行

### Phase 2: 内存优化 `complete`
- [x] blob URL 生命周期管理：滚动离开可视区后 revoke
- [x] 缩略图缓存上限：LRU 淘汰策略 (MAX_CACHE_SIZE=500)
- [x] Vue reactive 对象瘦身：避免大数组深度响应
- [x] Rust 端内存：检查缩略图生成时的临时 Vec 是否及时释放

### Phase 3: UI 美观 `complete`
- [x] 统一间距/圆角/阴影系统 (CSS 变量: --radius-sm, --shadow-sm/md/lg)
- [x] 按钮 hover/active 状态统一
- [x] 加载状态骨架屏 (skeleton shimmer animation)
- [x] 空状态插画/图标优化
- [x] 滚动条美化 (已部分完成)
- [x] 选中态高亮更明显 (box-shadow + border)

### Phase 4: 细节体验 `complete`
- [x] 图片详情面板：显示完整文件路径
- [ ] 键盘导航：上下左右切换选中图片
- [ ] 拖拽排序/拖拽添加标签
- [ ] 搜索历史记录
- [ ] 工作区统计信息（图片总数、总大小）
- [x] 缩略图加载失败时显示 fallback 图标

## Status
- Phase 1: pending
- Phase 2: pending
- Phase 3: pending
- Phase 4: pending

## Errors Encountered
| Error | Attempt | Resolution |
|-------|---------|------------|
