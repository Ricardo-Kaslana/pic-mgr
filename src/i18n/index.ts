import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import en from './en'

import { STORAGE_KEYS } from '../composables/storageKeys'

const savedLang = localStorage.getItem(STORAGE_KEYS.language) || 'zh-CN'

const i18n = createI18n({
  legacy: false,
  locale: savedLang,
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    en,
  },
})

export default i18n
