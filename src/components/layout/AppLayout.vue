<script setup lang="ts">
import { useSettingsStore } from '../../stores/settings'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const settings = useSettingsStore()
const router = useRouter()
const route = useRoute()

function goHome() { router.push('/') }
function goSettings() { router.push('/settings') }
</script>

<template>
  <div class="app-layout" :class="{ collapsed: settings.sidebarCollapsed }">
    <aside class="sidebar">
      <div class="sidebar-brand" @click="goHome">
        <div class="brand-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="3" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <path d="M21 15l-5-5L5 21" />
          </svg>
        </div>
        <span class="brand-text">PicMgr</span>
        <button
          class="sidebar-toggle"
          :aria-label="t('nav.toggleSidebar')"
          @click.stop="settings.toggleSidebar()"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6" />
          </svg>
        </button>
      </div>

      <nav class="sidebar-nav">
        <button
          class="nav-item"
          :class="{ active: route.name === 'home' }"
          :data-label="t('nav.home')"
          @click="goHome"
        >
          <span class="nav-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
            </svg>
          </span>
          <span class="nav-label">{{ t('nav.home') }}</span>
        </button>
        <button
          class="nav-item"
          :class="{ active: route.name === 'settings' }"
          :data-label="t('nav.settings')"
          @click="goSettings"
        >
          <span class="nav-icon">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
            </svg>
          </span>
          <span class="nav-label">{{ t('nav.settings') }}</span>
        </button>
      </nav>

      <div class="sidebar-footer">
        <button
          class="theme-toggle"
          :data-label="settings.theme === 'dark' ? t('settings.light') : t('settings.dark')"
          @click="settings.toggleTheme()"
        >
          <span class="theme-icon">{{ settings.theme === 'dark' ? '☀️' : '🌙' }}</span>
          <span>{{ settings.theme === 'dark' ? t('settings.light') : t('settings.dark') }}</span>
        </button>
        <p class="version">v0.3</p>
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
  width: 220px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width 200ms ease-out;
}
.app-layout.collapsed .sidebar { width: 64px; }

/* Brand */
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 18px 16px;
  color: var(--accent);
  cursor: pointer;
  border-bottom: 1px solid var(--border-light);
}
.brand-icon {
  width: 34px; height: 34px;
  background: linear-gradient(135deg, var(--accent-bg) 0%, transparent 100%);
  border-radius: var(--radius);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.brand-text {
  font-size: 16px;
  font-weight: 700;
  letter-spacing: -0.03em;
  transition: opacity 150ms ease-out;
}
.sidebar-toggle {
  margin-left: auto;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: transparent;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  flex-shrink: 0;
  cursor: pointer;
  transition: transform var(--transition-fast), background var(--transition-fast), border-color var(--transition-fast), color var(--transition-fast);
}
.sidebar-toggle:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-color: var(--border);
}
.sidebar-toggle:active { opacity: 0.7; }
.app-layout.collapsed .sidebar-toggle { transform: rotate(180deg); }

.app-layout.collapsed .sidebar-brand { padding-left: 15px; padding-right: 15px; }
.app-layout.collapsed .brand-text {
  opacity: 0;
  width: 0;
  pointer-events: none;
  overflow: hidden;
}

/* Nav */
.sidebar-nav {
  flex: 1;
  padding: 12px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.nav-item {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  background: transparent;
  color: var(--text-secondary);
  padding: 9px 12px;
  font-size: 13.5px;
  font-weight: 450;
  border-radius: var(--radius-sm);
  border: 1px solid transparent;
  transition: all var(--transition-fast);
}
.nav-item:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
  border-color: var(--border);
}
.nav-item:active { opacity: 0.7; }
.nav-item.active {
  background: var(--accent-bg);
  color: var(--accent);
  font-weight: 550;
  border-color: transparent;
}
.nav-item.active .nav-icon { opacity: 1; }
.nav-icon {
  width: 20px;
  opacity: 0.55;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: opacity var(--transition-fast);
}
.nav-item:hover .nav-icon { opacity: 0.8; }
.nav-label {
  transition: opacity 150ms ease-out;
}
.app-layout.collapsed .nav-item { justify-content: center; gap: 0; }
.app-layout.collapsed .nav-label { opacity: 0; }

/* Footer */
.sidebar-footer {
  padding: 12px 10px;
  border-top: 1px solid var(--border-light);
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.theme-toggle {
  width: 100%;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  padding: 8px 12px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
  border: 1px solid transparent;
  transition: all var(--transition-fast);
}
.theme-toggle:hover {
  background: var(--bg-elevated);
  color: var(--text-primary);
  border-color: var(--border);
}
.theme-toggle:active { opacity: 0.7; }
.theme-icon {
  width: 20px;
  text-align: center;
  flex-shrink: 0;
  font-size: 14px;
}
.theme-toggle > span:last-child { transition: width 150ms ease-out, opacity 150ms ease-out; }
.app-layout.collapsed .theme-toggle {
  justify-content: center;
  padding-left: 0;
  padding-right: 0;
}
.app-layout.collapsed .theme-toggle > span:last-child {
  width: 0;
  overflow: hidden;
  opacity: 0;
}
.version {
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  opacity: 0.6;
}
.app-layout.collapsed .version { display: none; }

/* Tooltips (collapsed-state only) */
.app-layout.collapsed .nav-item,
.app-layout.collapsed .theme-toggle {
  position: relative;
}
.app-layout.collapsed .nav-item::after,
.app-layout.collapsed .theme-toggle::after {
  content: attr(data-label);
  position: absolute;
  left: calc(100% + 8px);
  top: 50%;
  transform: translateY(-50%);
  background: var(--bg-elevated);
  color: var(--text-primary);
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--transition-fast);
  border: 1px solid var(--border);
  z-index: 100;
}
.app-layout.collapsed .nav-item:hover::after,
.app-layout.collapsed .nav-item:focus-visible::after,
.app-layout.collapsed .theme-toggle:hover::after,
.app-layout.collapsed .theme-toggle:focus-visible::after {
  opacity: 1;
}

.main-content {
  flex: 1;
  overflow: hidden;
  background: var(--bg-primary);
}
</style>