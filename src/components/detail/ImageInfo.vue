<script setup lang="ts">
import type { ImageRecord } from '../../types'

defineProps<{ image: ImageRecord }>()

function formatSize(bytes: number | null): string {
  if (!bytes) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}
</script>

<template>
  <div class="image-info">
    <div class="info-row">
      <span class="label">文件名</span>
      <span>{{ image.file_name }}</span>
    </div>
    <div class="info-row">
      <span class="label">格式</span>
      <span>{{ image.format?.toUpperCase() ?? '-' }}</span>
    </div>
    <div class="info-row">
      <span class="label">尺寸</span>
      <span>{{ image.width }} × {{ image.height }}</span>
    </div>
    <div class="info-row">
      <span class="label">文件大小</span>
      <span>{{ formatSize(image.file_size) }}</span>
    </div>
    <div class="info-row">
      <span class="label">索引时间</span>
      <span>{{ image.indexed_at }}</span>
    </div>
  </div>
</template>

<style scoped>
.image-info { padding: 8px 0; border-top: 1px solid var(--border); }
.info-row {
  display: flex; justify-content: space-between;
  padding: 6px 0; font-size: 12px;
}
.label { color: var(--text-muted); }
</style>
