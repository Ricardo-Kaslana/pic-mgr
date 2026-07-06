<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useImageStore } from '../../stores/images'
import ImagePreview from '../detail/ImagePreview.vue'
import MetadataEditor from '../detail/MetadataEditor.vue'
import ImageInfo from '../detail/ImageInfo.vue'

const props = defineProps<{ workspaceId: number }>()
const emit = defineEmits<{ locate: [] }>()
const imageStore = useImageStore()
const { t } = useI18n()

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
      <h3>{{ t('detail.title') }}</h3>
      <div class="header-btns">
        <button class="locate-btn" :title="t('detail.locate')" @click="emit('locate')">&#x2316;</button>
        <button class="close-btn" @click="imageStore.selectedDetail = null">×</button>
      </div>
    </div>
    <div class="panel-body">
      <ImagePreview
        :image-id="imageStore.selectedDetail.id"
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
  flex: 1;
  min-width: 0;
  height: 100%;
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
}
.panel-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-light);
}
.panel-header h3 { font-size: 14px; font-weight: 650; letter-spacing: -0.01em; }
.close-btn {
  background: var(--bg-tertiary);
  font-size: 15px;
  color: var(--text-muted);
  padding: 5px 10px;
  border-radius: var(--radius-xs);
  font-weight: 600;
  border: 1px solid transparent;
  transition: all var(--transition-fast);
}
.close-btn:hover { background: var(--bg-elevated); color: var(--text-primary); border-color: var(--border); }
.close-btn:active { transform: scale(0.95); opacity: 0.8; }
.header-btns { display: flex; align-items: center; gap: 6px; }
.locate-btn {
  background: var(--bg-tertiary);
  font-size: 16px;
  color: var(--text-muted);
  padding: 4px 9px;
  border-radius: var(--radius-xs);
  font-weight: 600;
  border: 1px solid transparent;
  transition: all var(--transition-fast);
  line-height: 1;
}
.locate-btn:hover { background: var(--accent-bg); color: var(--accent); border-color: var(--accent); }
.locate-btn:active { transform: scale(0.95); opacity: 0.8; }
.panel-body { flex: 1; overflow-y: auto; padding: 14px 16px; }
</style>
