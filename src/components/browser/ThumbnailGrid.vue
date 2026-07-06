<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, reactive, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useSettingsStore } from '../../stores/settings'
import { useThumbnailStore } from '../../stores/thumbnail'
import { useWorkspaceStore } from '../../stores/workspace'
import type { ImageRecord } from '../../types'
import MoveToWorkspaceDialog from '../workspace/MoveToWorkspaceDialog.vue'

const props = defineProps<{
  workspaceId: number
  selectMode: boolean
}>()
const emit = defineEmits<{
  selectDetail: [id: number]
  refresh: []
  viewFullscreen: [index: number]
  showToast: [msg: string]
}>()

const imageStore = useImageStore()
const selection = useSelectionStore()
const settings = useSettingsStore()
const thumbStore = useThumbnailStore()
const workspaceStore = useWorkspaceStore()
const { t } = useI18n()

const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const containerWidth = ref(800)
const containerHeight = ref(600)
// ID of the image currently flashing a brief "return focus" highlight
const flashId = ref<number | null>(null)

const GAP = 12
const COL_MIN = computed(() => Math.max(160, settings.thumbnailSize))

const columns = computed(() =>
  Math.max(1, Math.floor((containerWidth.value + GAP) / (COL_MIN.value + GAP)))
)

const cellWidth = computed(() =>
  (containerWidth.value - GAP * (columns.value - 1)) / columns.value
)

const cellHeight = computed(() => settings.thumbnailSize + 48)

const rows = computed(() =>
  Math.ceil(imageStore.images.length / columns.value)
)

const totalHeight = computed(() =>
  rows.value * (cellHeight.value + GAP)
)

const BUFFER_ROWS = 3

const startRow = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / (cellHeight.value + GAP)) - BUFFER_ROWS)
)

const endRow = computed(() =>
  Math.min(
    rows.value,
    Math.ceil((scrollTop.value + containerHeight.value) / (cellHeight.value + GAP)) + BUFFER_ROWS
  )
)

interface VisibleItem {
  image: ImageRecord
  idx: number
  style: Record<string, string>
}

const visibleItems = computed<VisibleItem[]>(() => {
  const items: VisibleItem[] = []

  for (let r = startRow.value; r < endRow.value; r++) {
    for (let c = 0; c < columns.value; c++) {
      const idx = r * columns.value + c
      if (idx >= imageStore.images.length) break
      items.push({
        image: imageStore.images[idx],
        idx,
        style: {
          position: 'absolute',
          top: `${r * (cellHeight.value + GAP)}px`,
          left: `${c * (cellWidth.value + GAP)}px`,
          width: `${cellWidth.value}px`,
          height: `${cellHeight.value}px`,
        },
      })
    }
  }
  return items
})

// Reactive thumb URL map — separate from store cache for Vue reactivity
const thumbUrls = reactive<Map<number, string>>(new Map())

// Throttled thumbnail loader — fires at most once per 80ms scroll burst
let thumbLoadTimer: ReturnType<typeof setTimeout> | null = null
watch(visibleItems, (items) => {
  if (thumbLoadTimer) clearTimeout(thumbLoadTimer)
  thumbLoadTimer = setTimeout(() => {
    const visibleIds = new Set(items.map(vi => vi.image.id))

    // Load new visible items
    for (const vi of items) {
      const id = vi.image.id
      if (!thumbUrls.has(id) && !thumbStore.loading.has(id)) {
        thumbStore.load(id).then(url => {
          if (url) thumbUrls.set(id, url)
        })
      }
    }

    // Revoke URLs for items that scrolled out of view (keep a buffer of 100)
    if (thumbUrls.size > visibleIds.size + 100) {
      for (const [id] of thumbUrls) {
        if (!visibleIds.has(id)) {
          thumbUrls.delete(id)
          thumbStore.revoke(id)
        }
      }
    }
  }, 80)
}, { immediate: true })

