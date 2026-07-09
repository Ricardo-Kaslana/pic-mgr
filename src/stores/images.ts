import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useSettingsStore } from './settings'
import type { ImageSummary, ImageDetail, ImagePage, ImageSummaryPage, ImageFilters } from '../types'

/**
 * The grid view uses ImageSummary (~80 bytes/row) rather than the full
 * ImageRecord (~250 bytes/row). For a 100+ image workspace this is the
 * difference between a snappy first paint and a noticeable stall while
 * the IPC layer serializes and ships the extra fields the grid never
 * reads (file_hash, relative_path, notes, last_viewed_at, file_created_at).
 *
 * The full record is still fetched on demand via get_image_detail when
 * the user opens the right-hand detail panel.
 */
export const useImageStore = defineStore('images', () => {
  const settings = useSettingsStore()
  const images = ref<ImageSummary[]>([])
  const total = ref(0)
  const page = ref(1)
  // Default page size bumped from 50 → 100 so 100+ image workspaces
  // (the exact case the user is hitting) render in a single round trip
  // instead of forcing a scroll-triggered second fetch.
  const pageSize = ref(100)
  const loading = ref(false)
  const selectedDetail = ref<ImageDetail | null>(null)
  const lastSelectedId = ref<number | null>(null)

  // sort_by / sort_dir are seeded from the settings store so the user's
  // last sort choice persists across launches. We keep the live values
  // on the filters reactive (rather than reading from settings on every
  // query) so the toolbar / grid can still do fine-grained reactivity
  // over the filter object, and mirror writes back to settings in
  // setSort() so the localStorage-backed ref stays in sync.
  const filters = reactive<ImageFilters>({
    tag_ids: null,
    format: null,
    rating_min: null,
    date_from: null,
    date_to: null,
    sort_by: settings.sortBy,
    sort_dir: settings.sortDir,
  })

  async function fetchImages(workspaceId: number, append = false) {
    loading.value = true
    try {
      // Hot path: use the lightweight projection.
      const result = await invoke<ImageSummaryPage>('get_images_light', {
        workspaceId,
        filters: { ...filters },
        page: page.value,
        pageSize: pageSize.value,
      })
      if (append && page.value > 1) {
        images.value = [...images.value, ...result.images]
      } else {
        images.value = result.images
      }
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  async function fetchDetail(imageId: number) {
    lastSelectedId.value = imageId
    // Full record is fetched on demand; the grid never pays this cost.
    selectedDetail.value = await invoke<ImageDetail>('get_image_detail', { imageId })
  }

  async function search(workspaceId: number, query: string) {
    loading.value = true
    try {
      // FTS search still returns full records because results are usually
      // small and the detail panel may be opened immediately.
      const result = await invoke<ImagePage>('search_images', {
        workspaceId,
        query,
        page: page.value,
        pageSize: pageSize.value,
      })
      images.value = result.images as unknown as ImageSummary[]
      total.value = result.total
    } finally {
      loading.value = false
    }
  }

  function setPage(p: number) { page.value = p }

  function setSort(by: string, dir: string, wsId: number) {
    filters.sort_by = by
    filters.sort_dir = dir
    // Mirror to the persisted settings ref. The settings store has its
    // own watch that writes the new value to localStorage.
    settings.sortBy = by
    settings.sortDir = dir
    page.value = 1
    fetchImages(wsId)
  }

  function reset() {
    images.value = []
    total.value = 0
    page.value = 1
    selectedDetail.value = null
    lastSelectedId.value = null
  }

  return { images, total, page, pageSize, loading, selectedDetail, lastSelectedId, filters, fetchImages, fetchDetail, search, setPage, setSort, reset }
})