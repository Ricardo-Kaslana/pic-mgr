<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'

const emit = defineEmits<{ create: [name: string, path: string]; close: [] }>()

const name = ref('')
const path = ref('')
const error = ref('')

async function browseFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择图片文件夹',
  })
  if (selected) {
    path.value = selected as string
    // Auto-fill name from folder name if empty
    if (!name.value) {
      const parts = (selected as string).replace(/[/\\]$/, '').split(/[/\\]/)
      name.value = parts[parts.length - 1] || ''
    }
  }
}

function submit() {
  if (!name.value.trim() || !path.value.trim()) {
    error.value = '请填写名称和路径'
    return
  }
  emit('create', name.value.trim(), path.value.trim())
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="dialog">
      <h2>新建工作区</h2>
      <div class="field">
        <label>名称</label>
        <input v-model="name" placeholder="例如：我的照片" @keyup.enter="submit" />
      </div>
      <div class="field">
        <label>文件夹路径</label>
        <div class="path-row">
          <input v-model="path" placeholder="点击右侧按钮选择文件夹" @keyup.enter="submit" />
          <button class="browse-btn" @click="browseFolder">📂 浏览</button>
        </div>
      </div>
      <p v-if="error" class="error">{{ error }}</p>
      <div class="actions">
        <button class="secondary" @click="emit('close')">取消</button>
        <button class="primary" @click="submit">创建工作区</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.6);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 28px;
  width: 440px;
}
.dialog h2 { margin-bottom: 20px; font-size: 18px; }
.field { margin-bottom: 16px; }
.field label { display: block; margin-bottom: 6px; font-size: 13px; color: var(--text-secondary); }
.field input { width: 100%; }
.path-row { display: flex; gap: 8px; }
.path-row input { flex: 1; }
.browse-btn {
  white-space: nowrap;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  color: var(--text-primary);
  font-size: 13px;
}
.browse-btn:hover { border-color: var(--accent); }
.error { color: var(--danger); font-size: 12px; margin-bottom: 12px; }
.actions { display: flex; gap: 12px; justify-content: flex-end; margin-top: 8px; }
</style>
