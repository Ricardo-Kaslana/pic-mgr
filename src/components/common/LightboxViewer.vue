<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../../stores/workspace'
import type { ImageRecord } from '../../types'

const props = defineProps<{
  images: ImageRecord[]
  initialIndex: number
}>()
const emit = defineEmits<{ close: [] }>()

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const currentIndex = ref(props.initialIndex)
const blobUrl = ref('')
const loading = ref(false)

const currentImage = computed(() => props.images[currentIndex.value])
const hasPrev = computed(() => currentIndex.value > 0)
const hasNext = computed(() => currentIndex.value < props.images.length - 1)

function getAbsPath(img: ImageRecord): string {
  const ws = workspaceStore.workspaces.find(w => w.id === img.workspace_id)
  if (!ws) return ''
  return `${ws.path}\\${img.relative_path}`.replace(/\\\\/g, '\\')
}

async function loadImage() {
  if (!currentImage.value) return
  // Revoke previous blob URL
  if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
  loading.value = true
  try {
    const bytes = await invoke<number[]>('read_file_bytes', { path: getAbsPath(currentImage.value) })
    const mime = getMimeType(currentImage.value.format || 'jpg')
    const buf = new Uint8Array(bytes)
    const blob = new Blob([buf], { type: mime })
    blobUrl.value = URL.createObjectURL(blob)
  } catch (e) {
    console.error('Failed to load image:', e)
    blobUrl.value = ''
  } finally {
    loading.value = false
  }
}

function getMimeType(fmt: string): string {
  const map: Record<string, string> = {
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    png: 'image/png',
    gif: 'image/gif',
    webp: 'image/webp',
    bmp: 'image/bmp',
    tiff: 'image/tiff',
    tif: 'image/tiff',
  }
  return map[fmt.toLowerCase()] || 'image/jpeg'
}

// Load image on mount and on index change
loadImage()
watch(currentIndex, () => loadImage())

function goPrev() {
  if (hasPrev.value) currentIndex.value--
}
function goNext() {
  if (hasNext.value) currentIndex.value++
}
function close() {
  emit('close')
}

function onKeyDown(e: KeyboardEvent) {
  switch (e.key) {
    case 'Escape': close(); break
    case 'ArrowLeft': goPrev(); break
    case 'ArrowRight': goNext(); break
  }
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown)
  if (blobUrl.value) URL.revokeObjectURL(blobUrl.value)
})
</script>

<template>
  <Teleport to="body">
    <div class="lightbox-overlay" role="dialog" aria-modal="true" :aria-label="currentImage?.file_name" @click.self="close">
      <div class="lightbox-toolbar">
        <span class="lightbox-info" v-if="currentImage">
          {{ currentImage.file_name }} ({{ currentIndex + 1 }}/{{ images.length }})
        </span>
        <button class="lightbox-btn" @click="close">✕</button>
      </div>
      <button v-if="hasPrev" class="lightbox-nav prev" @click.stop="goPrev">‹</button>
      <div class="lightbox-content">
        <div v-if="loading" class="lightbox-loading">{{ t('common.loading') }}</div>
        <img v-else-if="blobUrl" :src="blobUrl" :alt="currentImage?.file_name" />
        <div v-else class="lightbox-error">{{ t('lightbox.loadFailed') }}</div>
      </div>
      <button v-if="hasNext" class="lightbox-nav next" @click.stop="goNext">›</button>
    </div>
  </Teleport>
</template>

<style scoped>
.lightbox-overlay {
  position: fixed;
  inset: 0;
  z-index: 5000;
  background: rgba(0, 0, 0, 0.92);
  display: flex;
  align-items: center;
  justify-content: center;
}
.lightbox-toolbar {
  position: absolute;
  top: 0; left: 0; right: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: rgba(0, 0, 0, 0.5);
}
.lightbox-info {
  color: #ccc;
  font-size: 14px;
}
.lightbox-btn {
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.1);
  color: #ccc;
  font-size: 18px;
  cursor: pointer;
  padding: 6px 12px;
  border-radius: var(--radius-xs);
  font-weight: 500;
  transition: all var(--transition-fast);
}
.lightbox-btn:hover { background: rgba(255,255,255,0.15); color: #fff; border-color: rgba(255,255,255,0.2); }
.lightbox-btn:active { opacity: 0.7; }
.lightbox-content {
  display: flex;
  align-items: center;
  justify-content: center;
  max-width: 95vw;
  max-height: 90vh;
}
.lightbox-content img {
  max-width: 95vw;
  max-height: 90vh;
  object-fit: contain;
}
.lightbox-loading, .lightbox-error {
  color: #999;
  font-size: 16px;
}
.lightbox-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  background: rgba(255,255,255,0.06);
  border: 1px solid rgba(255,255,255,0.1);
  color: #ccc;
  font-size: 44px;
  cursor: pointer;
  padding: 14px 8px;
  border-radius: var(--radius-xs);
  transition: all var(--transition-fast);
  z-index: 1;
}
.lightbox-nav:hover { background: rgba(255,255,255,0.2); color: #fff; border-color: rgba(255,255,255,0.25); }
.lightbox-nav:active { opacity: 0.7; transform: translateY(-50%) scale(0.95); }
.lightbox-nav.prev { left: 12px; }
.lightbox-nav.next { right: 12px; }
</style>
