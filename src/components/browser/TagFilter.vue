<script setup lang="ts">
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
        @click="toggleTag(tag.id)"
      >
        <span class="tag-dot" :style="tag.color ? { background: tag.color } : {}"></span>
        {{ tag.name }}
        <span class="count">{{ tag.image_count }}</span>
      </button>
    </div>
    <p v-if="tagStore.tags.length === 0" class="empty">{{ t('sidebar.noTags') }}</p>
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
