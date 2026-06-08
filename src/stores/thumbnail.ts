import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const MAX_CACHE_SIZE = 500

export const useThumbnailStore = defineStore('thumbnail', () => {
  const cache = ref<Map<number, string>>(new Map())
  const loading = ref<Set<number>>(new Set())
  // Track access order for LRU eviction
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
        const newCache = new Map(cache.value)
        newCache.delete(oldest)
        cache.value = newCache
      }
    }
  }

  async function load(imageId: number) {
    if (cache.value.has(imageId)) {
      touchAccess(imageId)
      return cache.value.get(imageId)!
    }
    if (loading.value.has(imageId)) return null

    loading.value = new Set([...loading.value, imageId])
    try {
      const bytes = await invoke<number[]>('get_thumbnail', { imageId })
      const uint8 = new Uint8Array(bytes)
      const blob = new Blob([uint8], { type: 'image/webp' })
      const url = URL.createObjectURL(blob)

      evictOldest()
      const newCache = new Map(cache.value)
      newCache.set(imageId, url)
      cache.value = newCache
      touchAccess(imageId)
      return url
    } catch {
      return null
    } finally {
      const newLoading = new Set(loading.value)
      newLoading.delete(imageId)
      loading.value = newLoading
    }
  }

  function revoke(imageId: number) {
    const url = cache.value.get(imageId)
    if (url) {
      URL.revokeObjectURL(url)
      const newCache = new Map(cache.value)
      newCache.delete(imageId)
      cache.value = newCache
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

  return { cache, loading, load, revoke, clearAll }
})
