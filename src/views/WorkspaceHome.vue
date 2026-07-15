<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useWorkspaceStore } from '../stores/workspace'
import WorkspaceCard from '../components/workspace/WorkspaceCard.vue'
import CreateWorkspaceDialog from '../components/workspace/CreateWorkspaceDialog.vue'
import MissingFilesDialog from '../components/workspace/MissingFilesDialog.vue'
import { useToast } from '../composables/useToast'
import type { DeletedCandidate } from '../types'

const { t } = useI18n()
const router = useRouter()
const store = useWorkspaceStore()
const { showToast } = useToast()
const showDialog = ref(false)
const deleteError = ref('')
const scanningIds = reactive(new Set<number>())
const justScannedIds = reactive(new Set<number>())
const pendingMissing = ref<DeletedCandidate[] | null>(null)

onMounted(() => store.fetchAll())

async function handleCreate(name: string, path: string) {
  const ws = await store.create(name, path)
  showDialog.value = false
  router.push(`/workspace/${ws.id}`)
  store.sync(ws.id)
}

function handleSelect(id: number) {
  router.push(`/workspace/${id}`)
}

async function handleDelete(id: number) {
  if (!confirm(t('workspace.confirmDelete'))) return
  deleteError.value = ''
  try {
    await store.remove(id)
  } catch (e: any) {
    deleteError.value = t('workspace.deleteFailed', { error: e?.toString?.() ?? e ?? '' })
  }
}

async function handleRescan(id: number) {
  // Guard against double-clicks: syncingIds is owned by the store.
  if (store.isSyncing(id)) return
  if (!confirm(t('workspace.syncConfirm'))) return
  scanningIds.add(id)
  try {
    const result = await store.sync(id)
    justScannedIds.add(id)
    setTimeout(() => justScannedIds.delete(id), 2000)
    showToast(t('sync.resultToast', {
      added: result.totals.added,
      modified: result.totals.modified,
      rebound: result.totals.rebound,
      deletedCandidates: result.totals.deletedCandidates,
    }))
    if (result.deletedCandidates.length > 0) {
      pendingMissing.value = result.deletedCandidates
    }
  } finally {
    scanningIds.delete(id)
  }
}

async function handleFullReset(id: number) {
  if (!confirm(t('workspace.fullResetFinal'))) return
  scanningIds.add(id)
  try {
    await store.rescan(id)
    justScannedIds.add(id)
    setTimeout(() => justScannedIds.delete(id), 2000)
  } finally {
    scanningIds.delete(id)
  }
}

async function handleMissingApply(ids: number[]) {
  try {
    const deleted = await store.applyDeletions(ids)
    showToast(t('missing.deletedToast', { count: deleted }))
  } catch (e: any) {
    showToast(t('missing.deleteFailed', { error: e?.toString?.() ?? e ?? '' }))
  } finally {
    pendingMissing.value = null
  }
}
</script>

<template>
  <div class="home">
    <header class="home-header">
      <h1>{{ t('workspace.title') }}</h1>
      <button class="primary" @click="showDialog = true">{{ t('workspace.create') }}</button>
    </header>

    <div v-if="deleteError" class="error-banner" @click="deleteError = ''">
      {{ deleteError }}
    </div>

    <div v-if="store.workspaces.length === 0" class="empty">
      <div class="empty-icon">🖼️</div>
      <p class="empty-title">{{ t('workspace.empty') }}</p>
      <p class="hint">{{ t('workspace.emptyHint') }}</p>
      <button class="primary" @click="showDialog = true" style="margin-top: 16px;">{{ t('workspace.create') }}</button>
    </div>

    <div class="grid">
      <WorkspaceCard
        v-for="ws in store.workspaces"
        :key="ws.id"
        :workspace="ws"
        :scanning="scanningIds.has(ws.id)"
        :syncing="store.isSyncing(ws.id)"
        :just-scanned="justScannedIds.has(ws.id)"
        @select="handleSelect"
        @delete="handleDelete"
        @rescan="handleRescan"
        @full-reset="handleFullReset"
      />
    </div>

    <CreateWorkspaceDialog
      v-if="showDialog"
      @create="handleCreate"
      @close="showDialog = false"
    />

    <MissingFilesDialog
      v-if="pendingMissing"
      :candidates="pendingMissing"
      @apply="handleMissingApply"
      @close="pendingMissing = null"
    />
  </div>
</template>

<style scoped>
.home { padding: 36px; max-width: 900px; margin: 0 auto; height: 100%; overflow-y: auto; }
.home-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 28px; }
.home-header h1 { font-size: 22px; font-weight: 700; letter-spacing: -0.02em; }
.empty { text-align: center; padding: 80px 0; color: var(--text-muted); }
.empty-icon { font-size: 56px; margin-bottom: 12px; opacity: 0.5; }
.empty-title { font-size: 17px; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px; }
.empty .hint { font-size: 13px; color: var(--text-muted); }
.grid { display: flex; flex-direction: column; gap: 10px; }
.error-banner {
  background: var(--danger-bg, #fef2f2);
  color: var(--danger, #ef4444);
  border: 1px solid var(--danger, #ef4444);
  border-radius: var(--radius, 8px);
  padding: 12px 16px;
  margin-bottom: 16px;
  font-size: 13px;
  cursor: pointer;
  user-select: none;
}
.error-banner:hover { opacity: 0.85; }
</style>
