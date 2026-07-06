<script setup lang="ts">
defineProps<{ name: string; color?: string | null }>()
</script>

<template>
  <span class="tag-badge" :style="color ? { background: color + '22', color: color, borderColor: color } : {}">
    {{ name }}
  </span>
</template>

<style scoped>
.tag-badge {
  display: inline-block;
  padding: 3px 10px;
  border-radius: 100px;
  font-size: 11.5px;
  font-weight: 500;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
  transition: all var(--transition-fast);
}
.tag-badge:hover {
  border-color: var(--text-muted);
  color: var(--text-primary);
}
</style>
