<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import { useImageStore } from '../../stores/images'
import SearchBar from './SearchBar.vue'
import TagFilter from './TagFilter.vue'

const props = defineProps<{ workspaceId: number }>()
const { t } = useI18n()


const imageStore = useImageStore()

// Note: previously this component fired its own `tagStore.fetchTags` on
// mount, which raced with BrowserView's own call on the same workspace
// and produced a duplicate IPC round-trip on every workspace open. The
// fetchTags store now has a single-flight guard that coalesces both
// callers into one DB read, so we just rely on that here.

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
    <div class="section-divider"></div>
    <TagFilter />
    <div class="filter-actions">
      <button class="apply-btn" @click="fetchWithFilters">{{ t('sidebar.apply') }}</button>
      <button class="clear-btn" @click="clearFilters">{{ t('sidebar.clear') }}</button>
    </div>
  </div>
</template>

<style scoped>
.sidebar-panel {
  flex: 1;
  min-width: 0;
  height: 100%;
  background: var(--bg-secondary);
  padding: 14px;
  overflow-y: auto;
}
.section-divider {
  height: 1px;
  background: var(--border-light);
  margin: 14px 0;
}
.filter-actions {
  margin-top: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.apply-btn {
  width: 100%;
  padding: 8px;
  font-size: 12.5px;
  font-weight: 600;
  background: var(--accent);
  color: #fff;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}
.apply-btn:hover { background: var(--accent-hover); box-shadow: 0 2px 12px var(--accent-glow); }
.apply-btn:active { opacity: 0.9; transform: scale(0.98); }
.clear-btn {
  width: 100%;
  background: none;
  color: var(--text-muted);
  font-size: 12px;
  padding: 6px;
  border-radius: var(--radius-xs);
  font-weight: 450;
  transition: all var(--transition-fast);
}
.clear-btn:hover { color: var(--text-secondary); background: var(--bg-tertiary); }
.clear-btn:active { opacity: 0.7; }
</style>