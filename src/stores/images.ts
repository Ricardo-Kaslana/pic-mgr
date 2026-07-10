import { defineStore } from 'pinia'
import { ref, reactive, watch } from 'vue'
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
  // Default page size bumped from 50 -> 100 so 100+ image workspaces
  // (the exact case the user is hitting) render in a single round trip
  // instead of forcing a scroll-triggered second fetch.
  const pageSize = ref(100)
  const loading = ref(false)
  const selectedDetail = ref<ImageDetail | null>(null)
  const lastSelectedId = ref<number | null>(null)

  // Track the workspace we most recently fetched for. The auto-refetch
  // watcher below needs to know which workspace to query when filters
  // change — components that mutate filters (TagFilter, sort dropdown,
  // future format/rating/date controls) don't carry the workspaceId.
  const currentWorkspaceId = ref<number | null>(null)

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
    currentWorkspaceId.value = workspaceId
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
    // Search is a fresh result set - start from page 1. Otherwise
    // a user sitting on page 5 of the grid would get page 5 of the
    // search results (usually empty).
    page.value = 1
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

  function setSort(by: string, dir: string, _wsId: number) {
    filters.sort_by = by
    filters.sort_dir = dir
    // Mirror to the persisted settings ref. The settings store has its
    // own watch that writes the new value to localStorage.
    settings.sortBy = by
    settings.sortDir = dir
    // No manual fetchImages(wsId) here - the watcher on `filters`
    // below auto-refetches when sort_by/sort_dir change. The old
    // explicit call would double-fire (one from setSort, one from the
    // watcher) for every sort change.
  }

  function reset() {
    images.value = []
    total.value = 0
    page.value = 1
    selectedDetail.value = null
    lastSelectedId.value = null
    // Note: filters are intentionally NOT reset on workspace switch.
    // Tag IDs are globally unique across workspaces, so a filter like
    // "show only landscape photos" stays meaningful when the user
    // navigates between workspaces. If a filter happens to match zero
    // images in the new workspace, the grid simply shows an empty
    // state - which is the correct behavior.
  }

  // ── Auto-refetch on any filter mutation ──
  // Without this watcher, filter changes (tag chip click, future
  // format/rating/date controls) would mutate `filters` but never
  // re-query the DB - the grid would keep showing the old result set
  // until the user manually triggered a fetch. The previous design
  // required an explicit "Apply Filters" button in the sidebar, which
  // was both easy to miss and a non-standard pattern (every other
  // image manager auto-applies on toggle). This watcher makes the
  // store the single source of truth: any mutation to `filters`
  // triggers a fresh query, regardless of which UI element made it.
  //
  // `flush: 'post'` defers the fetch to after the current reactive
  // batch settles, so a single user action that mutates multiple
  // fields (e.g. clearing all filters) only fires one IPC.
  watch(
    filters,
    () => {
      if (currentWorkspaceId.value == null) return
      page.value = 1
      fetchImages(currentWorkspaceId.value)
    },
    { deep: true, flush: 'post' }
  )

  return {
    images, total, page, pageSize, loading, selectedDetail, lastSelectedId,
    filters, currentWorkspaceId,
    fetchImages, fetchDetail, search, setPage, setSort, reset,
  }
})