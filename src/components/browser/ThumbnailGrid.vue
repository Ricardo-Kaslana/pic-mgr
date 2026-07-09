<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, shallowRef, triggerRef, watch, reactive, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useSettingsStore } from '../../stores/settings'
import { useThumbnailStore } from '../../stores/thumbnail'
import { useWorkspaceStore } from '../../stores/workspace'
import type { ImageSummary } from '../../types'
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
  image: ImageSummary
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

// ── Thumbnail URL cache ──
//
// The original implementation used `reactive(new Map())`, which makes
// Vue proxy every entry. With 100+ thumbnails being loaded in parallel
// this proxy machinery becomes the dominant cost on first paint.
//
// shallowRef<Map> + triggerRef gives us manual control: the Map is
// mutated in place (O(1) per insert) and a single triggerRef fans out
// one re-render to all consumers. The template still works because
// shallowRef.value is auto-unwrapped in render contexts.
const thumbUrls = shallowRef<Map<number, string>>(new Map())

function setThumbUrlsBulk(updates: Map<number, string>) {
  if (updates.size === 0) return
  const m = thumbUrls.value
  for (const [id, url] of updates) m.set(id, url)
  triggerRef(thumbUrls)
}

function removeThumbUrls(ids: Iterable<number>) {
  const m = thumbUrls.value
  let changed = false
  for (const id of ids) {
    if (m.delete(id)) changed = true
  }
  if (changed) triggerRef(thumbUrls)
}

function evictAllThumbUrls() {
  if (thumbUrls.value.size === 0) return
  for (const [id, url] of thumbUrls.value) {
    // Skip URLs the store cache is still holding — the cache will
    // revoke them via its own LRU eviction when its size exceeds
    // MAX_CACHE_SIZE. Revoking here would leave a dangling reference
    // in `thumbStore.cache` and break cached thumbnails on the next
    // workspace visit.
    if (thumbStore.cache.has(id)) continue
    URL.revokeObjectURL(url)
  }
  thumbUrls.value = new Map()
  triggerRef(thumbUrls)
}

/**
 * Priority-based batch thumbnail loading.
 *
 * Key changes from the original:
 *  1. Initial batch size bumped from 20 → 60 to cover the typical
 *     first-viewport (50-100 cells) in one IPC round-trip. The
 *     backend's `get_thumbnails_batch` now uses a single SQL IN
 *     query so this scales linearly without DB lock contention.
 *  2. thumbUrls is shallowRef + triggerRef — no deep proxy overhead
 *     for 100+ concurrent entries.
 *  3. Prefetch zone: we also queue thumbnails one viewport ahead
 *     of the bottom so the user does not see a flash of skeleton
 *     placeholders while scrolling down.
 *  4. Single trigger per batch flush — 60 inserts + 1 triggerRef
 *     instead of 60 individual reactive updates.
 */
const BATCH_CHUNK_SIZE = 60
const PREFETCH_ROWS = 6
let pendingBatch: number[] = []
let batchTimer: ReturnType<typeof setTimeout> | null = null

const FALLBACK_CONCURRENCY = 4
const inflightFallback = new Set<number>()
const fallbackQueue: number[] = []

function processFallbackQueue() {
  while (inflightFallback.size < FALLBACK_CONCURRENCY && fallbackQueue.length > 0) {
    const id = fallbackQueue.shift()!
    if (inflightFallback.has(id) || thumbUrls.value.has(id)) continue
    inflightFallback.add(id)
    thumbStore.load(id).then(url => {
      if (url) {
        thumbUrls.value.set(id, url)
        triggerRef(thumbUrls)
      }
    }).finally(() => {
      inflightFallback.delete(id)
      processFallbackQueue()
    })
  }
}

async function flushBatch() {
  if (pendingBatch.length === 0) return
  const batch = pendingBatch
  pendingBatch = []
  // batchLoad returns only the ids that required a backend round-trip;
  // ids that were already in the cache are filtered out and never appear
  // in `loaded`. We must still surface their URLs to the grid, otherwise
  // cached thumbnails render as placeholders on the *next* workspace
  // visit (after this component instance is recreated).
  await thumbStore.batchLoad(batch)
  const updates = new Map<number, string>()
  for (const id of batch) {
    const url = thumbStore.cache.get(id)
    if (url) updates.set(id, url)
  }
  setThumbUrlsBulk(updates)
}

function scheduleBatchFlush() {
  if (batchTimer) clearTimeout(batchTimer)
  if (pendingBatch.length >= BATCH_CHUNK_SIZE) {
    flushBatch()
  } else {
    batchTimer = setTimeout(() => flushBatch(), 50)
  }
}

