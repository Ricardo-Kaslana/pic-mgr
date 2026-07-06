import { defineStore } from 'pinia'
import { shallowRef, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const MAX_CACHE_SIZE = 500

export interface ThumbnailBatchEntry {
  image_id: number
  bytes: number[]
}

export const useThumbnailStore = defineStore('thumbnail', () => {
  // shallowRef: avoids Vue proxying each Map entry; we mutate in-place and trigger manually
  const cache = shallowRef<Map<number, string>>(new Map())
  const loading = shallowRef<Set<number>>(new Set())
  const accessOrder = ref<number[]>([])

  function touchAccess(imageId: number) {
    const idx = accessOrder.value.indexOf(imageId)
    if (idx !== -1) accessOrder.value.splice(idx, 1)
    accessOrder.value.push(imageId)
  }

  function evictOldest() {
    while (cache.value.size >= MAX_CACHE_SIZE && accessOrder.value.length > 0) {
      const oldest = accessOrder.value.shift()!
      const url = cache.value.get(oldest)
      if (url) {
        URL.revokeObjectURL(url)
        cache.value.delete(oldest)
      }
    }
  }

  function bytesToUrl(bytes: number[]): string {
    const uint8 = new Uint8Array(bytes)
    const blob = new Blob([uint8], { type: 'image/jpeg' })
    return URL.createObjectURL(blob)
  }

  function addToCache(imageId: number, url: string) {
    evictOldest()
    cache.value.set(imageId, url)
    cache.value = new Map(cache.value) // trigger shallowRef update
    touchAccess(imageId)
  }

  async function load(imageId: number) {
    if (cache.value.has(imageId)) {
      touchAccess(imageId)
      return cache.value.get(imageId)!
    }
    if (loading.value.has(imageId)) return null

    loading.value.add(imageId)
    try {
      const bytes = await invoke<number[]>('get_thumbnail', { imageId })
      const url = bytesToUrl(bytes)
      addToCache(imageId, url)
      return url
    } catch {
      return null
    } finally {
      loading.value.delete(imageId)
      loading.value = new Set(loading.value) // trigger shallowRef update
    }
  }

  /**
   * Batch-load thumbnails for multiple IDs in a single IPC call.
   * Returns the set of IDs that were actually loaded (may be fewer than requested
   * if some are already cached or fail to generate).
   */
  async function batchLoad(imageIds: number[]): Promise<Set<number>> {
    const uncached = imageIds.filter(id =>
      !cache.value.has(id) && !loading.value.has(id)
    )
    if (uncached.length === 0) return new Set()

    // Mark all as loading
    for (const id of uncached) loading.value.add(id)
    loading.value = new Set(loading.value)

    try {
      const entries = await invoke<ThumbnailBatchEntry[]>('get_thumbnails_batch', {
        imageIds: uncached,
      })
      const loaded = new Set<number>()
      for (const entry of entries) {
        const url = bytesToUrl(entry.bytes)
        addToCache(entry.image_id, url)
        loaded.add(entry.image_id)
      }
      return loaded
    } catch {
      return new Set()
    } finally {
      for (const id of uncached) loading.value.delete(id)
      loading.value = new Set(loading.value)
    }
  }

  function revoke(imageId: number) {
    const url = cache.value.get(imageId)
    if (url) {
      URL.revokeObjectURL(url)
      cache.value.delete(imageId)
      cache.value = new Map(cache.value)
      const idx = accessOrder.value.indexOf(imageId)
      if (idx !== -1) accessOrder.value.splice(idx, 1)
    }
  }

  function clearAll() {
    for (const url of cache.value.values()) {
      URL.revokeObjectURL(url)
    }
    cache.value = new Map()
    loading.value = new Set()
    accessOrder.value = []
  }

  return { cache, loading, load, batchLoad, revoke, clearAll }
})
