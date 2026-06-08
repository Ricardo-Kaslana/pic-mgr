<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '../stores/workspace'
import WorkspaceCard from '../components/workspace/WorkspaceCard.vue'
import CreateWorkspaceDialog from '../components/workspace/CreateWorkspaceDialog.vue'

const router = useRouter()
const store = useWorkspaceStore()
const showDialog = ref(false)

onMounted(() => store.fetchAll())

async function handleCreate(name: string, path: string) {
  const ws = await store.create(name, path)
  showDialog.value = false
  await store.scan(ws.id)
  router.push(`/workspace/${ws.id}`)
}

function handleSelect(id: number) {
  router.push(`/workspace/${id}`)
}

async function handleDelete(id: number) {
  if (confirm('确定删除此工作区？不会影响原始图片文件。')) {
    await store.remove(id)
  }
}

async function handleRescan(id: number) {
  await store.scan(id)
}
</script>

<template>
  <div class="home">
    <header class="home-header">
      <h1>工作区</h1>
      <button class="primary" @click="showDialog = true">+ 新建工作区</button>
    </header>

    <div v-if="store.workspaces.length === 0" class="empty">
      <div class="empty-icon">🖼️</div>
      <p class="empty-title">还没有工作区</p>
      <p class="hint">将一个图片文件夹设为工作区，即可开始管理</p>
      <button class="primary" @click="showDialog = true" style="margin-top: 16px;">+ 新建工作区</button>
    </div>

    <div class="grid">
      <WorkspaceCard
        v-for="ws in store.workspaces"
        :key="ws.id"
        :workspace="ws"
        @select="handleSelect"
        @delete="handleDelete"
        @rescan="handleRescan"
      />
    </div>

    <CreateWorkspaceDialog
      v-if="showDialog"
      @create="handleCreate"
      @close="showDialog = false"
    />
  </div>
</template>

<style scoped>
.home { padding: 32px; max-width: 900px; margin: 0 auto; height: 100%; overflow-y: auto; }
.home-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px; }
.home-header h1 { font-size: 24px; font-weight: 700; }
.empty { text-align: center; padding: 80px 0; color: var(--text-muted); }
.empty-icon { font-size: 64px; margin-bottom: 16px; opacity: 0.6; }
.empty-title { font-size: 18px; font-weight: 600; color: var(--text-secondary); margin-bottom: 8px; }
.empty .hint { font-size: 14px; color: var(--text-muted); }
.grid { display: flex; flex-direction: column; gap: 12px; }
</style>
