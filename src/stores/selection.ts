import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useSelectionStore = defineStore('selection', () => {
  const selectedIds = ref<Set<number>>(new Set())
  const lastClickedId = ref<number | null>(null)

  const count = computed(() => selectedIds.value.size)
  const hasSelection = computed(() => selectedIds.value.size > 0)
  const all = computed(() => Array.from(selectedIds.value))

  function toggle(id: number, shiftKey: boolean = false) {
    if (shiftKey && lastClickedId.value !== null) {
      // Shift+click: range select (handled by ThumbnailGrid)
    }
    const newSet = new Set(selectedIds.value)
    if (newSet.has(id)) {
      newSet.delete(id)
    } else {
      newSet.add(id)
    }
    selectedIds.value = newSet
    lastClickedId.value = id
  }

  function selectRange(fromId: number, toId: number, allIds: number[]) {
    const from = allIds.indexOf(fromId)
    const to = allIds.indexOf(toId)
    if (from === -1 || to === -1) return
    const [start, end] = from < to ? [from, to] : [to, from]
    const newSet = new Set(selectedIds.value)
    for (let i = start; i <= end; i++) {
      newSet.add(allIds[i])
    }
    selectedIds.value = newSet
  }

  function selectAll(imageIds: number[]) {
    selectedIds.value = new Set(imageIds)
  }

  function clear() {
    selectedIds.value = new Set()
    lastClickedId.value = null
  }

  return { selectedIds, lastClickedId, count, hasSelection, all, toggle, selectRange, selectAll, clear }
})
