<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { ImageRecord } from '../../types'

defineProps<{ image: ImageRecord }>()

const { t } = useI18n()

function formatSize(bytes: number | null | undefined): string {
  if (bytes == null) return '—'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / 1048576).toFixed(1)} MB`
}

function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr) return '—'
  return dateStr.replace('T', ' ')
}

const copied = defineModel<boolean>('copied', { default: false })

async function copyFileName(name: string) {
  try {
    await navigator.clipboard.writeText(name)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = name
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}
</script>

<template>
  <div class="image-info">
    <div class="info-row">
      <span class="label">{{ t('detail.fileName') }}</span>
      <span class="value filename-row">
        {{ image.file_name }}
        <button class="copy-btn" @click="copyFileName(image.file_name)" :title="t('detail.copyName')">
          {{ copied ? '✓' : '📋' }}
        </button>
      </span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.path') }}</span>
      <span class="value path" :title="image.relative_path">{{ image.relative_path }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.format') }}</span>
      <span class="value">{{ image.format?.toUpperCase() ?? '—' }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.dimensions') }}</span>
      <span class="value">{{ image.width && image.height ? `${image.width} × ${image.height}` : '—' }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.fileSize') }}</span>
      <span class="value">{{ formatSize(image.file_size) }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.fileCreatedAt') }}</span>
      <span class="value">{{ formatDate(image.file_created_at) }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.fileModifiedAt') }}</span>
      <span class="value">{{ formatDate(image.created_at) }}</span>
    </div>
    <div class="info-row">
      <span class="label">{{ t('detail.indexedAt') }}</span>
      <span class="value">{{ formatDate(image.indexed_at) }}</span>
    </div>
  </div>
</template>

<style scoped>
.image-info {
  padding: 4px 0;
  border-top: 1px solid var(--border-light);
  margin-top: 4px;
}
.info-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 5px 0;
  font-size: 12px;
  gap: 12px;
}
.label {
  color: var(--text-muted);
  flex-shrink: 0;
  font-weight: 450;
}
.value {
  text-align: right;
  word-break: break-all;
  font-weight: 450;
  color: var(--text-secondary);
}
.path {
  font-size: 11px;
  color: var(--text-muted);
  max-height: 3em;
  overflow: hidden;
  text-overflow: ellipsis;
}
.filename-row {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
}
.copy-btn {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 12px;
  padding: 0 2px;
  opacity: 0.4;
  transition: all var(--transition-fast);
  line-height: 1;
}
.copy-btn:hover { opacity: 1; color: var(--accent); }
</style>
