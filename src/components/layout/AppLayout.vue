<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings'
import { useRouter, useRoute } from 'vue-router'

const settings = useSettingsStore()
const router = useRouter()
const route = useRoute()

function goHome() { router.push('/') }
function goSettings() { router.push('/settings') }
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="sidebar-brand" @click="goHome">
        <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="3" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <path d="M21 15l-5-5L5 21" />
        </svg>
        <span class="brand-text">PicMgr</span>
      </div>

      <nav class="sidebar-nav">
        <button
          class="nav-item"
          :class="{ active: route.name === 'home' }"
          @click="goHome"
        >
          工作区
        </button>
        <button
          class="nav-item"
          :class="{ active: route.name === 'settings' }"
          @click="goSettings"
        >
          设置
        </button>
      </nav>

      <div class="sidebar-footer">
        <button class="theme-toggle" @click="settings.toggleTheme()">
          {{ settings.theme === 'dark' ? '☀️' : '🌙' }}
        </button>
      </div>
    </aside>

    <main class="main-content">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
}

.sidebar {
  width: 200px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  color: var(--accent);
  cursor: pointer;
  border-bottom: 1px solid var(--border);
}

.brand-text {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.5px;
}

.sidebar-nav {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  text-align: left;
  background: none;
  color: var(--text-secondary);
  padding: 10px 12px;
  font-size: 14px;
}
.nav-item:hover { background: var(--bg-tertiary); color: var(--text-primary); }
.nav-item.active { background: var(--bg-tertiary); color: var(--accent); }

.sidebar-footer {
  padding: 12px;
  border-top: 1px solid var(--border);
}

.theme-toggle {
  width: 100%;
  background: none;
  font-size: 18px;
  padding: 8px;
}
.theme-toggle:hover { background: var(--bg-tertiary); }

.main-content {
  flex: 1;
  overflow: hidden;
}
</style>
