<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const emit = defineEmits<{ create: [name: string, path: string]; close: [] }>()

const name = ref('')
const path = ref('')
const error = ref('')

async function browseFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: t('workspace.browseFolderTitle'),
  })
  if (selected) {
    path.value = selected as string
    if (!name.value) {
      const parts = (selected as string).replace(/[/\\]$/, '').split(/[/\\]/)
      name.value = parts[parts.length - 1] || ''
    }
  }
}

function submit() {
  if (!name.value.trim() || !path.value.trim()) {
    error.value = t('workspace.namePathRequired')
    return
  }
  emit('create', name.value.trim(), path.value.trim())
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog">
      <div class="dialog-icon">📁</div>
      <h2>{{ t('workspace.createTitle') }}</h2>
      <p class="dialog-desc">{{ t('workspace.emptyHint') }}</p>
      <div class="field">
        <label>{{ t('workspace.name') }}</label>
        <input v-model="name" :placeholder="t('workspace.namePlaceholder')" @keyup.enter="submit" />
      </div>
      <div class="field">
        <label>{{ t('workspace.folderPath') }}</label>
        <div class="path-row">
          <input v-model="path" :placeholder="t('workspace.pathPlaceholder')" @keyup.enter="submit" />
          <button class="primary browse-btn" @click="browseFolder">{{ t('workspace.browse') }}</button>
        </div>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="actions">
        <button class="secondary" @click="emit('close')">{{ t('common.cancel') }}</button>
        <button class="primary" @click="submit">{{ t('workspace.createTitle') }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0;
  background: var(--bg-overlay);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
  animation: overlay-in 0.2s ease;
}
@keyframes overlay-in { from { opacity: 0; } to { opacity: 1; } }

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 32px;
  width: 440px;
  box-shadow: var(--shadow-lg);
  animation: dialog-in 0.25s var(--ease-spring);
}
@keyframes dialog-in { from { opacity: 0; transform: scale(0.95) translateY(-8px); } }
.dialog-icon { font-size: 36px; margin-bottom: 8px; }
.dialog h2 { font-size: 20px; font-weight: 700; margin-bottom: 4px; letter-spacing: -0.02em; }
.dialog-desc { font-size: 13px; color: var(--text-muted); margin-bottom: 24px; }
.field { margin-bottom: 16px; }
.field label { display: block; margin-bottom: 6px; font-size: 13px; font-weight: 500; color: var(--text-secondary); }
.field input { width: 100%; }
.path-row { display: flex; gap: 8px; }
.path-row input { flex: 1; }
.browse-btn { white-space: nowrap; font-weight: 600; }
.error { color: var(--danger); font-size: 12px; margin-bottom: 12px; }
.actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 4px; }
</style>