function onScroll(e: Event) {
  const el = e.target as HTMLElement
  scrollTop.value = el.scrollTop

  // Load more when near bottom
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 500) {
    if (!imageStore.loading && imageStore.images.length < imageStore.total) {
      imageStore.setPage(imageStore.page + 1)
      imageStore.fetchImages(props.workspaceId, true)
    }
  }
}

function handleClick(image: ImageRecord, e: MouseEvent) {
  if (props.selectMode) {
    selection.toggle(image.id, e.shiftKey)
  } else {
    emit('selectDetail', image.id)
  }
}

function handleDoubleClick(image: ImageRecord) {
  if (props.selectMode) return
  const idx = imageStore.images.findIndex(i => i.id === image.id)
  if (idx >= 0) emit('viewFullscreen', idx)
}

// Context menu
const contextMenu = reactive({ visible: false, x: 0, y: 0, image: null as ImageRecord | null })
const showWorkspacePicker = ref(false)

function handleContextMenu(image: ImageRecord, e: MouseEvent) {
  e.preventDefault()
  if (!selection.selectedIds.has(image.id)) {
    selection.clear()
    selection.toggle(image.id, false)
  }
  contextMenu.visible = true
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.image = image
}

function closeContextMenu() {
  contextMenu.visible = false
}

function getAbsolutePath(image: ImageRecord): string {
  const ws = workspaceStore.workspaces.find(w => w.id === image.workspace_id)
  if (!ws) return image.relative_path
  return `${ws.path}\\${image.relative_path}`.replace(/\\\\/g, '\\')
}

async function openInExplorer() {
  if (contextMenu.image) {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_in_explorer', { path: getAbsolutePath(contextMenu.image) })
  }
  closeContextMenu()
}

async function copyPath() {
  if (contextMenu.image) {
    await navigator.clipboard.writeText(getAbsolutePath(contextMenu.image))
  }
  closeContextMenu()
}

async function removeFromWorkspace() {
  if (contextMenu.image && confirm(t('context.confirmRemove'))) {
    const { invoke } = await import('@tauri-apps/api/core')
    try {
      await invoke('delete_image', { id: contextMenu.image.id })
      emit('refresh')
    } catch (e) {
      console.error('Remove failed:', e)
    }
  }
  closeContextMenu()
}

async function trashFile() {
  if (!contextMenu.image) return
  if (!confirm(t('context.confirmTrash'))) return
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    await invoke('trash_images', { ids: [contextMenu.image.id] })
    emit('refresh')
  } catch (e) {
    console.error('Trash failed:', e)
  }
  closeContextMenu()
}

async function moveToWorkspace() {
  if (!contextMenu.image) return
  showWorkspacePicker.value = true
  closeContextMenu()
}

async function handleWorkspaceSelected(targetWsId: number) {
  showWorkspacePicker.value = false
  if (!contextMenu.image) return
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    const result = await invoke<{ success: number; failed: number; errors: string[] }>('move_images', {
      ids: [contextMenu.image.id],
      destWorkspaceId: targetWsId,
    })
    if (result.failed > 0) {
      const details = result.errors.length > 0 ? `\n${result.errors.join('\n')}` : ''
      emit('showToast', t('browser.movePartial', { success: result.success, failed: result.failed }) + details)
    } else {
      emit('showToast', t('browser.moveSuccess', { count: result.success }))
    }
    if (result.success > 0) {
      emit('refresh')
    }
  } catch (e) {
    console.error('Move to workspace failed:', e)
    emit('showToast', t('browser.moveFailed', { error: String(e) }))
  }
}

async function moveFile() {
  if (!contextMenu.image) return
  const { invoke } = await import('@tauri-apps/api/core')
  const { open } = await import('@tauri-apps/plugin-dialog')
  const selected = await open({ directory: true, multiple: false, title: t('browser.moveDialogTitle') })
  if (!selected) return
  const destDir = typeof selected === 'string' ? selected : selected[0]
  try {
    await invoke('move_images', { ids: [contextMenu.image.id], destDir })
    emit('refresh')
  } catch (e) {
    console.error('Move failed:', e)
  }
  closeContextMenu()
}

