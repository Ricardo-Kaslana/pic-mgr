<script setup lang="ts">
import { ref } from 'vue'
import { useImageStore } from '../../stores/images'

const props = defineProps<{ workspaceId: number }>()
const imageStore = useImageStore()
const query = ref('')
const debounce = ref<ReturnType<typeof setTimeout> | null>(null)

function onInput() {
  if (debounce.value) clearTimeout(debounce.value)
  debounce.value = setTimeout(() => {
    if (query.value.trim()) {
      imageStore.search(props.workspaceId, query.value.trim())
    } else {
      imageStore.fetchImages(props.workspaceId)
    }
  }, 300)
}
</script>

<template>
  <div class="search-bar">
    <input
      v-model="query"
      placeholder="搜索文件名或备注..."
      @input="onInput"
    />
  </div>
</template>

<style scoped>
.search-bar { margin-bottom: 16px; }
.search-bar input { width: 100%; }
</style>
