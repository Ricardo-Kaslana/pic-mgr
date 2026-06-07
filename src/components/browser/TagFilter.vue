<script setup lang="ts">
import { useTagStore } from '../../stores/tags'
import { useImageStore } from '../../stores/images'

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
      <span>标签</span>
      <button v-if="imageStore.filters.tag_ids?.length" class="clear-btn" @click="clearTags">清除</button>
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
    <p v-if="tagStore.tags.length === 0" class="empty">暂无标签</p>
  </div>
</template>

<style scoped>
.tag-filter { margin-bottom: 16px; }
.filter-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; }
.filter-header span { font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.clear-btn { background: none; color: var(--accent); padding: 2px 8px; font-size: 11px; }
.tag-list { display: flex; flex-wrap: wrap; gap: 6px; }
.tag-chip {
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  padding: 4px 10px;
  border-radius: 100px;
  font-size: 12px;
  display: flex; align-items: center; gap: 6px;
}
.tag-chip:hover { color: var(--text-primary); }
.tag-chip.active { background: var(--accent); color: white; }
.tag-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--accent); }
.count { font-size: 10px; color: var(--text-muted); }
.tag-chip.active .count { color: rgba(255,255,255,0.7); }
.empty { font-size: 12px; color: var(--text-muted); }
</style>
