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

  function toggleTheme() {
    theme.value = theme.value === 'dark' ? 'light' : 'dark'
  }

  return { theme, thumbnailSize, language, toggleTheme }
})
