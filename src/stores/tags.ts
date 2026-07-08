import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Tag, TagWithCount } from '../types'

export const useTagStore = defineStore('tags', () => {
  const tags = ref<TagWithCount[]>([])
  const loading = ref(false)
  // Track the last workspace we loaded tags for. Multiple components
  // (BrowserView + SidebarPanel) used to fire fetchTags on mount, and
  // the duplicate IPC call was visible as a ~50ms stall on every
  // workspace open. The guard coalesces them into one call.
  const loadedWorkspaceId = ref<number | null>(null)
  // Inflight promise so concurrent callers (e.g. SidebarPanel mounted
  // while BrowserView's await is still pending) get the same result
  // instead of issuing a second IPC.
  let inflight: Promise<void> | null = null

  async function fetchTags(workspaceId: number, force = false) {
    if (loading.value && inflight) return inflight
    if (!force && loadedWorkspaceId.value === workspaceId && tags.value.length > 0) {
      return // Already loaded for this workspace
    }
    loadedWorkspaceId.value = workspaceId
    loading.value = true
    inflight = (async () => {
      try {
        tags.value = await invoke<TagWithCount[]>('get_tags', { workspaceId })
      } finally {
        loading.value = false
        inflight = null
      }
    })()
    return inflight
  }

  async function createTag(name: string, color?: string) {
    const tag: Tag = await invoke('create_tag', { name, color })
    // Invalidate the cache so the next fetchTags reloads.
    loadedWorkspaceId.value = null
    return tag
  }

  async function deleteTag(tagId: number) {
    await invoke('delete_tag', { tagId })
    tags.value = tags.value.filter(t => t.id !== tagId)
  }

  async function batchUpdate(imageIds: number[], tagIds: number[], action: string) {
    await invoke('batch_update_tags', { imageIds, tagIds, action })
    // Tag counts may have changed; mark stale so the next fetchTags reloads.
    loadedWorkspaceId.value = null
  }

  function reset() {
    tags.value = []
    loadedWorkspaceId.value = null
  }

  return { tags, loading, fetchTags, createTag, deleteTag, batchUpdate, reset }
})