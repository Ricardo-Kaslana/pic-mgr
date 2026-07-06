<script setup lang="ts">
import { watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '../stores/settings'

const settings = useSettingsStore()
const { locale } = useI18n()

const sizePresets = [
  { label: 'S', value: 120 },
  { label: 'M', value: 200 },
  { label: 'L', value: 320 },
]

// Sync language changes to i18n
watch(() => settings.language, (lang) => {
  locale.value = lang
  localStorage.setItem('pic-mgr-lang', lang)
})
</script>

<template>
  <div class="settings-page">
    <h1>{{ $t('settings.title') }}</h1>

    <div class="section">
      <h3>{{ $t('settings.appearance') }}</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">{{ $t('settings.theme') }}</span>
          <span class="desc">{{ $t('settings.themeDesc') }}</span>
        </div>
        <select v-model="settings.theme">
          <option value="dark">{{ $t('settings.dark') }}</option>
          <option value="light">{{ $t('settings.light') }}</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-label">
          <span class="title">{{ $t('settings.thumbSize') }}</span>
          <span class="desc">{{ $t('settings.thumbSizeDesc') }}</span>
        </div>
        <div class="size-presets">
          <button
            v-for="preset in sizePresets"
            :key="preset.value"
            :class="['preset-btn', { active: settings.thumbnailSize === preset.value }]"
            @click="settings.thumbnailSize = preset.value"
          >{{ preset.label }}</button>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>{{ $t('settings.language') }}</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">{{ $t('settings.language') }}</span>
          <span class="desc">{{ $t('settings.languageDesc') }}</span>
        </div>
        <select v-model="settings.language">
          <option value="zh-CN">简体中文</option>
          <option value="en">English</option>
        </select>
      </div>
    </div>

    <div class="section">
      <h3>{{ $t('settings.about') }}</h3>
      <div class="about">
        <p><strong>PicMgr</strong> v0.3.0</p>
        <p class="desc">{{ $t('settings.aboutDesc') }}</p>
        <p class="desc">Tauri + Vue 3 + SQLite</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 36px;
  max-width: 640px;
  margin: 0 auto;
  height: 100%;
  overflow-y: auto;
}
h1 { font-size: 22px; font-weight: 700; margin-bottom: 32px; letter-spacing: -0.02em; }
.section { margin-bottom: 32px; }
.section h3 {
  font-size: 12px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 14px;
  font-weight: 600;
}
.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 8px;
  transition: border-color var(--transition-fast);
}
.setting-row:hover { border-color: var(--border); }
.setting-label .title { font-size: 14px; font-weight: 550; }
.setting-label .desc { font-size: 12px; color: var(--text-muted); display: block; margin-top: 3px; }
select { min-width: 120px; }
.size-presets { display: flex; gap: 4px; }
.preset-btn {
  width: 38px; height: 34px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12.5px; font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
}
.preset-btn:hover { border-color: var(--accent); color: var(--accent); background: var(--accent-bg); }
.preset-btn:active { transform: scale(0.95); }
.preset-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); box-shadow: 0 1px 3px rgba(0,0,0,0.2); }
.preset-btn.active:hover { background: var(--accent-hover); }
.about { padding: 16px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius); }
.about p { margin-bottom: 4px; font-size: 13px; }
.about .desc { font-size: 12px; color: var(--text-muted); }
</style>
