<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useI18n } from 'vue-i18n'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useTagStore } from '../../stores/tags'

const props = defineProps<{ workspaceId: number; workspaceName?: string }>()
const emit = defineEmits<{
  showToast: [msg: string]
  showDedup: []
  refresh: []
  locateLast: []
}>()

const { t } = useI18n()
const router = useRouter()
const imageStore = useImageStore()
const selection = useSelectionStore()
const tagStore = useTagStore()

const selectMode = defineModel<boolean>('selectMode', { default: false })
const showSortMenu = ref(false)
const showExportMenu = ref(false)

function goBack() { router.push('/') }

const sortOptions = computed(() => [
  { value: 'name', label: t('sort.name') },
  { value: 'indexed_at', label: t('sort.indexedAt') },
  { value: 'updated_at', label: t('sort.updatedAt') },
  { value: 'file_created_at', label: t('sort.fileCreatedAt') },
])

const currentSortLabel = computed(() => {
  const opt = sortOptions.value.find(o => o.value === imageStore.filters.sort_by)
  return opt ? opt.label : t('sort.indexedAt')
})

const allSelected = computed(() =>
  imageStore.images.length > 0 && selection.count === imageStore.images.length
)

function onSort(by: string) {
  const dir = imageStore.filters.sort_by === by && imageStore.filters.sort_dir === 'desc' ? 'asc' : 'desc'
  imageStore.setSort(by, dir, props.workspaceId)
  showSortMenu.value = false
}

function toggleSelectAll() {
  if (allSelected.value) {
    selection.clear()
  } else {
    selection.selectAll(imageStore.images.map(i => i.id))
  }
}

function toggleSelectMode() {
  selectMode.value = !selectMode.value
  if (!selectMode.value) selection.clear()
}

async function exportMetadata(format: 'json' | 'csv') {
  showExportMenu.value = false
  const ids = selection.count > 0 ? Array.from(selection.selectedIds) : null

  const filePath = await save({
    defaultPath: `pic-mgr-export.${format}`,
    filters: [{ name: format.toUpperCase(), extensions: [format] }],
  })
  if (!filePath) return

  try {
    const result = await invoke<string>('export_metadata', {
      workspaceId: props.workspaceId,
      imageIds: ids,
      format,
    })
    await writeTextFile(filePath, result)
    emit('showToast', t('browser.exportSuccess', { path: filePath }))
  } catch (e) {
    console.error('Export failed:', e)
    emit('showToast', t('browser.exportFailed', { error: String(e) }))
  }
}

async function batchDelete() {
  if (selection.count === 0) return
  const msg = t('browser.confirmDelete', { count: selection.count })
  if (!confirm(msg)) return

  try {
    const result = await invoke<{ success: number; failed: number; errors: string[] }>('trash_images', {
      ids: Array.from(selection.selectedIds),
    })
    if (result.failed > 0) {
      emit('showToast', t('browser.deletePartial', { success: result.success, failed: result.failed }))
    } else {
      emit('showToast', t('browser.deleteSuccess', { count: result.success }))
    }
    selection.clear()
    selectMode.value = false
    imageStore.setPage(1)
    await imageStore.fetchImages(props.workspaceId)
    await tagStore.fetchTags(props.workspaceId)
  } catch (e) {
    console.error('Delete failed:', e)
    emit('showToast', t('browser.deleteFailed', { error: String(e) }))
  }
}

async function batchMove() {
  if (selection.count === 0) return

  const selected = await open({
    directory: true,
    multiple: false,
    title: t('browser.moveDialogTitle'),
  })
  if (!selected) return

  try {
    const destDir = typeof selected === 'string' ? selected : selected[0]
    const result = await invoke<{ success: number; failed: number; errors: string[] }>('move_images', {
      ids: Array.from(selection.selectedIds),
      destDir,
    })
    if (result.failed > 0) {
      emit('showToast', t('browser.movePartial', { success: result.success, failed: result.failed }))
    } else {
      emit('showToast', t('browser.moveSuccess', { count: result.success }))
    }
    selection.clear()
    selectMode.value = false
    imageStore.setPage(1)
    await imageStore.fetchImages(props.workspaceId)
    await tagStore.fetchTags(props.workspaceId)
  } catch (e) {
    console.error('Move failed:', e)
    emit('showToast', t('browser.moveFailed', { error: String(e) }))
  }
}
</script>

