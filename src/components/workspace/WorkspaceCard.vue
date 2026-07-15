<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { Workspace } from '../../types'

const props = defineProps<{
  workspace: Workspace
  scanning?: boolean
  justScanned?: boolean
  syncing?: boolean
}>()
const emit = defineEmits<{
  select: [id: number]
  delete: [id: number]
  rescan: [id: number]
  fullReset: [id: number]
}>()

const { t } = useI18n()

function confirmFullReset() {
  if (!confirm(t('workspace.fullResetConfirm'))) return
  if (!confirm(t('workspace.fullResetConfirm2'))) return
  emit('fullReset', props.workspace.id)
}
</script>

<template>
  <div class="card" @click="emit('select', workspace.id)">
    <div class="card-front">
      <div class="card-icon">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
        </svg>
      </div>
      <div class="card-body">
        <h3>{{ workspace.name }}</h3>
        <p class="path">{{ workspace.path }}</p>
      </div>
      <div class="card-meta">
        <span class="date">{{ workspace.updated_at?.replace('T', ' ') }}</span>
      </div>
    </div>
    <div class="card-actions">
      <details class="advanced-menu" @click.stop>
        <summary :title="t('workspace.advancedReset')">...</summary>
        <div class="advanced-actions">
          <button class="danger-link" type="button" @click="confirmFullReset">
            {{ t('workspace.fullResetConfirm') }}
          </button>
        </div>
      </details>
      <button
        class="action-btn rescan-btn"
        :class="{ scanning: scanning || syncing, done: justScanned }"
        :disabled="scanning || syncing"
        @click.stop="emit('rescan', workspace.id)"
        :title="(scanning || syncing) ? t('workspace.syncing') : justScanned ? t('workspace.scannedDone') : t('workspace.rescan')"
      >
        <svg v-if="justScanned" class="icon-check" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"/>
        </svg>
        <svg v-else class="icon-refresh" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
        </svg>
      </button>
      <button
        class="action-btn danger"
        @click.stop="emit('delete', workspace.id)"
        :title="t('workspace.delete')"
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="3 6 5 6 21 6"/>
          <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 18px 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all var(--transition);
}
.card:hover {
  border-color: var(--accent);
  background: var(--bg-tertiary);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.card-front {
  display: flex;
  align-items: center;
  gap: 14px;
  flex: 1;
  min-width: 0;
}
.card-icon {
  width: 44px; height: 44px;
  background: var(--accent-bg);
  border-radius: var(--radius);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
  flex-shrink: 0;
}
.card-body { flex: 1; min-width: 0; }
.card-body h3 {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 3px;
  letter-spacing: -0.01em;
}
.path {
  font-size: 12px;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 400px;
}
.card-meta { flex-shrink: 0; }
.date {
  font-size: 11px;
  color: var(--text-muted);
  opacity: 0.7;
}

.card-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--transition-fast);
  align-items: center;
}
.card:hover .card-actions { opacity: 1; }
.card:has(.scanning) .card-actions,
.card:has(.done) .card-actions { opacity: 1; }
.action-btn {
  width: 34px; height: 34px;
  border-radius: var(--radius-xs);
  background: var(--bg-tertiary);
  color: var(--text-muted);
  font-size: 14px;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-fast);
  border: 1px solid transparent;
}
.action-btn:hover { background: var(--bg-elevated); color: var(--text-primary); border-color: var(--border); }
.action-btn:active { opacity: 0.7; transform: scale(0.95); }
.action-btn:disabled { opacity: 0.5; cursor: not-allowed; pointer-events: none; }
.action-btn.danger:hover { background: var(--danger-bg); color: var(--danger); border-color: var(--danger); }

.rescan-btn.scanning {
  color: var(--accent);
  border-color: var(--accent);
  background: var(--accent-bg);
  pointer-events: none;
}
.rescan-btn.scanning .icon-refresh {
  animation: spin 0.8s linear infinite;
}
.rescan-btn.done {
  color: var(--success, #22c55e);
  border-color: var(--success, #22c55e);
  background: color-mix(in srgb, var(--success, #22c55e) 10%, transparent);
}
.icon-check {
  animation: pop-in 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
@keyframes pop-in {
  0% { transform: scale(0); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}

.advanced-menu { position: relative; }
.advanced-menu summary {
  list-style: none;
  cursor: pointer;
  width: 34px; height: 34px;
  border-radius: var(--radius-xs);
  background: var(--bg-tertiary);
  color: var(--text-muted);
  display: flex; align-items: center; justify-content: center;
  border: 1px solid transparent;
  transition: all var(--transition-fast);
  font-size: 14px;
  user-select: none;
}
.advanced-menu summary::-webkit-details-marker { display: none; }
.advanced-menu summary:hover { background: var(--bg-elevated); color: var(--text-primary); border-color: var(--border); }
.advanced-actions {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  padding: 8px 12px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-md);
  z-index: 10;
  min-width: 220px;
}
.danger-link {
  background: none; border: none; padding: 0;
  color: var(--danger); cursor: pointer; font-size: 12px;
  text-decoration: none;
  text-align: left;
  width: 100%;
}
.danger-link:hover { text-decoration: underline; }
</style>
