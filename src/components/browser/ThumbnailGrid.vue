<script setup lang="ts">
import { ref, computed, onMounted, watch, reactive } from 'vue'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useSettingsStore } from '../../stores/settings'
import { useThumbnailStore } from '../../stores/thumbnail'
import type { ImageRecord } from '../../types'

const props = defineProps<{ workspaceId: number }>()
const emit = defineEmits<{ selectDetail: [id: number] }>()

const imageStore = useImageStore()
const selection = useSelectionStore()
const settings = useSettingsStore()
const thumbStore = useThumbnailStore()

const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const containerWidth = ref(800)
const containerHeight = ref(600)

const GAP = 12
const COL_MIN = 160

const columns = computed(() =>
  Math.max(1, Math.floor((containerWidth.value + GAP) / (COL_MIN + GAP)))
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

// Load thumbnails for newly visible items
watch(visibleItems, (items) => {
  for (const vi of items) {
    const id = vi.image.id
    if (!thumbUrls.has(id) && !thumbStore.loading.has(id)) {
      thumbStore.load(id).then(url => {
        if (url) thumbUrls.set(id, url)
      })
    }
  }
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
  selection.toggle(image.id, e.shiftKey)
}

function handleDblClick(image: ImageRecord) {
  emit('selectDetail', image.id)
}

// Context menu
const contextMenu = reactive({ visible: false, x: 0, y: 0, image: null as ImageRecord | null })

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

async function openInExplorer() {
  if (contextMenu.image) {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('open_in_explorer', { path: contextMenu.image.file_path })
  }
  closeContextMenu()
}

async function copyPath() {
  if (contextMenu.image) {
    await navigator.clipboard.writeText(contextMenu.image.file_path)
  }
  closeContextMenu()
}

async function removeFromWorkspace() {
  if (contextMenu.image && confirm('从工作区移除此图片？不会删除原始文件。')) {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('delete_image', { id: contextMenu.image.id })
    imageStore.fetchImages(props.workspaceId)
  }
  closeContextMenu()
}

onMounted(() => {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth
    containerHeight.value = containerRef.value.clientHeight
    const observer = new ResizeObserver(entries => {
      containerWidth.value = entries[0].contentRect.width
      containerHeight.value = entries[0].contentRect.height
    })
    observer.observe(containerRef.value)
  }
})
</script>

<template>
  <div ref="containerRef" class="thumb-grid" @scroll="onScroll" @click="closeContextMenu" @contextmenu.prevent>
    <div class="grid-sizer" :style="{ height: `${totalHeight}px` }">
      <div
        v-for="vi in visibleItems"
        :key="vi.image.id"
        class="grid-cell"
        :style="vi.style"
        :class="{ selected: selection.selectedIds.has(vi.image.id) }"
        @click="handleClick(vi.image, $event)"
        @dblclick="handleDblClick(vi.image)"
        @contextmenu.prevent="handleContextMenu(vi.image, $event)"
      >
        <div class="thumb-img">
          <img
            v-if="thumbUrls.get(vi.image.id)"
            :src="thumbUrls.get(vi.image.id)"
            :alt="vi.image.file_name"
          />
          <div v-else class="placeholder"><span>🖼️</span></div>
        </div>
        <div class="thumb-info">
          <p class="file-name">{{ vi.image.file_name }}</p>
          <div v-if="vi.image.rating" class="rating">
            {{ '★'.repeat(vi.image.rating) }}{{ '☆'.repeat(5 - vi.image.rating) }}
          </div>
        </div>
        <div v-if="selection.selectedIds.has(vi.image.id)" class="check-mark">✓</div>
      </div>
    </div>

    <!-- Context Menu -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible"
        class="context-menu"
        :style="{ left: `${contextMenu.x}px`, top: `${contextMenu.y}px` }"
      >
        <button @click="openInExplorer">📂 在资源管理器中打开</button>
        <button @click="copyPath">📋 复制文件路径</button>
        <div class="context-divider"></div>
        <button class="danger" @click="removeFromWorkspace">🗑️ 从工作区移除</button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.thumb-grid {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0 12px;
}

.grid-sizer {
  position: relative;
  width: 100%;
}

.grid-cell {
  background: var(--bg-secondary);
  border: 2px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: border-color var(--transition);
  display: flex;
  flex-direction: column;
  position: absolute;
}
.grid-cell:hover { border-color: var(--accent); }
.grid-cell.selected { border-color: var(--accent); background: var(--bg-tertiary); }

.thumb-img {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  min-height: 0;
}
.thumb-img img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.placeholder {
  font-size: 32px;
  color: var(--text-muted);
}

.thumb-info {
  padding: 6px 8px;
  border-top: 1px solid var(--border);
}
.file-name {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rating { font-size: 10px; color: var(--warning); margin-top: 2px; }

.check-mark {
  position: absolute; top: 6px; right: 6px;
  width: 22px; height: 22px;
  background: var(--accent); color: white;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700;
}

.context-menu {
  position: fixed;
  z-index: 1000;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 4px;
  min-width: 180px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.context-menu button {
  display: block;
  width: 100%;
  text-align: left;
  padding: 8px 12px;
  border: none;
  background: none;
  color: var(--text-primary);
  font-size: 13px;
  border-radius: var(--radius-sm);
  cursor: pointer;
}
.context-menu button:hover { background: var(--accent-bg); }
.context-menu button.danger:hover { background: var(--danger); color: white; }
.context-divider { height: 1px; background: var(--border); margin: 4px 0; }
</style>
