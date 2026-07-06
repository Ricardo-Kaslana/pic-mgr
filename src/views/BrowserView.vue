<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../stores/workspace'
import { useImageStore } from '../stores/images'
import { useSelectionStore } from '../stores/selection'
import { useTagStore } from '../stores/tags'
import { useToast } from '../composables/useToast'
import SidebarPanel from '../components/browser/SidebarPanel.vue'
import GridToolbar from '../components/browser/GridToolbar.vue'
import ThumbnailGrid from '../components/browser/ThumbnailGrid.vue'
import DetailPanel from '../components/browser/DetailPanel.vue'
import DedupView from './DedupView.vue'
import LightboxViewer from '../components/common/LightboxViewer.vue'

const route = useRoute()
const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const imageStore = useImageStore()
const selection = useSelectionStore()
const tagStore = useTagStore()
const { toastMessage, toastVisible, showToast } = useToast()

const workspaceId = computed(() => Number(route.params.id))
const selectMode = ref(false)
const showDedup = ref(false)
const gridRef = ref<InstanceType<typeof ThumbnailGrid> | null>(null)
const sidebarCollapsed = ref(false)
const detailCollapsed = ref(false)

watch(() => imageStore.selectedDetail, (detail) => {
  if (detail) detailCollapsed.value = false
})

const lightboxIndex = ref(0)
const lightboxVisible = ref(false)

function locateImage() {
  const targetId = imageStore.selectedDetail?.id ?? imageStore.lastSelectedId
  if (targetId != null) {
    gridRef.value?.locateAndFlash(targetId)
  }
}

function openFullscreen(index: number) {
  lightboxIndex.value = index
  lightboxVisible.value = true
}

onMounted(async () => {
  try {
    imageStore.reset()
    selection.clear()
    await workspaceStore.fetchAll()
    workspaceStore.select(workspaceId.value)
    await imageStore.fetchImages(workspaceId.value)
    await tagStore.fetchTags(workspaceId.value)
  } catch (e) {
    console.error('[BrowserView] init failed:', e)
  }
})

// Keyboard shortcuts
function onKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return

  switch (e.key) {
    case 'Escape':
      if (selectMode.value) {
        selectMode.value = false
        selection.clear()
      } else {
        imageStore.selectedDetail = null
      }
      break
    case 'a':
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault()
        if (selectMode.value) {
          if (imageStore.images.length > 0 && selection.count === imageStore.images.length) {
            selection.clear()
          } else {
            selection.selectAll(imageStore.images.map(i => i.id))
          }
        }
      }
      break
  }
}

onMounted(() => window.addEventListener('keydown', onKeyDown))
onUnmounted(() => window.removeEventListener('keydown', onKeyDown))
</script>

