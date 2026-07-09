import { defineStore } from 'pinia'
import { useLocalStorage } from '../composables/useLocalStorage'
import { STORAGE_KEYS } from '../composables/storageKeys'

type Theme = 'dark' | 'light'

// Theme defaults to dark. We coerce a stored value through a small
// whitelist so a hand-edited localStorage entry like
//   localStorage.setItem('pic-mgr-theme', 'sepia')
// does not end up applied as a data-theme attribute the stylesheet
// has no rules for. Unrecognized values silently fall back to dark.
function parseTheme(raw: string): Theme {
  return raw === 'light' || raw === 'dark' ? raw : 'dark'
}

export const useSettingsStore = defineStore('settings', () => {
  const theme = useLocalStorage<Theme>(STORAGE_KEYS.theme, 'dark', {
    read: parseTheme,
    apply: (v) => document.documentElement.setAttribute('data-theme', v),
  })

  // Coerce non-numeric / NaN values to the default. The previous
  // implementation used `Number(...) || 200`, which silently turned
  // a stored "0" (i.e. user picked nothing) into 200.
  const thumbnailSize = useLocalStorage<number>(STORAGE_KEYS.thumbnailSize, 200, {
    read: (raw) => {
      const n = Number(raw)
      return Number.isFinite(n) && n > 0 ? n : 200
    },
    write: String,
  })

  const language = useLocalStorage<string>(STORAGE_KEYS.language, 'zh-CN')

  const sidebarCollapsed = useLocalStorage<boolean>(STORAGE_KEYS.sidebarCollapsed, false, {
    read: (raw) => raw === 'true',
    write: (v) => String(v),
  })

  const browserSidebarCollapsed = useLocalStorage<boolean>(
    STORAGE_KEYS.browserSidebarCollapsed,
    false,
    {
      read: (raw) => raw === 'true',
      write: (v) => String(v),
    }
  )

  // Default sort mirrors the original hardcoded fallback in imageStore
  // filters ('indexed_at' / 'desc'). Stored as plain strings (not a
  // typed union) so a stale localStorage entry from a renamed sort
  // key does not crash the store - GridToolbar falls back to its
  // 'indexedAt' label when sort_by does not match any option.
  const sortBy = useLocalStorage<string>(STORAGE_KEYS.sortBy, 'indexed_at')
  const sortDir = useLocalStorage<string>(STORAGE_KEYS.sortDir, 'desc')

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return {
    theme,
    thumbnailSize,
    language,
    sidebarCollapsed,
    browserSidebarCollapsed,
    sortBy,
    sortDir,
    toggleTheme,
    toggleSidebar,
  }
})