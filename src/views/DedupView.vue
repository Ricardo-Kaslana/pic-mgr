<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import type { ImageRecord } from '../types'

interface DuplicateGroup {
  hash: string
  images: ImageRecord[]
}

const route = useRoute()
const router = useRouter()
const workspaceId = ref(Number(route.params.id))
const groups = ref<DuplicateGroup[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    groups.value = await invoke('find_duplicates', { workspaceId: workspaceId.value })
  } catch (e: any) {
    console.error('Failed to find duplicates:', e)
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function formatSize(bytes: number | null | undefined): string {
  if (bytes == null) return '—'
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + ' KB'
  return (bytes / 1048576).toFixed(1) + ' MB'
}

async function removeImage(id: number) {
  if (confirm('从工作区移除此图片？不会删除原始文件。')) {
    await invoke('delete_image', { id })
    groups.value = await invoke('find_duplicates', { workspaceId: workspaceId.value })
  }
}

function goBack() {
  router.push(`/workspace/${workspaceId.value}`)
}
</script>

<template>
  <div class="dedup-view">
    <header class="dedup-header">
      <button class="back-btn" @click="goBack">← 返回</button>
      <h1>重复图片检测</h1>
      <span class="count" v-if="!loading && !error">{{ groups.length }} 组重复</span>
    </header>

    <div v-if="loading" class="loading">扫描中...</div>

    <div v-else-if="error" class="error">
      <p>⚠️ 加载失败</p>
      <p class="error-detail">{{ error }}</p>
    </div>

    <div v-else-if="groups.length === 0" class="empty">
      <p>🎉 没有发现重复图片</p>
    </div>

    <div v-else class="groups">
      <div v-for="group in groups" :key="group.hash" class="dup-group">
        <div class="group-header">
          <span class="hash">SHA-256: {{ group.hash.substring(0, 16) }}...</span>
          <span class="group-count">{{ group.images.length }} 个文件</span>
        </div>
        <div class="group-images">
          <div v-for="img in group.images" :key="img.id" class="dup-item">
            <div class="item-info">
              <p class="file-name">{{ img.file_name }}</p>
              <p class="file-path">{{ img.relative_path }}</p>
              <p class="file-meta">{{ formatSize(img.file_size) }} · {{ img.format || '—' }}</p>
            </div>
            <button class="remove-btn" @click="removeImage(img.id)">移除</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dedup-view {
  padding: 24px;
  height: 100%;
  overflow-y: auto;
}
.dedup-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 24px;
}
.back-btn {
  padding: 6px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  cursor: pointer;
}
.back-btn:hover { border-color: var(--accent); }
h1 { font-size: 20px; font-weight: 600; }
.count { font-size: 13px; color: var(--text-muted); }
.loading { text-align: center; padding: 60px 0; color: var(--text-muted); }
.empty { text-align: center; padding: 60px 0; color: var(--text-muted); font-size: 16px; }
.error { text-align: center; padding: 60px 0; color: var(--danger); }
.error-detail { font-size: 12px; color: var(--text-muted); margin-top: 8px; }
.groups { display: flex; flex-direction: column; gap: 20px; }
.dup-group {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
}
.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border);
}
.hash { font-size: 12px; font-family: monospace; color: var(--text-muted); }
.group-count { font-size: 12px; color: var(--accent); }
.group-images { padding: 8px; }
.dup-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
}
.dup-item:hover { background: var(--bg-tertiary); }
.item-info { flex: 1; }
.file-name { font-size: 14px; font-weight: 500; }
.file-path { font-size: 12px; color: var(--text-muted); margin-top: 2px; }
.file-meta { font-size: 11px; color: var(--text-muted); margin-top: 2px; }
.remove-btn {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: none;
  color: var(--text-secondary);
  cursor: pointer;
}
.remove-btn:hover { background: var(--danger); color: white; border-color: var(--danger); }
</style>
