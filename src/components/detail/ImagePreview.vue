<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useThumbnailStore } from '../../stores/thumbnail'

const { t } = useI18n()
const props = defineProps<{
  imageId: number
  fileName: string
  thumbnailPath: string | null
}>()

const thumbStore = useThumbnailStore()

const thumbUrl = computed(() => thumbStore.cache.get(props.imageId) || null)
</script>

<template>
  <div class="image-preview">
    <img
      v-if="thumbUrl"
      :src="thumbUrl"
      :alt="fileName"
      class="preview-img"
    />
    <div v-else class="preview-placeholder">
      <span class="icon">🖼️</span>
      <p>{{ fileName }}</p>
      <p class="hint">{{ t('detail.noPreview') }}</p>
    </div>
  </div>
</template>

<style scoped>
.image-preview {
  background: var(--bg-primary);
  border: 1px solid var(--border-light);
  border-radius: var(--radius);
  min-height: 180px;
  max-height: 360px;
  display: flex; align-items: center; justify-content: center;
  margin-bottom: 18px;
  overflow: hidden;
}
.preview-img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.preview-placeholder { text-align: center; color: var(--text-muted); }
.icon { font-size: 48px; display: block; margin-bottom: 8px; opacity: 0.6; }
.hint { font-size: 11px; margin-top: 4px; }
</style>
