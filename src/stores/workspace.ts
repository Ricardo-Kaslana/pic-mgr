import { defineStore } from 'pinia'
import { ref, computed, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Workspace, ScanProgress, SyncResult } from '../types'

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

  // Tracks which workspace ids are mid-sync. UI components read this via
  // isSyncing(id) to disable the rescan button and show a spinner. The set
  // is reactive so consumers can also iterate it directly.
  const syncingIds = reactive(new Set<number>())

  function isSyncing(id: number): boolean {
    return syncingIds.has(id)
  }

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

  // Reconcile-style sync: walks the workspace directory, classifies files
  // against the DB index, and applies kept/modified/added/rebound in one
  // transaction. Files missing from disk come back as `deletedCandidates`
  // for the user to confirm via MissingFilesDialog. Subscribes to
  // scan-progress so BrowserView's existing finished=true handler runs
  // (re-fetching images + tags + kicking off background thumbnail regen).
  async function sync(id: number): Promise<SyncResult> {
    syncingIds.add(id)
    scanProgress.value = null

    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload
    })

    try {
      const result = await invoke<SyncResult>('sync_workspace', { id })
      scanProgress.value = { ...(scanProgress.value ?? {} as ScanProgress), finished: true }
      return result
    } finally {
      unlisten()
      syncingIds.delete(id)
    }
  }

  // Destructive full reset for the "Advanced -> Full reset" submenu.
  // Wraps the existing rescan_workspace command (preserved in commands/scan.rs).
  // The frontend is expected to require the user to confirm twice before
  // calling this; the backend does not double-check.
  async function rescan(id: number): Promise<void> {
    syncingIds.add(id)
    scanProgress.value = null
    const unlisten = await listen<ScanProgress>('scan-progress', (event) => {
      scanProgress.value = event.payload
    })
    try {
      await invoke('rescan_workspace', { id })
    } finally {
      unlisten()
      syncingIds.delete(id)
    }
  }

  // Apply the user-confirmed subset of MissingFilesDialog selections.
  // Returns the number of rows actually deleted (may be less than ids.length
  // if some rows no longer exist).
  async function applyDeletions(ids: number[]): Promise<number> {
    return invoke<number>('apply_sync_deletions', { ids })
  }

  function select(id: number) {
    currentId.value = id
  }

  function invalidate() {
    loaded.value = false
  }

  return {
    workspaces, currentId, current, scanProgress, loaded, loading,
    syncingIds, isSyncing,
    fetchAll, create, remove, sync, rescan, applyDeletions, select, invalidate,
  }
})
