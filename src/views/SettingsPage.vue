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
        <p><strong>PicMgr</strong> v0.1.0</p>
        <p class="desc">{{ $t('settings.aboutDesc') }}</p>
        <p class="desc">Tauri + Vue 3 + SQLite</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  padding: 32px;
  max-width: 640px;
  margin: 0 auto;
  height: 100%;
  overflow-y: auto;
}
h1 { font-size: 24px; font-weight: 700; margin-bottom: 28px; }
.section { margin-bottom: 28px; }
.section h3 { font-size: 14px; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 16px; }
.setting-row {
  display: flex; align-items: center; justify-content: space-between;
  padding: 14px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  margin-bottom: 8px;
}
.setting-label .title { font-size: 14px; font-weight: 500; }
.setting-label .desc { font-size: 12px; color: var(--text-muted); display: block; margin-top: 2px; }
select {
  min-width: 120px;
}
.size-presets { display: flex; gap: 6px; }
.preset-btn {
  width: 36px; height: 32px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px; font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}
.preset-btn:hover { border-color: var(--accent); }
.preset-btn.active { background: var(--accent); color: #fff; border-color: var(--accent); }
.about p { margin-bottom: 4px; }
.about .desc { font-size: 12px; color: var(--text-muted); }
</style>
