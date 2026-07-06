import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Workspace, ScanProgress } from '../types'

export const useWorkspaceStore = defineStore('workspace', () => {
  const workspaces = ref<Workspace[]>([])
  const currentId = ref<number | null>(null)
  const scanProgress = ref<ScanProgress | null>(null)

  const current = computed(() =>
    workspaces.value.find(w => w.id === currentId.value) ?? null
  )

  async function fetchAll() {
    workspaces.value = await invoke<Workspace[]>('list_workspaces')
  }

  async function create(name: string, path: string) {
    const ws = await invoke<Workspace>('create_workspace', { name, path })
    await fetchAll()
    return ws
  }

  async function remove(id: number) {
    await invoke('delete_workspace', { id })
    if (currentId.value === id) currentId.value = null
    await fetchAll()
  }

  async function scan(id: number) {
    scanProgress.value = null
    const progress = await invoke<ScanProgress>('scan_workspace', { id })
    scanProgress.value = progress
    return progress
  }

  async function startScan(id: number) {
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

  return { workspaces, currentId, current, scanProgress, fetchAll, create, remove, scan, startScan, select }
})
