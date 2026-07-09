/**
 * Centralized localStorage keys. Refer to these instead of inlining
 * magic strings in stores and components so renames stay safe.
 */
export const STORAGE_KEYS = {
  theme: 'pic-mgr-theme',
  thumbnailSize: 'pic-mgr-thumb-size',
  language: 'pic-mgr-lang',
  sidebarCollapsed: 'pic-mgr-sidebar-collapsed',
  browserSidebarCollapsed: 'pic-mgr-browser-sidebar-collapsed',
  sortBy: 'pic-mgr-sort-by',
  sortDir: 'pic-mgr-sort-dir',
} as const

export type StorageKey = typeof STORAGE_KEYS[keyof typeof STORAGE_KEYS]