<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ fileName: string; thumbnailPath: string | null }>()

const displayName = computed(() => props.fileName)
</script>

<template>
  <div class="image-preview">
    <div class="preview-placeholder">
      <span class="icon">🖼️</span>
      <p>{{ displayName }}</p>
      <p class="hint">双击缩略图查看大图</p>
    </div>
  </div>
</template>

<style scoped>
.image-preview {
  background: var(--bg-primary);
  border-radius: var(--radius);
  min-height: 200px;
  display: flex; align-items: center; justify-content: center;
  margin-bottom: 16px;
}
.preview-placeholder { text-align: center; color: var(--text-muted); }
.icon { font-size: 48px; display: block; margin-bottom: 8px; }
.hint { font-size: 11px; margin-top: 4px; }
</style>
