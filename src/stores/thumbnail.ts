import { defineStore } from 'pinia'
import { shallowRef, triggerRef } from 'vue'
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
  // shallowRef + manual triggerRef —this list is mutated on every
  // thumbnail load (LRU touch + evict). Deep reactivity would
  // proxy every array entry; for the accessOrder list we never
  // need to read individual entries reactively, only fan out
  // once per mutation batch.
  const accessOrder = shallowRef<number[]>([])

  function touchAccess(imageId: number) {
    const arr = accessOrder.value
    const idx = arr.indexOf(imageId)
    if (idx !== -1) arr.splice(idx, 1)
    arr.push(imageId)
    triggerRef(accessOrder)
  }

  function evictOldest() {
    const arr = accessOrder.value
    while (cache.value.size >= MAX_CACHE_SIZE && arr.length > 0) {
      const oldest = arr.shift()!
      const url = cache.value.get(oldest)
      if (url) {
        URL.revokeObjectURL(url)
        cache.value.delete(oldest)
      }
    }
    if (arr.length < accessOrder.value.length) triggerRef(accessOrder)
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
      triggerRef(loading)
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
    triggerRef(loading)

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
      triggerRef(loading)
    }
  }

  function revoke(imageId: number) {
    const url = cache.value.get(imageId)
    if (!url) return
    URL.revokeObjectURL(url)
    cache.value.delete(imageId)
    cache.value = new Map(cache.value)
    const arr = accessOrder.value
    const idx = arr.indexOf(imageId)
    if (idx !== -1) {
      arr.splice(idx, 1)
      triggerRef(accessOrder)
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
