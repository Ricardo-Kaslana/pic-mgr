import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<'dark' | 'light'>(
    (localStorage.getItem('pic-mgr-theme') as 'dark' | 'light') ?? 'dark'
  )
  const thumbnailSize = ref<number>(
    Number(localStorage.getItem('pic-mgr-thumb-size')) || 200
  )
  const language = ref<string>(
    localStorage.getItem('pic-mgr-lang') ?? 'zh-CN'
  )

  watch(theme, (v) => {
    localStorage.setItem('pic-mgr-theme', v)
    document.documentElement.setAttribute('data-theme', v)
  }, { immediate: true })

  watch(thumbnailSize, (v) => {
    localStorage.setItem('pic-mgr-thumb-size', String(v))
  })

  watch(language, (v) => {
    localStorage.setItem('pic-mgr-lang', v)
  })

  const sidebarCollapsed = ref<boolean>(
    localStorage.getItem('pic-mgr-sidebar-collapsed') === 'true'
  )

  watch(sidebarCollapsed, (v) => {
    localStorage.setItem('pic-mgr-sidebar-collapsed', String(v))
  }, { immediate: true })

  // Default sort mirrors the original hardcoded fallback in imageStore
  // filters ('indexed_at' / 'desc'). Stored as plain strings (not a
  // typed union) so a stale localStorage entry from a renamed sort
  // key does not crash the store - GridToolbar falls back to its
  // 'indexedAt' label when sort_by does not match any option.
  const sortBy = ref<string>(
    localStorage.getItem('pic-mgr-sort-by') ?? 'indexed_at'
  )
  const sortDir = ref<string>(
    localStorage.getItem('pic-mgr-sort-dir') ?? 'desc'
  )

  watch(sortBy, (v) => {
    localStorage.setItem('pic-mgr-sort-by', v)
  })
  watch(sortDir, (v) => {
    localStorage.setItem('pic-mgr-sort-dir', v)
  })

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return { theme, thumbnailSize, language, sidebarCollapsed, sortBy, sortDir, toggleTheme, toggleSidebar }
})