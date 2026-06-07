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
  padding: 2px 8px;
  border-radius: 100px;
  font-size: 12px;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  border: 1px solid var(--border);
}
</style>
