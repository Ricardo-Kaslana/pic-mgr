<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useThumbnailStore } from '../../stores/thumbnail'
import type { ImageRecord } from '../../types'

const props = defineProps<{ image: ImageRecord; selected: boolean }>()
const emit = defineEmits<{ click: [id: number, shiftKey: boolean]; dblclick: [] }>()

const thumb = useThumbnailStore()
const thumbUrl = ref<string | null>(null)

onMounted(async () => {
  thumbUrl.value = await thumb.load(props.image.id)
})

onUnmounted(() => {
  thumb.revoke(props.image.id)
})
</script>

<template>
  <div
    class="thumb-card"
    :class="{ selected }"
    @click="emit('click', image.id, $event.shiftKey)"
    @dblclick="emit('dblclick')"
  >
    <div class="thumb-img">
      <img v-if="thumbUrl" :src="thumbUrl" :alt="image.file_name" loading="lazy" />
      <div v-else class="placeholder">
        <span>🖼️</span>
      </div>
    </div>
    <div class="thumb-info">
      <p class="file-name">{{ image.file_name }}</p>
      <div v-if="image.rating" class="rating">
        {{ '★'.repeat(image.rating) }}{{ '☆'.repeat(5 - image.rating) }}
      </div>
    </div>
    <div v-if="selected" class="check-mark">✓</div>
  </div>
</template>

<style scoped>
.thumb-card {
  width: 100%;
  height: 100%;
  background: var(--bg-secondary);
  border: 2px solid var(--border);
  border-radius: var(--radius);
  overflow: hidden;
  cursor: pointer;
  transition: border-color var(--transition);
  position: relative;
  display: flex;
  flex-direction: column;
}
.thumb-card:hover { border-color: var(--accent); }
.thumb-card.selected { border-color: var(--accent); background: var(--bg-tertiary); }

.thumb-img {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-primary);
  min-height: 0;
}
.thumb-img img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.placeholder {
  font-size: 32px;
  color: var(--text-muted);
}

.thumb-info {
  padding: 6px 8px;
  border-top: 1px solid var(--border);
}
.file-name {
  font-size: 11px;
  color: var(--text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rating { font-size: 10px; color: var(--warning); margin-top: 2px; }

.check-mark {
  position: absolute; top: 6px; right: 6px;
  width: 22px; height: 22px;
  background: var(--accent); color: white;
  border-radius: 50%;
  display: flex; align-items: center; justify-content: center;
  font-size: 12px; font-weight: 700;
}
</style>
