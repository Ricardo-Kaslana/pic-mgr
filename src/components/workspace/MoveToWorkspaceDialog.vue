<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../../stores/workspace'

const props = defineProps<{ currentWorkspaceId: number }>()
const emit = defineEmits<{
  select: [workspaceId: number]
  close: []
}>()

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()

const workspaces = computed(() =>
  workspaceStore.workspaces.filter(ws => ws.id !== props.currentWorkspaceId)
)

function handleSelect(wsId: number) {
  emit('select', wsId)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ t('moveToWorkspace.title') }}</h3>
        <button class="close-btn" @click="emit('close')">×</button>
      </div>
      <div class="dialog-body">
        <div
          v-for="ws in workspaces"
          :key="ws.id"
          class="workspace-item"
          @click="handleSelect(ws.id)"
        >
          <div class="ws-name">{{ ws.name }}</div>
          <div class="ws-path">{{ ws.path }}</div>
        </div>
        <div v-if="workspaces.length === 0" class="empty">
          {{ t('moveToWorkspace.noWorkspaces') }}
        </div>
      </div>
      <div class="dialog-footer">
        <button class="secondary" @click="emit('close')">{{ t('common.cancel') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: var(--bg-overlay);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  animation: overlay-in 0.2s ease;
}
@keyframes overlay-in { from { opacity: 0; } to { opacity: 1; } }

.dialog {
  background: var(--bg-secondary);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border);
  width: 400px;
  max-height: 500px;
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-lg);
  animation: dialog-in 0.25s var(--ease-spring);
}
@keyframes dialog-in { from { opacity: 0; transform: scale(0.95) translateY(-8px); } }

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
}
.dialog-header h3 {
  font-size: 15px;
  font-weight: 650;
  margin: 0;
  letter-spacing: -0.01em;
}
.close-btn {
  background: none;
  border: none;
  font-size: 18px;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px 8px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}
.close-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}
.dialog-body {
  flex: 1;
  overflow-y: auto;
  padding: 12px 20px;
}
.workspace-item {
  padding: 12px 16px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-bottom: 8px;
  border: 1px solid transparent;
}
.workspace-item:hover {
  background: var(--bg-tertiary);
  border-color: var(--border);
}
.ws-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}
.ws-path {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.empty {
  text-align: center;
  padding: 24px;
  color: var(--text-muted);
  font-size: 13px;
}
.dialog-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 20px;
  border-top: 1px solid var(--border-light);
}
.secondary {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 8px 16px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.secondary:hover {
  background: var(--bg-elevated);
  border-color: var(--accent);
}
</style>
