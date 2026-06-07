import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ImageRecord, ImageDetail, ImagePage, ImageFilters } from '../types'

export const useImageStore = defineStore('images', () => {
  const images = ref<ImageRecord[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(100)
  const loading = ref(false)
  const selectedDetail = ref<ImageDetail | null>(null)

  const filters = reactive<ImageFilters>({
    tag_ids: null,
    format: null,
    rating_min: null,
    date_from: null,
    date_to: null,
    sort_by: 'date',
    sort_dir: 'desc',
  })

  async function fetchImages(workspaceId: number) {
    loading.value = true
    try {
      const result = await invoke<ImagePage>('get_images', {
        workspaceId,
        filters: { ...filters },
        page: page.value,
        pageSize: pageSize.value,
      })
      images.value = result.images
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  async function fetchDetail(imageId: number) {
    selectedDetail.value = await invoke<ImageDetail>('get_image_detail', { imageId })
  }

  async function search(workspaceId: number, query: string) {
    loading.value = true
    try {
      const result = await invoke<ImagePage>('search_images', {
        workspaceId,
        query,
        page: page.value,
        pageSize: pageSize.value,
      })
      images.value = result.images
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  function setPage(p: number) { page.value = p }

  return { images, total, page, pageSize, loading, selectedDetail, filters, fetchImages, fetchDetail, search, setPage }
})
