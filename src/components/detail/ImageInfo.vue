<script setup lang="ts">
import type { ImageRecord } from '../../types'

defineProps<{ image: ImageRecord }>()

function formatSize(bytes: number | null | undefined): string {
  if (bytes == null) return '—'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}
</script>

<template>
  <div class="image-info">
    <div class="info-row">
      <span class="label">文件名</span>
      <span class="value">{{ image.file_name }}</span>
    </div>
    <div class="info-row">
      <span class="label">路径</span>
      <span class="value path" :title="image.relative_path">{{ image.relative_path }}</span>
    </div>
    <div class="info-row">
      <span class="label">格式</span>
      <span class="value">{{ image.format?.toUpperCase() ?? '—' }}</span>
    </div>
    <div class="info-row">
      <span class="label">尺寸</span>
      <span class="value">{{ image.width && image.height ? `${image.width} × ${image.height}` : '—' }}</span>
    </div>
    <div class="info-row">
      <span class="label">大小</span>
      <span class="value">{{ formatSize(image.file_size) }}</span>
    </div>
    <div class="info-row">
      <span class="label">收录时间</span>
      <span class="value">{{ image.indexed_at }}</span>
    </div>
  </div>
</template>

<style scoped>
.image-info {
  padding: 8px 0;
  border-top: 1px solid var(--border);
}
.info-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 5px 0;
  font-size: 12px;
  gap: 8px;
}
.label {
  color: var(--text-muted);
  flex-shrink: 0;
  min-width: 56px;
}
.value {
  text-align: right;
  word-break: break-all;
}
.path {
  font-size: 11px;
  color: var(--text-secondary);
  max-height: 3em;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
