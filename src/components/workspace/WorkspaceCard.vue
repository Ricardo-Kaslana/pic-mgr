<script setup lang="ts">
import type { Workspace } from '../../types'

const props = defineProps<{ workspace: Workspace }>()
const emit = defineEmits<{
  select: [id: number]
  delete: [id: number]
  rescan: [id: number]
}>()
</script>

<template>
  <div class="card" @click="emit('select', workspace.id)">
    <div class="card-icon">📁</div>
    <div class="card-body">
      <h3>{{ workspace.name }}</h3>
      <p class="path">{{ workspace.path }}</p>
      <p class="date">更新于 {{ workspace.updated_at }}</p>
    </div>
    <button
      class="action-btn rescan-btn"
      @click.stop="emit('rescan', workspace.id)"
      title="重新扫描"
    >🔄</button>
    <button
      class="action-btn delete-btn"
      @click.stop="emit('delete', workspace.id)"
      title="删除工作区（不删文件）"
    >×</button>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all var(--transition);
  position: relative;
}
.card:hover { border-color: var(--accent); transform: translateY(-1px); }
.card-icon { font-size: 32px; }
.card-body { flex: 1; }
.card-body h3 { font-size: 16px; font-weight: 600; margin-bottom: 4px; }
.path { font-size: 12px; color: var(--text-muted); margin-bottom: 2px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 400px; }
.date { font-size: 11px; color: var(--text-muted); }
.action-btn {
  background: none;
  color: var(--text-muted);
  font-size: 16px;
  padding: 4px 8px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}
.card:hover .action-btn { opacity: 1; }
.rescan-btn:hover { background: var(--accent-bg); }
.delete-btn { font-size: 20px; }
.delete-btn:hover { background: var(--danger); color: white; }
</style>