let thumbLoadTimer: ReturnType<typeof setTimeout> | null = null
watch(visibleItems, (items) => {
  if (thumbLoadTimer) clearTimeout(thumbLoadTimer)
  thumbLoadTimer = setTimeout(() => {
    const visibleIds = new Set(items.map(vi => vi.image.id))

    // Collect uncached IDs, prioritizing items in the first few rows.
    // The local Map is named `rowGroups` to avoid shadowing the
    // outer `rows` computed (which gives the total row count).
    const rowGroups = new Map<number, number[]>()
    for (const vi of items) {
      const id = vi.image.id
      if (!thumbUrls.value.has(id) && !thumbStore.loading.has(id)) {
        const rowIndex = Math.floor(vi.idx / columns.value)
        if (!rowGroups.has(rowIndex)) rowGroups.set(rowIndex, [])
        rowGroups.get(rowIndex)!.push(id)
      }
    }
    const currentFirstRow = Math.floor(scrollTop.value / (cellHeight.value + GAP))
    const sortedRows = Array.from(rowGroups.entries()).sort(([a], [b]) =>
      Math.abs(a - currentFirstRow) - Math.abs(b - currentFirstRow)
    )
    const uncached: number[] = []
    for (const [, ids] of sortedRows) uncached.push(...ids)

    // First batch goes immediately; remainder is processed in
    // smaller concurrent fallback (4 at a time, increased from 2
    // since each individual request is now cheap).
    const priorityCount = Math.min(uncached.length, BATCH_CHUNK_SIZE)
    pendingBatch.push(...uncached.slice(0, priorityCount))
    scheduleBatchFlush()
    for (const id of uncached.slice(priorityCount)) {
      if (!fallbackQueue.includes(id) && !inflightFallback.has(id)) {
        fallbackQueue.push(id)
      }
    }
    processFallbackQueue()

    // Prefetch thumbnails one viewport-row ahead of the bottom so
    // scrolling does not surface empty skeleton cells.
    const prefetchEndRow = Math.min(
      rows.value,
      endRow.value + PREFETCH_ROWS
    )
    if (prefetchEndRow > endRow.value) {
      for (let r = endRow.value; r < prefetchEndRow; r++) {
        for (let c = 0; c < columns.value; c++) {
          const idx = r * columns.value + c
          if (idx >= imageStore.images.length) break
          const id = imageStore.images[idx].id
          if (!thumbUrls.value.has(id) && !thumbStore.loading.has(id) && !fallbackQueue.includes(id) && !inflightFallback.has(id)) {
            fallbackQueue.push(id)
          }
        }
      }
      processFallbackQueue()
    }

    // Evict URLs for items that scrolled far out of view (keep 200 cells of buffer)
    if (thumbUrls.value.size > visibleIds.size + 200) {
      const toEvict: number[] = []
      for (const [id] of thumbUrls.value) {
        if (!visibleIds.has(id)) toEvict.push(id)
      }
      if (toEvict.length > 0) {
        // Evict in one pass: bulk remove from grid cache, then revoke URLs.
        for (const id of toEvict) thumbStore.revoke(id)
        removeThumbUrls(toEvict)
      }
    }
  }, 80)
}, { immediate: true })

