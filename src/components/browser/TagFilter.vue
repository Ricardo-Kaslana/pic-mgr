<script setup lang="ts">
import { reactive, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTagStore } from '../../stores/tags'
import { useImageStore } from '../../stores/images'

const { t } = useI18n()
const tagStore = useTagStore()
const imageStore = useImageStore()

function toggleTag(tagId: number) {
  const current = imageStore.filters.tag_ids ?? []
  const idx = current.indexOf(tagId)
  if (idx === -1) {
    imageStore.filters.tag_ids = [...current, tagId]
  } else {
    imageStore.filters.tag_ids = current.filter(id => id !== tagId)
  }
}

function clearTags() {
  imageStore.filters.tag_ids = null
}

async function deleteTag(tagId: number, tagName: string, imageCount: number) {
  // Tag deletion is global and cascades through image_tags (FK ON DELETE
  // CASCADE) — every image across every workspace that uses this tag
  // loses it. Show a confirmation with the current workspace's count so
  // the user has at least a local signal of what they're about to do.
  const msg = t('tag.confirmDelete', { name: tagName, count: imageCount })
  if (!confirm(msg)) return
  try {
    await tagStore.deleteTag(tagId)
    // If this tag was currently filtering the grid, drop it from filters
    // so the grid doesn't sit on a dead tag id showing zero results
    // until the user manually clears. The filters watcher then fires
    // fetchImages on its own.
    const current = imageStore.filters.tag_ids ?? []
    if (current.includes(tagId)) {
      imageStore.filters.tag_ids = current.filter(id => id !== tagId)
    }
  } catch (e: any) {
    console.error('[TagFilter] deleteTag failed:', e)
    alert(t('tag.deleteFailed', { error: e?.toString?.() ?? e ?? '' }))
  }
}

// ── Right-click context menu ──────────────────────────────────────
// Replaces the previous hover-revealed × button, which was too easy to
// mis-click while moving the mouse across the sidebar. Right-click is
// intentional, and the explicit menu item makes the destructive action
// unambiguous. The menu is Teleported to body (so it isn't clipped by
// the sidebar's overflow:auto) and uses position:fixed (so it doesn't
// drift if the sidebar scrolls while the menu is open).

type MenuTag = { id: number; name: string; image_count: number }

const contextMenu = reactive<{
  visible: boolean
  x: number
  y: number
  tag: MenuTag | null
}>({ visible: false, x: 0, y: 0, tag: null })

function openContextMenu(e: MouseEvent, tag: MenuTag) {
  e.preventDefault()
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.tag = tag
  contextMenu.visible = true
}

function closeContextMenu() {
  contextMenu.visible = false
  contextMenu.tag = null
}

function handleMenuAction(action: 'delete') {
  const tag = contextMenu.tag
  closeContextMenu()
  if (!tag) return
  if (action === 'delete') {
    void deleteTag(tag.id, tag.name, tag.image_count)
  }
}

function onDocPointerDown(e: MouseEvent) {
  if (!contextMenu.visible) return
  // Clicking inside the menu is handled by its own buttons (we stop
  // propagation on the menu root). Anything outside dismisses it.
  const target = e.target as HTMLElement | null
  if (target?.closest('.tag-context-menu')) return
  closeContextMenu()
}

function onDocKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && contextMenu.visible) closeContextMenu()
}

onMounted(() => {
  document.addEventListener('pointerdown', onDocPointerDown)
  document.addEventListener('keydown', onDocKey)
})
onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocPointerDown)
  document.removeEventListener('keydown', onDocKey)
})
</script>

