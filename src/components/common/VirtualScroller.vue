<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'

const props = withDefaults(defineProps<{
  items: any[]
  itemHeight: number
  gap?: number
  buffer?: number
}>(), { gap: 8, buffer: 5 })

const emit = defineEmits<{ loadMore: [] }>()

const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const containerHeight = ref(600)

const totalHeight = computed(() =>
  props.items.length * (props.itemHeight + props.gap)
)

const startIndex = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / (props.itemHeight + props.gap)) - props.buffer)
)

const endIndex = computed(() =>
  Math.min(
    props.items.length,
    Math.ceil((scrollTop.value + containerHeight.value) / (props.itemHeight + props.gap)) + props.buffer
  )
)

const visibleItems = computed(() =>
  props.items.slice(startIndex.value, endIndex.value).map((item, i) => ({
    item,
    index: startIndex.value + i,
    style: {
      position: 'absolute' as const,
      top: `${(startIndex.value + i) * (props.itemHeight + props.gap)}px`,
      height: `${props.itemHeight}px`,
      left: 0,
      right: 0,
    }
  }))
)

function onScroll(e: Event) {
  const el = e.target as HTMLElement
  scrollTop.value = el.scrollTop
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 500) {
    emit('loadMore')
  }
}

onMounted(() => {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight
    const observer = new ResizeObserver(entries => {
      containerHeight.value = entries[0].contentRect.height
    })
    observer.observe(containerRef.value)
  }
})
</script>

<template>
  <div ref="containerRef" class="virtual-scroller" @scroll="onScroll">
    <div :style="{ height: `${totalHeight}px`, position: 'relative' }">
      <div
        v-for="vi in visibleItems"
        :key="vi.index"
        :style="vi.style"
      >
        <slot name="item" :item="vi.item" :index="vi.index" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.virtual-scroller {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}
</style>