<template>
  <div class="grid-header">
    <div class="header-left">
      <span class="workspace-name" v-if="props.workspaceName" @click="goBack" :title="t('browser.backToWorkspaces')">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10 2L4 8l6 6"/>
        </svg>
        {{ props.workspaceName }}
      </span>
      <span class="image-count">{{ t('browser.imageCount', { count: imageStore.total }) }}</span>
      <div class="sort-dropdown">
        <button class="toolbar-btn sort-btn" aria-haspopup="listbox" :aria-expanded="showSortMenu" @click="showSortMenu = !showSortMenu">
          {{ currentSortLabel }}
          <span class="caret">{{ imageStore.filters.sort_dir === 'asc' ? '↑' : '↓' }}</span>
        </button>
        <div v-if="showSortMenu" class="sort-menu" role="listbox">
          <button
            v-for="opt in sortOptions"
            :key="opt.value"
            :class="{ active: imageStore.filters.sort_by === opt.value }"
            @click="onSort(opt.value)"
          >
            {{ opt.label }}
            <span v-if="imageStore.filters.sort_by === opt.value" class="dir-indicator">
              {{ imageStore.filters.sort_dir === 'asc' ? '↑' : '↓' }}
            </span>
          </button>
        </div>
      </div>
    </div>
    <div class="header-actions">
      <template v-if="selectMode">
        <span class="selection-info">{{ t('browser.selectedCount', { count: selection.count }) }}</span>
        <div class="btn-group">
          <button class="toolbar-btn" @click="toggleSelectAll">
            {{ allSelected ? t('browser.deselectAll') : t('browser.selectAll') }}
          </button>
        </div>
        <div class="divider"></div>
        <div class="btn-group">
          <button class="toolbar-btn" @click="batchMove">{{ t('browser.move') }}</button>
          <button class="toolbar-btn danger" @click="batchDelete">{{ t('browser.trash') }}</button>
        </div>
        <div class="divider"></div>
        <div class="export-dropdown">
          <button class="toolbar-btn" aria-haspopup="menu" :aria-expanded="showExportMenu" @click="showExportMenu = !showExportMenu">{{ t('browser.exportSelected') }}</button>
          <div v-if="showExportMenu" class="export-menu" role="menu">
            <button role="menuitem" @click="exportMetadata('json')">JSON</button>
            <button role="menuitem" @click="exportMetadata('csv')">CSV</button>
          </div>
        </div>
        <button class="toolbar-btn primary" @click="toggleSelectMode">{{ t('browser.done') }}</button>
      </template>
      <template v-else>
        <button
          v-if="imageStore.lastSelectedId != null"
          class="toolbar-btn"
          :title="t('browser.locateLast')"
          @click="emit('locateLast')"
        >
          {{ t('browser.locateLast') }}
        </button>
        <button class="toolbar-btn" @click="toggleSelectMode">{{ t('browser.select') }}</button>
        <button class="toolbar-btn" @click="emit('showDedup')">{{ t('browser.findDuplicates') }}</button>
        <div class="export-dropdown">
          <button class="toolbar-btn" aria-haspopup="menu" :aria-expanded="showExportMenu" @click="showExportMenu = !showExportMenu">{{ t('browser.export') }}</button>
          <div v-if="showExportMenu" class="export-menu" role="menu">
            <button role="menuitem" @click="exportMetadata('json')">JSON</button>
            <button role="menuitem" @click="exportMetadata('csv')">CSV</button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.grid-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 6px 0;
  padding: 10px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}
.image-count { padding: 5px 10px; font-size: 12px; font-weight: 500; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-secondary); white-space: nowrap; }
.header-left { display: flex; align-items: center; flex-wrap: wrap; gap: 8px 6px; }

.workspace-name { cursor: pointer; display: flex; align-items: center; gap: 6px; padding: 5px 10px; font-size: 12px; font-weight: 600; border: 1px solid var(--border); border-radius: var(--radius-sm); color: var(--text-primary); max-width: 260px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.workspace-name:hover { background: var(--bg-tertiary); }
.workspace-name svg { flex-shrink: 0; }

.header-actions { display: flex; align-items: center; flex-wrap: wrap; gap: 6px 4px; }
.btn-group { display: flex; align-items: center; gap: 4px; }
.divider { width: 1px; height: 18px; background: var(--border); margin: 0 4px; flex-shrink: 0; }
.sort-dropdown { position: relative; }
.sort-btn { font-weight: 500 !important; color: var(--text-secondary) !important; }
.sort-btn .caret { font-size: 10px; opacity: 0.6; margin-left: 2px; }
.sort-menu {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 6px;
  min-width: 150px;
  box-shadow: var(--shadow-md);
  z-index: 100;
  animation: menu-in 0.15s ease;
}
.sort-menu button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  text-align: left;
  padding: 7px 12px;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 12.5px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  font-weight: 450;
  transition: all var(--transition-fast);
}
.sort-menu button:hover { background: var(--accent-bg); color: var(--accent); }
.sort-menu button.active { color: var(--accent); font-weight: 600; }
.sort-menu button .dir-indicator { font-size: 11px; opacity: 0.7; }
.selection-info {
  font-size: 13px;
  color: var(--accent);
  font-weight: 600;
  background: var(--accent-bg);
  padding: 4px 12px;
  border-radius: var(--radius-sm);
}
.toolbar-btn {
  padding: 5px 10px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}
.toolbar-btn:hover { background: var(--bg-tertiary); color: var(--text-primary); border-color: var(--text-muted); }
.toolbar-btn:active { background: var(--border); }
/* Toolbar button variants: same shape, padding, border treatment.
   Differ only by text + border color. Hover fills with the variant color. */
.toolbar-btn.primary {
  color: var(--accent);
  border-color: var(--accent);
  font-weight: 600;
}
.toolbar-btn.primary:hover {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
}
.toolbar-btn.danger {
  color: var(--danger);
  border-color: var(--danger);
  font-weight: 600;
}
.toolbar-btn.danger:hover {
  background: var(--danger);
  color: #fff;
  border-color: var(--danger);
}
.export-dropdown { position: relative; }
.export-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 6px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 6px;
  min-width: 130px;
  box-shadow: var(--shadow-md);
  z-index: 100;
  animation: menu-in 0.15s ease;
}
@keyframes menu-in { from { opacity: 0; transform: translateY(-4px); } }
.export-menu button {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  text-align: left;
  padding: 7px 12px;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 12.5px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  font-weight: 450;
  transition: all var(--transition-fast);
}
.export-menu button:hover { background: var(--accent-bg); color: var(--accent); }
.export-menu button:active { opacity: 0.8; }
</style>
