<script setup lang="ts">
import { useSettingsStore } from '../stores/settings'

const settings = useSettingsStore()
</script>

<template>
  <div class="settings-page">
    <h1>设置</h1>

    <div class="section">
      <h3>外观</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">主题</span>
          <span class="desc">暗色或亮色界面主题</span>
        </div>
        <select v-model="settings.theme">
          <option value="dark">暗色</option>
          <option value="light">亮色</option>
        </select>
      </div>

      <div class="setting-row">
        <div class="setting-label">
          <span class="title">缩略图尺寸</span>
          <span class="desc">缩略图网格中每张图的高度 (像素)</span>
        </div>
        <div class="size-control">
          <input
            type="range"
            min="120"
            max="400"
            step="20"
            v-model.number="settings.thumbnailSize"
          />
          <span class="size-value">{{ settings.thumbnailSize }}px</span>
        </div>
      </div>
    </div>

    <div class="section">
      <h3>语言</h3>
      <div class="setting-row">
        <div class="setting-label">
          <span class="title">界面语言</span>
          <span class="desc">应用界面显示语言</span>
        </div>
        <select v-model="settings.language">
          <option value="zh-CN">简体中文</option>
          <option value="en">English</option>
        </select>
      </div>
    </div>

    <div class="section">
      <h3>关于</h3>
      <div class="about">
        <p><strong>PicMgr</strong> v0.1.0</p>
        <p class="desc">轻量本地图片管理器</p>
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
.size-control { display: flex; align-items: center; gap: 12px; }
.size-control input[type="range"] { width: 120px; accent-color: var(--accent); }
.size-value { font-size: 13px; color: var(--text-secondary); min-width: 50px; }
.about p { margin-bottom: 4px; }
.about .desc { font-size: 12px; color: var(--text-muted); }
</style>
