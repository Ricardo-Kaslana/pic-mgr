<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const props = defineProps<{ workspaceId: number }>()
const emit = defineEmits<{ close: [] }>()

const groups = ref<any[]>([])
const loading = ref(true)
const error = ref('')

onMounted(async () => {
  try {
    const result = await invoke<any[]>('find_duplicates', { workspaceId: props.workspaceId })
    groups.value = result || []
  } catch (e: any) {
    console.error('[DedupView] error:', e)
    error.value = String(e)
  } finally {
    loading.value = false
  }
})

function formatSize(bytes: any): string {
  if (bytes == null) return '—'
  const n = Number(bytes)
  if (isNaN(n)) return '—'
  if (n < 1024) return n + ' B'
  if (n < 1048576) return (n / 1024).toFixed(1) + ' KB'
  return (n / 1048576).toFixed(1) + ' MB'
}

async function removeImage(id: number) {
  if (confirm(t('context.confirmRemove'))) {
    try {
      await invoke('delete_image', { id })
      groups.value = await invoke<any[]>('find_duplicates', { workspaceId: props.workspaceId }) || []
    } catch (e) {
      console.error('Failed to remove:', e)
    }
  }
}
</script>

<template>
  <div class="dedup-view">
    <header class="dedup-header">
      <button class="back-btn" @click="emit('close')">← {{ t('common.back') }}</button>
      <h1>{{ t('dedup.title') }}</h1>
      <span class="count" v-if="!loading && !error">{{ t('dedup.groupCount', { count: groups.length }) }}</span>
    </header>

    <div v-if="loading" class="status-msg">{{ t('dedup.scanning') }}</div>
    <div v-else-if="error" class="status-msg error">⚠️ {{ error }}</div>
    <div v-else-if="groups.length === 0" class="status-msg">{{ t('dedup.noDuplicates') }}</div>

    <div v-else class="groups">
      <div v-for="group in groups" :key="group.hash" class="dup-group">
        <div class="group-header">
          <span class="hash">SHA-256: {{ group.hash ? group.hash.substring(0, 16) : '?' }}...</span>
          <span class="group-count">{{ t('dedup.fileCount', { count: group.images ? group.images.length : 0 }) }}</span>
        </div>
        <div class="group-images" v-if="group.images">
          <div v-for="img in group.images" :key="img.id" class="dup-item">
            <div class="item-info">
              <p class="file-name">{{ img.file_name || '—' }}</p>
              <p class="file-path">{{ img.relative_path || '—' }}</p>
              <p class="file-meta">{{ formatSize(img.file_size) }} · {{ img.format || '—' }}</p>
            </div>
            <button class="remove-btn" @click="removeImage(img.id)">{{ t('dedup.remove') }}</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dedup-view {
  flex: 1;
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
  padding: 6px 14px;
  font-size: 12.5px;
  font-weight: 500;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.back-btn:hover { background: var(--bg-tertiary); color: var(--text-primary); border-color: var(--text-muted); }
.back-btn:active { opacity: 0.8; }
h1 { font-size: 20px; font-weight: 600; }
.count { font-size: 13px; color: var(--text-muted); }
.status-msg { text-align: center; padding: 60px 0; color: var(--text-muted); font-size: 16px; }
.status-msg.error { color: var(--danger); }
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
  padding: 5px 12px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.remove-btn:hover { background: var(--danger-bg); color: var(--danger); border-color: var(--danger); }
.remove-btn:active { opacity: 0.8; }
</style>