let resizeObserver: ResizeObserver | null = null
onMounted(() => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth
    containerHeight.value = containerRef.value.clientHeight
    resizeObserver = new ResizeObserver(entries => {
      containerWidth.value = entries[0].contentRect.width
      containerHeight.value = entries[0].contentRect.height
    })
    resizeObserver.observe(containerRef.value)
  }
})
onUnmounted(() => {
  resizeObserver?.disconnect()
})

const detailImageId = computed(() => imageStore.selectedDetail?.id ?? null)

function scrollToImage(imageId: number) {
  const idx = imageStore.images.findIndex(i => i.id === imageId)
  if (idx < 0 || !containerRef.value) return
  const row = Math.floor(idx / columns.value)
  const targetTop = row * (cellHeight.value + GAP)
  const el = containerRef.value
  const viewH = el.clientHeight
  const cellBottom = targetTop + cellHeight.value + GAP
  if (targetTop < el.scrollTop || cellBottom > el.scrollTop + viewH) {
    el.scrollTo({ top: Math.max(0, targetTop - viewH / 2 + cellHeight.value / 2), behavior: 'smooth' })
  }
}

async function locateAndFlash(imageId: number) {
  await nextTick()
  await new Promise(r => setTimeout(r, 50))
  scrollToImage(imageId)
  // 清掉旧的 flash，下一帧再设回来，确保同一图片连续触发也能重新播放动画
  flashId.value = null
  await nextTick()
  flashId.value = imageId
  setTimeout(() => { flashId.value = null }, 1200)
}

watch(() => imageStore.selectedDetail, async (detail, prev) => {
  if (detail) {
    // 打开详情：滚动到该图片
    await nextTick()
    await new Promise(r => setTimeout(r, 80))
    scrollToImage(detail.id)
  } else if (!detail && prev) {
    // 关闭详情：定位并短暂高亮上次打开的图片
    const targetId = imageStore.lastSelectedId
    if (targetId == null) return
    locateAndFlash(targetId)
  }
})

defineExpose({ scrollToImage, locateAndFlash })
</script>

<template>
  <div ref="containerRef" class="thumb-grid" @scroll="onScroll" @click="closeContextMenu" @contextmenu.prevent>
    <div class="grid-sizer" :style="{ height: `${totalHeight}px` }">
      <div
        v-for="vi in visibleItems"
        :key="vi.image.id"
        class="grid-cell"
        :style="vi.style"
        :class="{ selected: selectMode && selection.selectedIds.has(vi.image.id), 'detail-active': detailImageId === vi.image.id, 'flash-locate': flashId === vi.image.id }"
        @click="handleClick(vi.image, $event)"
        @dblclick="handleDoubleClick(vi.image)"
        @contextmenu.prevent="handleContextMenu(vi.image, $event)"
      >
        <div class="thumb-img">
          <img
            v-if="thumbUrls.get(vi.image.id)"
            :src="thumbUrls.get(vi.image.id)"
            :alt="vi.image.file_name"
            loading="lazy"
          />
          <div v-else-if="thumbStore.loading.has(vi.image.id)" class="skeleton"></div>
          <div v-else class="placeholder"><span>🖼️</span></div>
        </div>
        <div class="thumb-info">
          <p class="file-name">{{ vi.image.file_name }}</p>
          <div v-if="vi.image.rating" class="rating">
            {{ '★'.repeat(vi.image.rating) }}{{ '☆'.repeat(5 - vi.image.rating) }}
          </div>
        </div>
        <div v-if="selectMode && selection.selectedIds.has(vi.image.id)" class="check-mark">✓</div>
      </div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        role="menu"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      >
        <button role="menuitem" @click="openInExplorer"><span class="ctx-icon">📂</span>{{ t('context.openExplorer') }}</button>
        <button role="menuitem" @click="copyPath"><span class="ctx-icon">📋</span>{{ t('context.copyPath') }}</button>
        <button role="menuitem" @click="moveFile"><span class="ctx-icon">📁</span>{{ t('context.moveFile') }}</button>
        <button role="menuitem" @click="moveToWorkspace"><span class="ctx-icon">🗂️</span>{{ t('context.moveToWorkspace') }}</button>
        <div class="context-divider"></div>
        <button role="menuitem" class="danger" @click="removeFromWorkspace"><span class="ctx-icon">🗑️</span>{{ t('context.removeFromWorkspace') }}</button>
        <button role="menuitem" class="danger" @click="trashFile"><span class="ctx-icon">🗑️</span>{{ t('context.moveToTrash') }}</button>
      </div>
      <MoveToWorkspaceDialog
        v-if="showWorkspacePicker"
        :current-workspace-id="props.workspaceId"
        @select="handleWorkspaceSelected"
        @close="showWorkspacePicker = false"
      />
    </Teleport>
  </div>
