import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Tag, TagWithCount } from '../types'

export const useTagStore = defineStore('tags', () => {
  const tags = ref<TagWithCount[]>([])
  // Global tag catalogue: every tag in the `tags` table regardless of
  // workspace. Used by MetadataEditor's suggestion dropdown and its
  // addTag() dedupe-before-create path. Without this, attaching an
  // existing global tag from a workspace that doesn't currently use
  // it would crash on UNIQUE(name) because the picker only sees the
  // workspace-scoped slice and falls through to create_tag().
  const allTags = ref<Tag[]>([])
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
  // Same single-flight treatment for the global catalogue. The list
  // changes rarely (only on create/delete), so we cache and reload on
  // demand rather than refetching on every workspace open.
  const allTagsLoaded = ref(false)
  let allTagsInflight: Promise<void> | null = null

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

  async function fetchAllTags(force = false) {
    if (allTagsInflight) return allTagsInflight
    if (!force && allTagsLoaded.value && allTags.value.length >= 0) {
      return // Already loaded (empty list is a valid cached state).
    }
    allTagsInflight = (async () => {
      try {
        allTags.value = await invoke<Tag[]>('get_all_tags')
        allTagsLoaded.value = true
      } finally {
        allTagsInflight = null
      }
    })()
    return allTagsInflight
  }

  async function createTag(name: string, color?: string) {
    const tag: Tag = await invoke('create_tag', { name, color })
    // Invalidate both caches so the next reads reload:
    //  - workspace-scoped: count may now include this tag in the workspace
    //    if it was just used
    //  - global: the new tag itself must appear in the catalogue
    loadedWorkspaceId.value = null
    allTagsLoaded.value = false
    return tag
  }

  async function deleteTag(tagId: number) {
    await invoke('delete_tag', { tagId })
    // Drop the tag from both the workspace-scoped list and the global
    // catalogue (a global tag delete cascades through image_tags so the
    // workspace count for this tag is now 0 here even if other
    // workspaces still reference it — caller must refetch to refresh).
    tags.value = tags.value.filter(t => t.id !== tagId)
    allTags.value = allTags.value.filter(t => t.id !== tagId)
    // Force refetch on the next read: the previous (buggy) implementation
    // forgot this line, so the next fetchTags() call would short-circuit
    // on the loadedWorkspaceId cache and return stale data with the
    // deleted tag still in it.
    loadedWorkspaceId.value = null
  }

  async function batchUpdate(imageIds: number[], tagIds: number[], action: string) {
    await invoke('batch_update_tags', { imageIds, tagIds, action })
    // Tag counts may have changed; mark stale so the next fetchTags reloads.
    loadedWorkspaceId.value = null
  }

  function reset() {
    tags.value = []
    loadedWorkspaceId.value = null
    // Also drop the global catalogue on full reset — it's cheap to
    // refetch and avoids showing stale definitions in the next session
    // after the user signs out / switches identity (future feature).
    allTags.value = []
    allTagsLoaded.value = false
  }

  return { tags, allTags, loading, fetchTags, fetchAllTags, createTag, deleteTag, batchUpdate, reset }
})