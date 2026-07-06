<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useImageStore } from '../../stores/images'

const props = defineProps<{ workspaceId: number }>()
const { t } = useI18n()
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
      :placeholder="t('sidebar.search')"
      @input="onInput"
    />
  </div>
</template>

<style scoped>
.search-bar { }
.search-bar input {
  width: 100%;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 9px 12px;
  font-size: 12.5px;
  transition: all var(--transition-fast);
}
.search-bar input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-bg);
}
.search-bar input::placeholder { color: var(--text-muted); font-size: 12px; }
</style>
