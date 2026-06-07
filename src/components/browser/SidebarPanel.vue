<script setup lang="ts">
import { onMounted } from 'vue'
import { useTagStore } from '../../stores/tags'
import { useImageStore } from '../../stores/images'
import SearchBar from './SearchBar.vue'
import TagFilter from './TagFilter.vue'

const props = defineProps<{ workspaceId: number }>()

const tagStore = useTagStore()
const imageStore = useImageStore()

onMounted(() => {
  tagStore.fetchTags(props.workspaceId)
})

function fetchWithFilters() {
  imageStore.setPage(1)
  imageStore.fetchImages(props.workspaceId)
}

function clearFilters() {
  imageStore.filters.tag_ids = null
  imageStore.filters.format = null
  imageStore.filters.rating_min = null
  imageStore.filters.date_from = null
  imageStore.filters.date_to = null
  imageStore.setPage(1)
  imageStore.fetchImages(props.workspaceId)
}
</script>

<template>
  <div class="sidebar-panel">
    <SearchBar :workspace-id="workspaceId" />
    <TagFilter />
    <button class="secondary" style="width:100%" @click="fetchWithFilters">应用筛选</button>
    <button class="clear-link" @click="clearFilters">清除所有筛选</button>
  </div>
</template>

<style scoped>
.sidebar-panel {
  width: 240px;
  height: 100%;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  padding: 16px;
  overflow-y: auto;
  flex-shrink: 0;
}
.clear-link {
  display: block; width: 100%; margin-top: 8px;
  background: none; color: var(--text-muted);
  font-size: 12px; padding: 4px;
}
.clear-link:hover { color: var(--text-secondary); }
</style>
