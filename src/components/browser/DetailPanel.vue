<script setup lang="ts">
import { useImageStore } from '../../stores/images'
import ImagePreview from '../detail/ImagePreview.vue'
import MetadataEditor from '../detail/MetadataEditor.vue'
import ImageInfo from '../detail/ImageInfo.vue'

const props = defineProps<{ workspaceId: number }>()
const imageStore = useImageStore()

async function handleUpdated() {
  if (imageStore.selectedDetail) {
    await imageStore.fetchDetail(imageStore.selectedDetail.id)
    await imageStore.fetchImages(props.workspaceId)
  }
}
</script>

<template>
  <div class="detail-panel" v-if="imageStore.selectedDetail">
    <div class="panel-header">
      <h3>图片详情</h3>
      <button class="close-btn" @click="imageStore.selectedDetail = null">×</button>
    </div>
    <div class="panel-body">
      <ImagePreview
        :file-name="imageStore.selectedDetail.file_name"
        :thumbnail-path="imageStore.selectedDetail.thumbnail_path"
      />
      <MetadataEditor
        :detail="imageStore.selectedDetail"
        :workspace-id="workspaceId"
        @updated="handleUpdated"
      />
      <ImageInfo :image="imageStore.selectedDetail" />
    </div>
  </div>
</template>

<style scoped>
.detail-panel {
  width: 320px;
  height: 100%;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.panel-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px; border-bottom: 1px solid var(--border);
}
.panel-header h3 { font-size: 14px; }
.close-btn { background: none; font-size: 18px; color: var(--text-muted); padding: 4px 8px; }
.close-btn:hover { background: var(--bg-tertiary); }
.panel-body { flex: 1; overflow-y: auto; padding: 16px; }
</style>
