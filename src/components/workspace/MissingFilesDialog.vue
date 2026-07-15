<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { DeletedCandidate } from '../../types'

const props = defineProps<{ candidates: DeletedCandidate[] }>()
const emit = defineEmits<{
  apply: [ids: number[]]
  close: []
}>()

const { t } = useI18n()

// All candidates start selected; the user can deselect individual rows.
const selected = ref<Set<number>>(new Set(props.candidates.map(c => c.id)))

const selectionCount = computed(() => selected.value.size)
const empty = computed(() => selectionCount.value === 0)

function toggle(id: number) {
  if (selected.value.has(id)) selected.value.delete(id)
  else selected.value.add(id)
  // Vue 3 reactivity requires replacing the Set for changes to propagate.
  selected.value = new Set(selected.value)
}

function selectAll() {
  selected.value = new Set(props.candidates.map(c => c.id))
}

function deselectAll() {
  selected.value = new Set()
}

function onApply() {
  if (empty.value) return
  emit('apply', Array.from(selected.value))
}

function onKeepAll() {
  emit('close')
}

function formatTags(names: string[]): string {
  return names.length === 0 ? '--' : names.join(', ')
}

function ratingStars(n: number | null): string {
  if (n == null) return '--'
  return '\u2605'.repeat(n) + '\u2606'.repeat(5 - n)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ t('missing.title', { count: candidates.length }) }}</h3>
        <button class="close-btn" @click="emit('close')" type="button">x</button>
      </div>
      <div class="dialog-body">
        <p class="dialog-desc">{{ t('missing.description') }}</p>
        <div class="list-actions">
          <button class="link" @click="selectAll" type="button">
            {{ t('common.confirm') }}
          </button>
          <span class="dot">.</span>
          <button class="link" @click="deselectAll" type="button">
            {{ t('common.cancel') }}
          </button>
          <span class="selection-count">
            {{ t('missing.deleteSelected', { count: selectionCount }) }}
          </span>
        </div>
        <ul class="candidates">
          <li
            v-for="c in candidates"
            :key="c.id"
            class="candidate"
            :class="{ checked: selected.has(c.id) }"
            @click="toggle(c.id)"
          >
            <input
              type="checkbox"
              :checked="selected.has(c.id)"
              @click.stop="toggle(c.id)"
            />
            <div class="meta">
              <div class="name">{{ c.fileName }}</div>
              <div class="path">{{ c.relativePath }}</div>
              <div class="badges">
                <span class="rating">{{ ratingStars(c.rating) }}</span>
                <span class="tags">{{ formatTags(c.tagNames) }}</span>
              </div>
            </div>
          </li>
        </ul>
      </div>
      <div class="dialog-footer">
        <button class="secondary" @click="onKeepAll" type="button">
          {{ t('missing.keepAll') }}
        </button>
        <button
          class="primary"
          :disabled="empty"
          @click="onApply"
          type="button"
        >
          {{ t('missing.deleteSelected', { count: selectionCount }) }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-overlay {
  position: fixed; inset: 0;
  background: var(--bg-overlay);
  backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
  animation: overlay-in 0.2s ease;
}
@keyframes overlay-in { from { opacity: 0; } to { opacity: 1; } }

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  width: 520px;
  max-height: 70vh;
  display: flex; flex-direction: column;
  box-shadow: var(--shadow-lg);
  animation: dialog-in 0.25s var(--ease-spring);
}
@keyframes dialog-in { from { opacity: 0; transform: scale(0.95) translateY(-8px); } }

.dialog-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-light);
}
.dialog-header h3 {
  font-size: 15px; font-weight: 650; margin: 0;
  letter-spacing: -0.01em;
}
.close-btn {
  background: none; border: none;
  font-size: 18px; color: var(--text-muted);
  cursor: pointer; padding: 4px 8px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}
.close-btn:hover { background: var(--bg-tertiary); color: var(--text-primary); }

.dialog-body { flex: 1; overflow-y: auto; padding: 12px 20px; }
.dialog-desc { color: var(--text-secondary); font-size: 13px; margin: 0 0 12px 0; line-height: 1.55; }

.list-actions {
  display: flex; align-items: center; gap: 6px;
  font-size: 12px; color: var(--text-muted);
  margin-bottom: 8px;
}
.link {
  background: none; border: none; padding: 0;
  color: var(--accent); cursor: pointer; font-size: 12px;
  text-decoration: none;
}
.link:hover { text-decoration: underline; }
.dot { color: var(--text-muted); }
.selection-count { margin-left: auto; }

.candidates { list-style: none; padding: 0; margin: 0; }
.candidate {
  display: flex; align-items: flex-start; gap: 10px;
  padding: 10px 12px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.candidate:hover { background: var(--bg-tertiary); }
.candidate.checked { background: var(--bg-tertiary); border-color: var(--border); }
.candidate input[type="checkbox"] { margin-top: 4px; cursor: pointer; }
.meta { flex: 1; min-width: 0; }
.name { font-size: 13px; font-weight: 600; color: var(--text-primary); }
.path { font-size: 11px; color: var(--text-muted); margin-top: 2px;
        overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.badges { display: flex; gap: 10px; margin-top: 4px; font-size: 11px; color: var(--text-secondary); }

.dialog-footer {
  display: flex; justify-content: flex-end; gap: 8px;
  padding: 12px 20px;
  border-top: 1px solid var(--border-light);
}
.secondary {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  padding: 8px 16px; border-radius: var(--radius-sm);
  font-size: 13px; font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.secondary:hover { background: var(--bg-elevated); border-color: var(--accent); }
.primary {
  background: var(--accent);
  color: white; border: none;
  padding: 8px 16px; border-radius: var(--radius-sm);
  font-size: 13px; font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.primary:hover:not(:disabled) { filter: brightness(1.05); }
.primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
