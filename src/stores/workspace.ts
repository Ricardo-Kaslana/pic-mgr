import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Workspace, ScanProgress } from '../types'

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentId = ref<number | null>(null)
  const scanProgress = ref<ScanProgress | null>(null)

  // The list of workspaces is small (typically < 20) and rarely changes,
  // so we cache it after the first load and skip the IPC round-trip
  // on every subsequent navigation. This was previously a blocking
  // `await` in BrowserView.onMounted that fired on every workspace
  // switch.
  const loaded = ref(false)
  const loading = ref(false)

  const current = computed(() =>
    workspaces.value.find(w => w.id === currentId.value) ?? null
  )

  async function fetchAll(force = false) {
    if (loading.value) return // single-flight: ignore concurrent calls
    if (loaded.value && !force) return
    loading.value = true
    try {
      workspaces.value = await invoke<Workspace[]>('list_workspaces')
      loaded.value = true
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, path: string) {
    const ws = await invoke<Workspace>('create_workspace', { name, path })
    // Update local cache instead of a full refetch.
    const next = [ws, ...workspaces.value.filter(w => w.id !== ws.id)]
    next.sort((a, b) => (a.updated_at < b.updated_at ? 1 : -1))
    workspaces.value = next
    return ws
  }

  async function remove(id: number) {
    await invoke('delete_workspace', { id })
    workspaces.value = workspaces.value.filter(w => w.id !== id)
    if (currentId.value === id) currentId.value = null
  }

  // Single scan entry point. Always subscribes to scan-progress so
 // callers (initial scan after create, or rescan from the home
 // page) get incremental updates regardless of which UI element
 // is currently showing scanProgress. The previous split into
 // scan (no listener, broken for rescan) and startScan (with
 // listener) was redundant and let rescan never report progress.
  async function scan(id: number) {
    scanProgress.value = null

    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload
    })

    try {
      const progress = await invoke<ScanProgress>('scan_workspace', { id })
      scanProgress.value = progress
      return progress
    } finally {
      unlisten()
    }
  }

  function select(id: number) {
    currentId.value = id
  }

  function invalidate() {
    loaded.value = false
  }

  return { workspaces, currentId, current, scanProgress, loaded, loading, fetchAll, create, remove, scan, select, invalidate }
})