<template>
  <div class="browser">
    <template v-if="showDedup">
      <DedupView :workspace-id="workspaceId" @close="showDedup = false" />
    </template>
    <template v-else>
      <div class="sidebar-wrapper" :class="{ collapsed: sidebarCollapsed }">
        <div class="panel-content" v-show="!sidebarCollapsed">
          <SidebarPanel :workspace-id="workspaceId" />
        </div>
      </div>
      <button
        class="edge-toggle sidebar-edge"
        :class="{ 'at-edge': sidebarCollapsed }"
        :title="sidebarCollapsed ? t('browser.expandSidebar') : t('browser.collapseSidebar')"
        @click="sidebarCollapsed = !sidebarCollapsed"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path v-if="sidebarCollapsed" d="M4 2l4 4-4 4" />
          <path v-else d="M8 2L4 6l4 4" />
        </svg>
      </button>
      <div class="grid-area">
        <GridToolbar
          v-model:select-mode="selectMode"
          :workspace-id="workspaceId"
          @show-toast="showToast"
          @show-dedup="showDedup = true"
          @locate-last="locateImage"
        />
        <ThumbnailGrid
          ref="gridRef"
          :workspace-id="workspaceId"
          :select-mode="selectMode"
          @select-detail="(id: number) => imageStore.fetchDetail(id)"
          @refresh="() => { imageStore.setPage(1); imageStore.fetchImages(workspaceId) }"
          @view-fullscreen="openFullscreen"
          @show-toast="showToast"
        />
      </div>
      <button
        v-if="imageStore.selectedDetail"
        class="edge-toggle detail-edge"
        :class="{ 'at-edge': detailCollapsed }"
        :title="detailCollapsed ? t('browser.expandDetail') : t('browser.collapseDetail')"
        @click="detailCollapsed = !detailCollapsed"
      >
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path v-if="detailCollapsed" d="M8 2L4 6l4 4" />
          <path v-else d="M4 2l4 4-4 4" />
        </svg>
      </button>
      <div v-if="imageStore.selectedDetail" class="detail-wrapper" :class="{ collapsed: detailCollapsed }">
        <div class="panel-content" v-show="!detailCollapsed">
          <DetailPanel :workspace-id="workspaceId" @locate="locateImage" />
        </div>
      </div>
    </template>

    <LightboxViewer
      v-if="lightboxVisible"
      :images="imageStore.images"
      :initial-index="lightboxIndex"
      @close="lightboxVisible = false"
    />
  </div>

  <!-- Toast notification -->
  <Teleport to="body">
    <Transition name="toast">
      <div v-if="toastVisible" class="toast" role="alert" aria-live="polite">{{ toastMessage }}</div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.browser {
  display: flex;
  height: 100%;
}
.sidebar-wrapper {
  display: flex;
  width: 256px;
  flex-shrink: 0;
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  position: relative;
}
.sidebar-wrapper.collapsed { width: 0; }
.detail-wrapper {
  display: flex;
  width: 336px;
  flex-shrink: 0;
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  position: relative;
}
.detail-wrapper.collapsed { width: 0; }

/* Panel content fades with collapse */
.panel-content {
  flex: 1;
  min-width: 0;
  height: 100%;
  opacity: 1;
  transition: opacity 0.15s ease;
}
.sidebar-wrapper.collapsed .panel-content,
.detail-wrapper.collapsed .panel-content {
  opacity: 0;
  transition: opacity 0.08s ease;
}

/* ── Edge toggle button ── */
.edge-toggle {
  position: relative;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  flex-shrink: 0;
  align-self: center;
  padding: 0;
  opacity: 0;
  transition: opacity 0.18s ease, transform 0.18s cubic-bezier(0.34, 1.56, 0.64, 1), background 0.15s ease, color 0.15s ease, border-color 0.15s ease, box-shadow 0.15s ease;
}
.sidebar-edge { margin-left: -14px; margin-right: -14px; }
.detail-edge { margin-left: -14px; margin-right: -14px; }
.edge-toggle.at-edge { opacity: 1; }
.browser:hover .edge-toggle:not(.at-edge) { opacity: 1; }
.edge-toggle:hover { opacity: 1; }
.edge-toggle:hover {
  background: var(--bg-secondary);
  color: var(--accent);
  border-color: var(--accent);
  box-shadow: 0 0 12px var(--accent-glow);
  transform: scale(1.08);
}
.edge-toggle:active {
  transform: scale(0.92);
  transition: transform 0.1s ease;
  box-shadow: 0 0 6px var(--accent-glow);
}

.grid-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

:global(.toast) {
  position: fixed;
  bottom: 28px;
  left: 50%;
  transform: translateX(-50%);
  padding: 10px 22px;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  font-size: 13px;
  color: var(--text-primary);
  box-shadow: var(--shadow-md);
  z-index: 2000;
  max-width: 500px;
  word-break: break-all;
}
:global(.toast-enter-active), :global(.toast-leave-active) { transition: all 0.25s var(--ease-spring); }
:global(.toast-enter-from), :global(.toast-leave-to) { opacity: 0; transform: translateX(-50%) translateY(12px); }
</style>