// Track the highest page we have already requested so the pre-load
// path is not re-triggered on every scroll tick while the request
// is in flight.
let highestRequestedPage = 1
function onScroll(e: Event) {
  const el = e.target as HTMLElement
  scrollTop.value = el.scrollTop
  updateScrollInfo()

  // Pre-load next page when 65% scrolled. The previous threshold
  // (within 500px of bottom) was too late: by the time the user hit
  // the trigger the IPC + render of the next page caused a visible
  // stutter on 100+ image workspaces. Pre-loading at 65% gives the
  // next page time to land before the user reaches the bottom.
  if (imageStore.images.length < imageStore.total && !imageStore.loading) {
    const scrolledPct = (el.scrollTop + el.clientHeight) / el.scrollHeight
    if (scrolledPct > 0.65 || el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {
      const nextPage = imageStore.page + 1
      if (nextPage > highestRequestedPage) {
        highestRequestedPage = nextPage
        imageStore.setPage(nextPage)
        imageStore.fetchImages(props.workspaceId, true)
      }
    }
  }
}

function handleClick(image: ImageSummary) {
  if (props.selectMode) {
    selection.toggle(image.id)
  } else {
    emit('selectDetail', image.id)
  }
}

function handleDoubleClick(image: ImageSummary) {
  if (props.selectMode) return
  const idx = imageStore.images.findIndex(i => i.id === image.id)
  if (idx >= 0) emit('viewFullscreen', idx)
}

// Hover-revealed expand button: an explicit affordance for opening the
// fullscreen viewer that works in both normal and select modes (unlike
// double-click, which is suppressed in select mode so the user can chain
// single-click selects without accidentally zooming in). @click.stop is
// required so clicking the button does not also trigger the cell's
// normal single-click handler (toggle selection / select detail).
function handleExpandClick(image: ImageSummary, e: MouseEvent) {
  e.stopPropagation()
  const idx = imageStore.images.findIndex(i => i.id === image.id)
  if (idx >= 0) emit('viewFullscreen', idx)
}

// Context menu
const contextMenu = reactive({ visible: false, x: 0, y: 0, image: null as ImageSummary | null })
const showWorkspacePicker = ref(false)

function handleContextMenu(image: ImageSummary, e: MouseEvent) {
  e.preventDefault()
  if (!selection.selectedIds.has(image.id)) {
    selection.clear()
    selection.toggle(image.id)
  }
  contextMenu.visible = true
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.image = image
}

function closeContextMenu() {
  contextMenu.visible = false
}

function getAbsolutePath(image: ImageSummary): string {
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
      updateScrollInfo()
    })
    resizeObserver.observe(containerRef.value)
    updateScrollInfo()
  }
})
onUnmounted(() => {
  resizeObserver?.disconnect()
  // Free blob URLs so they don't leak across workspace switches.
  evictAllThumbUrls()
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
  flashId.value = null
  await nextTick()
  flashId.value = imageId
  setTimeout(() => { flashId.value = null }, 1200)
}

// ── Floating scroll FAB state ──
//
// One reactive bag updated only inside onScroll / ResizeObserver /
// totalHeight watcher — reading scrollHeight and clientHeight every
// frame is cheap, but we don't want each call to fan out as a separate
// ref change. The threshold (300px) matches the typical first-screen
// height so the buttons stay tucked out of the way while the user is
// reading the top of the grid.
const SCROLL_FAB_THRESHOLD = 300
const scrollInfo = ref({
  canScroll: false,
  scrollTop: 0,
  distanceFromBottom: 0,
})

function updateScrollInfo() {
  const el = containerRef.value
  if (!el) return
  const canScroll = el.scrollHeight > el.clientHeight + SCROLL_FAB_THRESHOLD
  scrollInfo.value = {
    canScroll,
    scrollTop: el.scrollTop,
    distanceFromBottom: el.scrollHeight - el.scrollTop - el.clientHeight,
  }
}

function scrollToTop() {
  containerRef.value?.scrollTo({ top: 0, behavior: 'smooth' })
}

function scrollToBottom() {
  const el = containerRef.value
  if (!el) return
  el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' })
}

// Reset scroll-pagination state on workspace switch so the next
// pre-load pass starts from page 1.
watch(() => props.workspaceId, () => {
  highestRequestedPage = 1
  pendingBatch = []
  fallbackQueue.length = 0
  inflightFallback.clear()
  if (batchTimer) { clearTimeout(batchTimer); batchTimer = null }
  if (thumbLoadTimer) { clearTimeout(thumbLoadTimer); thumbLoadTimer = null }
  evictAllThumbUrls()
  scrollTop.value = 0
  nextTick(updateScrollInfo)
})

// Recompute FAB visibility when total content height changes
// (pagination appends, items removed, thumb-size adjusted, etc.).
watch(totalHeight, () => nextTick(updateScrollInfo))

watch(() => imageStore.selectedDetail, async (detail, prev) => {
  if (detail) {
    await nextTick()
    await new Promise(r => setTimeout(r, 80))
    scrollToImage(detail.id)
  } else if (!detail && prev) {
    const targetId = imageStore.lastSelectedId
    if (targetId == null) return
    locateAndFlash(targetId)
  }
})

defineExpose({ scrollToImage, locateAndFlash })
</script>

<template>
  <div class="thumb-grid-outer">
    <div ref="containerRef" class="thumb-grid" @scroll="onScroll" @click="closeContextMenu" @contextmenu.prevent>
    <div class="grid-sizer" :style="{ height: `${totalHeight}px` }">
      <div
        v-for="vi in visibleItems"
        :key="vi.image.id"
        class="grid-cell"
        :style="vi.style"
        :class="{ selected: selectMode && selection.selectedIds.has(vi.image.id), 'detail-active': detailImageId === vi.image.id, 'flash-locate': flashId === vi.image.id }"
        @click="handleClick(vi.image)"
        @dblclick="handleDoubleClick(vi.image)"
        @contextmenu.prevent="handleContextMenu(vi.image, $event)"
      >
        <div class="thumb-img">
          <img
            v-if="thumbUrls.get(vi.image.id)"
            :src="thumbUrls.get(vi.image.id)"
            :alt="vi.image.file_name"
            loading="lazy"
            decoding="async"
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
        <button
          class="expand-btn"
          :title="t('browser.viewFullscreen')"
          :aria-label="t('browser.viewFullscreen')"
          @click="handleExpandClick(vi.image, $event)"
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
            <path d="M2 6V2h4" />
            <path d="M14 6V2h-4" />
            <path d="M2 10v4h4" />
            <path d="M14 10v4h-4" />
          </svg>
        </button>
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

    <!-- Floating scroll FABs. Positioned absolute inside
         .thumb-grid-outer (which does NOT scroll), so the
         buttons stay pinned to the visible viewport area
         regardless of .thumb-grid content scroll. -->
    <div v-show="scrollInfo.canScroll" class="scroll-fab" aria-label="Scroll navigation">
      <button
        class="scroll-fab-btn"
        :class="{ visible: scrollInfo.scrollTop > SCROLL_FAB_THRESHOLD }"
        :title="t('browser.scrollToTop')"
        :aria-label="t('browser.scrollToTop')"
        @click="scrollToTop"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 13V3" />
          <path d="M3 8l5-5 5 5" />
        </svg>
      </button>
      <button
        class="scroll-fab-btn"
        :class="{ visible: scrollInfo.distanceFromBottom > SCROLL_FAB_THRESHOLD }"
        :title="t('browser.scrollToBottom')"
        :aria-label="t('browser.scrollToBottom')"
        @click="scrollToBottom"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M8 3v10" />
          <path d="M3 8l5 5 5-5" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
/* ── Scroll-fab wrapper: takes flex space, anchors absolute FAB ── */
.thumb-grid-outer {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.thumb-grid {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 12px;
}

/* ── Floating scroll FABs ──
   Positioned absolute inside .thumb-grid-outer (which does NOT
   scroll), so the buttons stay pinned to the visible viewport
   area regardless of .thumb-grid content scroll. */

.grid-sizer {
  position: relative;
  width: 100%;
}

.scroll-fab {
  position: absolute;
  right: 30px;   /* 18px + 12px (.thumb-grid right padding) */
  bottom: 30px;  /* 22px +  8px (.thumb-grid bottom padding) */
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 5;
}
.scroll-fab-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  box-shadow: var(--shadow-md);
  opacity: 0;
  transform: translateY(8px) scale(0.85);
  pointer-events: none;
  transition: opacity 0.18s ease, transform 0.22s var(--ease-spring),
              background 0.15s ease, color 0.15s ease, border-color 0.15s ease,
              box-shadow 0.15s ease;
}
.scroll-fab-btn.visible {
  opacity: 0.92;
  transform: translateY(0) scale(1);
  pointer-events: auto;
}
.scroll-fab-btn.visible:hover {
  opacity: 1;
  background: var(--bg-secondary);
  color: var(--accent);
  border-color: var(--accent);
  box-shadow: var(--shadow-md), 0 0 12px var(--accent-glow);
  transform: translateY(-2px) scale(1.05);
}
.scroll-fab-btn.visible:active {
  transform: scale(0.92);
  transition: transform 0.1s ease;
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
  contain: layout style;
  will-change: transform;
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

/* Hover-revealed expand button in the top-left corner of each
  thumbnail cell. Mirrors the look-and-feel of .scroll-fab-btn so
  the two floating-button systems feel like the same family, but is
  smaller (26px vs 36px) to fit inside a thumbnail and uses z-index
  3 so it stacks above .check-mark (which is top-right, z-index 2).
  Default state is opacity 0 + pointer-events none so it does not
  intercept clicks when the cell is not being hovered. */
.expand-btn {
  position: absolute;
  top: 6px;
  left: 6px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  box-shadow: var(--shadow-sm);
  opacity: 0;
  transform: scale(0.85);
  pointer-events: none;
  z-index: 3;
  transition: opacity 0.18s ease, transform 0.22s var(--ease-spring),
              background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
}
.grid-cell:hover .expand-btn {
  opacity: 0.95;
  transform: scale(1);
  pointer-events: auto;
}
.expand-btn:hover {
  opacity: 1;
  background: var(--bg-secondary);
  color: var(--accent);
  border-color: var(--accent);
  transform: scale(1.05);
}
.expand-btn:active {
  transform: scale(0.92);
  transition: transform 0.1s ease;
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