</template>

<style scoped>
.thumb-grid {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 12px;
}

.grid-sizer {
  position: relative;
  width: 100%;
}

.grid-cell {
  background: var(--bg-secondary);
  border: 1.5px solid var(--border-light);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: all var(--transition);
  display: flex;
  flex-direction: column;
  position: absolute;
}
.grid-cell:hover {
  border-color: var(--accent);
  transform: translateY(-3px);
  box-shadow: var(--shadow-md), 0 0 0 1px var(--accent-bg);
  z-index: 1;
}
.grid-cell.selected {
  border-color: var(--accent);
  background: var(--bg-tertiary);
  box-shadow: 0 0 0 2px var(--accent-bg);
}
.grid-cell.selected::before {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--accent-bg);
  pointer-events: none;
  z-index: 0;
}
.grid-cell.detail-active {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent), 0 0 12px var(--accent-bg);
  z-index: 2;
}
.grid-cell.flash-locate {
  animation: flash-locate 1.2s ease;
  z-index: 2;
}
@keyframes flash-locate {
  0%   { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent), 0 0 20px var(--accent-glow); }
  40%  { border-color: var(--accent); box-shadow: 0 0 0 3px var(--accent), 0 0 20px var(--accent-glow); }
  100% { border-color: var(--border-light); box-shadow: none; }
}

.thumb-img {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  min-height: 0;
  position: relative;
}
.thumb-img img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
  transition: transform var(--transition-slow);
}
.grid-cell:hover .thumb-img img {
  transform: scale(1.03);
}
.placeholder {
  font-size: 32px;
  color: var(--text-muted);
  opacity: 0.4;
}
.skeleton {
  width: 100%;
  height: 100%;
  background: linear-gradient(110deg, var(--bg-tertiary) 25%, var(--bg-elevated) 40%, var(--bg-tertiary) 55%);
  background-size: 200% 100%;
  animation: skeleton-shimmer 1.8s ease-in-out infinite;
}
@keyframes skeleton-shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.thumb-info {
  padding: 7px 9px;
  border-top: 1px solid var(--border-light);
  position: relative;
  z-index: 1;
  background: var(--bg-secondary);
}
.file-name {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 450;
}
.rating { font-size: 10px; color: var(--warning); margin-top: 2px; }

.check-mark {
  position: absolute; top: 7px; right: 7px;
  width: 24px; height: 24px;
  background: var(--accent); color: #fff;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 13px; font-weight: 700;
  box-shadow: var(--shadow-sm);
  z-index: 2;
}

.context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 6px;
  min-width: 190px;
  box-shadow: var(--shadow-lg);
  animation: menu-in 0.12s ease;
}
@keyframes menu-in { from { opacity: 0; transform: scale(0.96); } }
.context-menu button {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 0;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 13px;
  border-radius: var(--radius-xs);
  cursor: pointer;
  font-weight: 450;
  transition: all var(--transition-fast);
}
.ctx-icon {
  width: 22px;
  text-align: center;
  flex-shrink: 0;
  font-size: 14px;
}
.context-menu button:hover { background: var(--accent-bg); color: var(--accent); }
.context-menu button:active { opacity: 0.8; }
.context-menu button.danger:hover { background: var(--danger-bg); color: var(--danger); }
.context-divider { height: 1px; background: var(--border); margin: 5px 0; }
</style>
