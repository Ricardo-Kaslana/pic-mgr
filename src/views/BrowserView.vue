<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useWorkspaceStore } from '../stores/workspace'
import { useImageStore } from '../stores/images'
import { useSelectionStore } from '../stores/selection'
import { useTagStore } from '../stores/tags'
import SidebarPanel from '../components/browser/SidebarPanel.vue'
import ThumbnailGrid from '../components/browser/ThumbnailGrid.vue'
import DetailPanel from '../components/browser/DetailPanel.vue'

const route = useRoute()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const imageStore = useImageStore()
const selection = useSelectionStore()
const tagStore = useTagStore()

const workspaceId = computed(() => Number(route.params.id))
const showExportMenu = ref(false)
const selectMode = ref(false)

function toggleSelectMode() {
  selectMode.value = !selectMode.value
  if (!selectMode.value) {
    selection.clear()
  }
}

function selectAll() {
  selection.selectAll(imageStore.images.map(i => i.id))
}

function deselectAll() {
  selection.clear()
}

onMounted(async () => {
  await workspaceStore.fetchAll()
  workspaceStore.select(workspaceId.value)
  await imageStore.fetchImages(workspaceId.value)
  await tagStore.fetchTags(workspaceId.value)
})

const toastMessage = ref('')
const toastVisible = ref(false)

function showToast(msg: string) {
  toastMessage.value = msg
  toastVisible.value = true
  setTimeout(() => { toastVisible.value = false }, 3000)
}

async function exportMetadata(format: 'json' | 'csv') {
  showExportMenu.value = false

  // Determine which images to export
  const ids = selection.count > 0 ? Array.from(selection.selectedIds) : null

  // Open save dialog
  const filePath = await save({
    defaultPath: `pic-mgr-export.${format}`,
    filters: [{ name: format.toUpperCase(), extensions: [format] }],
  })
  if (!filePath) return

  try {
    const result = await invoke<string>('export_metadata', {
      workspaceId: workspaceId.value,
      imageIds: ids,
      format,
    })
    await writeTextFile(filePath, result)
    showToast(`导出成功：${filePath}`)
  } catch (e) {
    console.error('Export failed:', e)
    showToast(`导出失败：${e}`)
  }
}

// Keyboard shortcuts
function onKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

  switch (e.key) {
    case 'Escape':
      if (selectMode.value) {
        selectMode.value = false
        selection.clear()
      } else {
        imageStore.selectedDetail = null
      }
      break
    case 'a':
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault()
        if (selectMode.value) selectAll()
      }
      break
  }
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onUnmounted(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <div class="browser">
    <SidebarPanel :workspace-id="workspaceId" />
    <div class="grid-area">
      <div class="grid-header">
        <span class="image-count">{{ imageStore.total }} 张图片</span>
        <div class="header-actions">
          <template v-if="selectMode">
            <span class="selection-info">已选 {{ selection.count }} 张</span>
            <button class="toolbar-btn" @click="selectAll">全选</button>
            <button class="toolbar-btn" @click="deselectAll">取消全选</button>
            <div class="export-dropdown">
              <button class="toolbar-btn" @click="showExportMenu = !showExportMenu">📤 导出选中</button>
              <div v-if="showExportMenu" class="export-menu">
                <button @click="exportMetadata('json')">JSON 格式</button>
                <button @click="exportMetadata('csv')">CSV 格式</button>
              </div>
            </div>
            <button class="toolbar-btn active" @click="toggleSelectMode">✓ 完成</button>
          </template>
          <template v-else>
            <button class="toolbar-btn" @click="toggleSelectMode">☐ 选择</button>
            <button class="toolbar-btn" @click="router.push(`/workspace/${workspaceId}/dedup`)">🔍 查重</button>
            <div class="export-dropdown">
              <button class="toolbar-btn" @click="showExportMenu = !showExportMenu">📤 导出</button>
              <div v-if="showExportMenu" class="export-menu">
                <button @click="exportMetadata('json')">JSON 格式</button>
                <button @click="exportMetadata('csv')">CSV 格式</button>
              </div>
            </div>
          </template>
        </div>
      </div>
      <ThumbnailGrid
        :workspace-id="workspaceId"
        :select-mode="selectMode"
        @select-detail="(id: number) => imageStore.fetchDetail(id)"
      />
    </div>
    <DetailPanel v-if="imageStore.selectedDetail" :workspace-id="workspaceId" />
  </div>

  <!-- Toast notification -->
  <Teleport to="body">
    <Transition name="toast">
      <div v-if="toastVisible" class="toast">{{ toastMessage }}</div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.browser {
  display: flex;
  height: 100%;
}
.grid-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.grid-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}
.image-count { font-size: 13px; color: var(--text-secondary); }
.header-actions { display: flex; align-items: center; gap: 12px; }
.selection-info { font-size: 12px; color: var(--accent); font-weight: 500; }
.toolbar-btn {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  cursor: pointer;
}
.toolbar-btn:hover { border-color: var(--accent); }
.toolbar-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.export-dropdown { position: relative; }
.export-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 4px;
  min-width: 120px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  z-index: 100;
}
.export-menu button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 6px 10px;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 12px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.export-menu button:hover { background: var(--accent-bg); }

:global(.toast) {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 20px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  font-size: 13px;
  color: var(--text-primary);
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
  z-index: 2000;
  max-width: 500px;
  word-break: break-all;
}
:global(.toast-enter-active), :global(.toast-leave-active) { transition: all 0.3s ease; }
:global(.toast-enter-from), :global(.toast-leave-to) { opacity: 0; transform: translateX(-50%) translateY(10px); }
</style>
