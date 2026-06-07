<script setup lang="ts">
import { computed } from 'vue'
import { useImageStore } from '../../stores/images'
import { useSelectionStore } from '../../stores/selection'
import { useSettingsStore } from '../../stores/settings'
import VirtualScroller from '../common/VirtualScroller.vue'
import ThumbnailCard from './ThumbnailCard.vue'

const props = defineProps<{ workspaceId: number }>()
const emit = defineEmits<{ selectDetail: [id: number] }>()

const imageStore = useImageStore()
const selection = useSelectionStore()
const settings = useSettingsStore()

const itemHeight = computed(() => settings.thumbnailSize + 40)

function handleClick(id: number, shiftKey: boolean) {
  selection.toggle(id, shiftKey)
}

function handleDblClick(id: number) {
  emit('selectDetail', id)
}
</script>

<template>
  <VirtualScroller
    :items="imageStore.images"
    :item-height="itemHeight"
    :gap="12"
    @load-more="imageStore.setPage(imageStore.page + 1); imageStore.fetchImages(workspaceId)"
  >
    <template #item="{ item, index }">
      <div class="grid-cell">
        <ThumbnailCard
          :image="item"
          :selected="selection.selectedIds.has(item.id)"
          @click="handleClick"
          @dblclick="() => handleDblClick(item.id)"
        />
      </div>
    </template>
  </VirtualScroller>
</template>

<style scoped>
.grid-cell { padding: 0 6px; }
</style>
