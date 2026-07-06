<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useTagStore } from '../../stores/tags'
import type { ImageDetail } from '../../types'
import TagBadge from '../common/TagBadge.vue'

const props = defineProps<{ detail: ImageDetail; workspaceId: number }>()
const emit = defineEmits<{ updated: [] }>()

const { t } = useI18n()
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
  await tagStore.fetchTags(props.workspaceId)
  emit('updated')
}

const newTagName = ref('')
const showSuggestions = ref(false)

// 已有标签中，过滤掉已在图片上的标签，并按输入内容匹配
const suggestions = computed(() => {
  const existingIds = new Set(props.detail.tags.map(t => t.id))
  const query = newTagName.value.trim().toLowerCase()
  return tagStore.tags
    .filter(t => !existingIds.has(t.id))
    .filter(t => !query || t.name.toLowerCase().includes(query))
})

async function addTagById(tagId: number, _tagName: string) {
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'add',
  })
  await tagStore.fetchTags(props.workspaceId)
  newTagName.value = ''
  showSuggestions.value = false
  emit('updated')
}

async function addTag() {
  if (!newTagName.value.trim()) return
  const existing = tagStore.tags.find(t => t.name === newTagName.value.trim())
  let tagId = existing?.id
  if (!tagId) {
    const tag = await tagStore.createTag(newTagName.value.trim())
    tagId = tag.id
  }
  await invoke('batch_update_tags', {
    imageIds: [props.detail.id],
    tagIds: [tagId],
    action: 'add',
  })
  await tagStore.fetchTags(props.workspaceId)
  newTagName.value = ''
  showSuggestions.value = false
  emit('updated')
}

function onInputFocus() {
  showSuggestions.value = true
}

function onInputBlur() {
  // 延迟关闭，让点击建议项的事件先触发
  setTimeout(() => { showSuggestions.value = false }, 150)
}
</script>

<template>
  <div class="metadata-editor">
    <!-- Rating -->
    <div class="field">
      <label>{{ t('detail.rating') }}</label>
      <div class="stars" role="radiogroup" :aria-label="t('detail.rating')">
        <button
          v-for="n in 5" :key="n"
          class="star" :class="{ active: n <= rating }"
          role="radio" :aria-checked="n <= rating" :aria-label="`${n}`"
          @click="setRating(n)"
        >{{ n <= rating ? '★' : '☆' }}</button>
      </div>
    </div>

    <!-- Tags -->
    <div class="field">
      <label>{{ t('detail.tags') }}</label>
      <div class="tags-row">
        <TagBadge
          v-for="tag in detail.tags"
          :key="tag.id"
          :name="tag.name"
          :color="tag.color"
          @click="removeTag(tag.id)"
          :title="t('detail.clickToRemove')"
        />
      </div>
      <div class="add-tag-row">
        <div class="tag-input-wrapper">
          <input
            v-model="newTagName"
            :placeholder="t('detail.addTag')"
            @keyup.enter="addTag"
            @focus="onInputFocus"
            @blur="onInputBlur"
            autocomplete="off"
          />
          <!-- 已有标签建议下拉 -->
          <div v-if="showSuggestions && suggestions.length > 0" class="tag-suggestions">
            <button
              v-for="tag in suggestions"
              :key="tag.id"
              class="suggestion-item"
              @mousedown.prevent="addTagById(tag.id, tag.name)"
            >
              <span
                class="suggestion-dot"
                :style="tag.color ? { background: tag.color } : {}"
              ></span>
              {{ tag.name }}
            </button>
          </div>
        </div>
        <button class="primary" @click="addTag" style="padding:8px 12px">+</button>
      </div>
    </div>

    <!-- Notes -->
    <div class="field">
      <label>{{ t('detail.notes') }}</label>
      <textarea
        v-model="notes"
        rows="4"
        :placeholder="t('detail.notesPlaceholder')"
        @blur="saveNotes"
      ></textarea>
    </div>

    <p v-if="saving" class="saving">{{ t('common.saving') }}</p>
  </div>
</template>

<style scoped>
.metadata-editor { padding: 8px 0; }
.field { margin-bottom: 22px; }
.field label {
  display: block;
  margin-bottom: 8px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
}
.stars { display: flex; gap: 1px; }
.star {
  background: none;
  font-size: 22px;
  padding: 3px 4px;
  color: var(--text-muted);
  transition: all var(--transition-fast);
  border-radius: var(--radius-xs);
  line-height: 1;
}
.star:hover { color: var(--warning); transform: scale(1.15); }
.star:active { transform: scale(0.95); }
.star.active { color: var(--warning); }
.tags-row { display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 8px; }
.add-tag-row { display: flex; gap: 8px; }

.tag-input-wrapper {
  flex: 1;
  position: relative;
}
.tag-input-wrapper input {
  width: 100%;
  box-sizing: border-box;
}

.tag-suggestions {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  z-index: 100;
  max-height: 180px;
  overflow-y: auto;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 7px 12px;
  background: none;
  border: none;
  border-radius: 0;
  text-align: left;
  font-size: 13px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background var(--transition-fast);
}
.suggestion-item:hover {
  background: var(--bg-tertiary);
}
.suggestion-item:first-child { border-radius: var(--radius-sm) var(--radius-sm) 0 0; }
.suggestion-item:last-child { border-radius: 0 0 var(--radius-sm) var(--radius-sm); }
.suggestion-item:only-child { border-radius: var(--radius-sm); }

.suggestion-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--accent);
  flex-shrink: 0;
}

textarea { width: 100%; resize: vertical; min-height: 80px; }
.saving { font-size: 12px; color: var(--accent); font-weight: 500; }
</style>
