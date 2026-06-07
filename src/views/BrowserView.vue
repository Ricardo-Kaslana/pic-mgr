<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useWorkspaceStore } from '../stores/workspace'
import { useImageStore } from '../stores/images'
import { useSelectionStore } from '../stores/selection'
import { useTagStore } from '../stores/tags'
import SidebarPanel from '../components/browser/SidebarPanel.vue'
import ThumbnailGrid from '../components/browser/ThumbnailGrid.vue'
import DetailPanel from '../components/browser/DetailPanel.vue'

const route = useRoute()
const workspaceStore = useWorkspaceStore()
const imageStore = useImageStore()
const selection = useSelectionStore()
const tagStore = useTagStore()

const workspaceId = computed(() => Number(route.params.id))

onMounted(async () => {
  await workspaceStore.fetchAll()
  workspaceStore.select(workspaceId.value)
  await imageStore.fetchImages(workspaceId.value)
  await tagStore.fetchTags(workspaceId.value)
})

// Keyboard shortcuts
function onKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

  switch (e.key) {
    case 'Escape':
      selection.clear()
      imageStore.selectedDetail = null
      break
    case 'a':
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault()
        selection.selectAll(imageStore.images.map(i => i.id))
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
        <span v-if="selection.count > 0" class="selection-info">
          已选 {{ selection.count }} 张
        </span>
      </div>
      <ThumbnailGrid
        :workspace-id="workspaceId"
        @select-detail="(id: number) => imageStore.fetchDetail(id)"
      />
    </div>
    <DetailPanel v-if="imageStore.selectedDetail" :workspace-id="workspaceId" />
  </div>
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
.selection-info { font-size: 12px; color: var(--accent); font-weight: 500; }
</style>
