import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Tag, TagWithCount } from '../types'

export const useTagStore = defineStore('tags', () => {
  const tags = ref<TagWithCount[]>([])
  const loading = ref(false)

  async function fetchTags(workspaceId: number) {
    loading.value = true
    try {
      tags.value = await invoke<TagWithCount[]>('get_tags', { workspaceId })
    } finally {
      loading.value = false
    }
  }

  async function createTag(name: string, color?: string) {
    const tag: Tag = await invoke('create_tag', { name, color })
    return tag
  }

  async function deleteTag(tagId: number) {
    await invoke('delete_tag', { tagId })
  }

  async function batchUpdate(imageIds: number[], tagIds: number[], action: string) {
    await invoke('batch_update_tags', { imageIds, tagIds, action })
  }

  return { tags, loading, fetchTags, createTag, deleteTag, batchUpdate }
})
