import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useThumbnailStore = defineStore('thumbnail', () => {
  const cache = ref<Map<number, string>>(new Map())
  const loading = ref<Set<number>>(new Set())

  async function load(imageId: number) {
    if (cache.value.has(imageId)) return cache.value.get(imageId)!
    if (loading.value.has(imageId)) return null

    loading.value = new Set([...loading.value, imageId])
    try {
      const bytes = await invoke<number[]>('get_thumbnail', { imageId })
      const uint8 = new Uint8Array(bytes)
      const blob = new Blob([uint8], { type: 'image/webp' })
      const url = URL.createObjectURL(blob)
      const newCache = new Map(cache.value)
      newCache.set(imageId, url)
      cache.value = newCache
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
    }
  }

  return { cache, loading, load, revoke }
})