<template>
  <div class="tag-filter">
    <div class="filter-header">
      <span>{{ t('sidebar.tags') }}</span>
      <button v-if="imageStore.filters.tag_ids?.length" class="clear-btn" @click="clearTags">{{ t('sidebar.clearTags') }}</button>
    </div>
    <div class="tag-list">
      <button
        v-for="tag in tagStore.tags"
        :key="tag.id"
        class="tag-chip"
        :class="{ active: imageStore.filters.tag_ids?.includes(tag.id) }"
        :style="tag.color ? { '--tag-color': tag.color } : {}"
        :title="t('tag.rightClickHint')"
        @click="toggleTag(tag.id)"
        @contextmenu="openContextMenu($event, tag)"
      >
        <span class="tag-dot" :style="tag.color ? { background: tag.color } : {}"></span>
        {{ tag.name }}
        <span class="count">{{ tag.image_count }}</span>
      </button>
    </div>
    <p v-if="tagStore.tags.length === 0" class="empty">{{ t('sidebar.noTags') }}</p>

    <!-- Right-click context menu. Teleported to body so the sidebar's
         overflow:auto scroll container can't clip it; position:fixed
         keys the coordinates to the viewport, not the sidebar. -->
    <Teleport to="body">
      <div
        v-if="contextMenu.visible && contextMenu.tag"
        class="tag-context-menu"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
        @pointerdown.stop
        @contextmenu.stop
        role="menu"
      >
        <button
          class="ctx-item danger"
          role="menuitem"
          @click="handleMenuAction('delete')"
        >
          <span class="ctx-icon">🗑️</span>
          {{ t('tag.delete') }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.tag-filter { }
.filter-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.filter-header span {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.clear-btn {
  background: none;
  color: var(--accent);
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 500;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
}
.clear-btn:hover { background: var(--accent-bg); }
.clear-btn:active { opacity: 0.7; }
.tag-list { display: flex; flex-wrap: wrap; gap: 5px; }
.tag-chip {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  padding: 5px 10px;
  border-radius: 100px;
  font-size: 12px;
  font-weight: 450;
  display: flex; align-items: center; gap: 6px;
  transition: all var(--transition-fast);
  border: 1px solid transparent;
}
.tag-chip:hover { color: var(--text-primary); border-color: var(--border); background: var(--bg-elevated); }
.tag-chip:active { transform: scale(0.97); }
.tag-chip.active { background: var(--accent); color: #fff; border-color: var(--accent); box-shadow: 0 1px 3px rgba(0,0,0,0.15); }
.tag-chip.active:hover { background: var(--accent-hover); }
.tag-dot { width: 7px; height: 7px; border-radius: 50%; background: var(--accent); flex-shrink: 0; }
.count {
  font-size: 10px;
  color: var(--text-muted);
  background: var(--bg-primary);
  padding: 1px 5px;
  border-radius: 100px;
}
.tag-chip.active .count { color: rgba(255,255,255,0.8); background: rgba(255,255,255,0.15); }
.empty { font-size: 12px; color: var(--text-muted); }
</style>

<!-- Context menu styles are unscoped because the menu is Teleported to
     <body>; a scoped style wouldn't reach across the portal. They live
     in their own root selector so they don't collide with other
     components' .ctx-* / .tag-context-menu classes. -->
<style>
.tag-context-menu {
  position: fixed;
  z-index: 5000;
  min-width: 160px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  box-shadow: 0 6px 20px rgba(0, 0, 0, 0.28);
  padding: 4px;
  animation: tag-ctx-in 0.12s ease-out;
}
@keyframes tag-ctx-in {
  from { opacity: 0; transform: scale(0.96); }
  to   { opacity: 1; transform: scale(1); }
}
.tag-context-menu .ctx-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  background: none;
  border: none;
  border-radius: var(--radius-xs);
  text-align: left;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background var(--transition-fast);
}
.tag-context-menu .ctx-item:hover { background: var(--bg-tertiary); }
.tag-context-menu .ctx-item.danger { color: var(--danger, #ef4444); }
.tag-context-menu .ctx-item.danger:hover {
  background: var(--danger-bg, #fef2f2);
}
.tag-context-menu .ctx-icon {
  font-size: 13px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;
}
</style>