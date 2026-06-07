<script setup lang="ts">
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useTagStore } from '../../stores/tags'
import type { ImageDetail } from '../../types'
import TagBadge from '../common/TagBadge.vue'

const props = defineProps<{ detail: ImageDetail; workspaceId: number }>()
const emit = defineEmits<{ updated: [] }>()

const tagStore = useTagStore()
const notes = ref(props.detail.notes ?? '')
const rating = ref(props.detail.rating ?? 0)
const saving = ref(false)

watch(() => props.detail.id, () => {
  notes.value = props.detail.notes ?? ''
  rating.value = props.detail.rating ?? 0
})

async function saveNotes() {
  saving.value = true
  try {
    await invoke('update_notes', { imageId: props.detail.id, notes: notes.value })
    emit('updated')
  } finally { saving.value = false }
}

async function setRating(r: number) {
  rating.value = r
  await invoke('update_rating', { imageId: props.detail.id, rating: r })
  emit('updated')
}

async function removeTag(tagId: number) {
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'remove',
  })
  emit('updated')
}

const newTagName = ref('')
async function addTag() {
  if (!newTagName.value.trim()) return
  const existing = tagStore.tags.find(t => t.name === newTagName.value.trim())
  let tagId = existing?.id
  if (!tagId) {
    const tag = await tagStore.createTag(newTagName.value.trim())
    tagId = tag.id
    await tagStore.fetchTags(props.workspaceId)
  }
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'add',
  })
  newTagName.value = ''
  emit('updated')
}
</script>

<template>
  <div class="metadata-editor">
    <!-- Rating -->
    <div class="field">
      <label>评分</label>
      <div class="stars">
        <button
          v-for="n in 5" :key="n"
          class="star" :class="{ active: n <= rating }"
          @click="setRating(n)"
        >{{ n <= rating ? '★' : '☆' }}</button>
      </div>
    </div>

    <!-- Tags -->
    <div class="field">
      <label>标签</label>
      <div class="tags-row">
        <TagBadge
          v-for="tag in detail.tags"
          :key="tag.id"
          :name="tag.name"
          :color="tag.color"
          @click="removeTag(tag.id)"
          title="点击移除"
        />
      </div>
      <div class="add-tag-row">
        <input
          v-model="newTagName"
          placeholder="添加标签..."
          @keyup.enter="addTag"
          style="flex:1"
        />
        <button class="primary" @click="addTag" style="padding:8px 12px">+</button>
      </div>
    </div>

    <!-- Notes -->
    <div class="field">
      <label>备注</label>
      <textarea
        v-model="notes"
        rows="4"
        placeholder="添加备注..."
        @blur="saveNotes"
      ></textarea>
    </div>

    <p v-if="saving" class="saving">保存中...</p>
  </div>
</template>

<style scoped>
.metadata-editor { padding: 8px 0; }
.field { margin-bottom: 20px; }
.field label { display: block; margin-bottom: 8px; font-size: 12px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; }
.stars { display: flex; gap: 4px; }
.star { background: none; font-size: 24px; padding: 0; color: var(--text-muted); }
.star.active { color: var(--warning); }
.tags-row { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.add-tag-row { display: flex; gap: 8px; }
textarea { width: 100%; resize: vertical; min-height: 80px; }
.saving { font-size: 12px; color: var(--accent); }
</style>
