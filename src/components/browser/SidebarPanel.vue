<script setup lang="ts">
import { useI18n } from 'vue-i18n'

import { useImageStore } from '../../stores/images'
import SearchBar from './SearchBar.vue'
import TagFilter from './TagFilter.vue'

defineProps<{ workspaceId: number }>()
const { t } = useI18n()


const imageStore = useImageStore()

// Note: previously this component fired its own `tagStore.fetchTags` on
// mount, which raced with BrowserView's own call on the same workspace
// and produced a duplicate IPC round-trip on every workspace open. The
// fetchTags store now has a single-flight guard that coalesces both
// callers into one DB read, so we just rely on that here.

// Filter mutations (tag chip toggle, clear-all) auto-trigger a refetch
// via the watcher in useImageStore, so we just reset the fields here -
// no need to call fetchImages manually.
function clearFilters() {
  imageStore.filters.tag_ids = null
  imageStore.filters.format = null
  imageStore.filters.rating_min = null
  imageStore.filters.date_from = null
  imageStore.filters.date_to = null
}
</script>

<template>
  <div class="sidebar-panel">
    <div class="search-section">
      <SearchBar :workspace-id="workspaceId" />
      <div class="section-divider"></div>
    </div>
    <TagFilter />
    <div class="filter-actions">
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
/* The search section (SearchBar + section-divider) is sized to match the
   brand area's height so its divider line aligns with the brand area's
   border-bottom. Brand area / toolbar height = 71px (expanded) / 55px (collapsed).
   panel padding-top = 14px. So search-section height = brand height - 14. */
.search-section {
  height: 57px; /* 71 - 14, expanded */
  display: flex;
  flex-direction: column;
  margin-bottom: 14px; /* preserve the gap between the divider and TagFilter below */
}
.app-layout.collapsed .search-section {
  height: 41px; /* 55 - 14, collapsed */
}
.search-section .section-divider {
  margin: 0;
  margin-top: auto; /* push the divider to the bottom of the flex container */
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
/* The .apply-btn styles have been removed along with the button itself:
   filter mutations now auto-trigger a refetch via the store watcher, so
   the explicit Apply step was redundant. Kept the .filter-actions wrapper
   so the Clear button stays pinned to the bottom of the sidebar. */
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
