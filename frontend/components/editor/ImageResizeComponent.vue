<script setup lang="ts">
import { ref, computed } from 'vue'
import { NodeViewWrapper, nodeViewProps } from '@tiptap/vue-3'

const props = defineProps(nodeViewProps)

const resizing = ref(false)
const startX = ref(0)
const startWidth = ref(0)
const imgRef = ref<HTMLImageElement | null>(null)

const imgWidth = computed(() => {
  const w = props.node?.attrs?.width
  if (!w) return undefined
  return typeof w === 'number' ? `${w}px` : w
})

function onMouseDown (e: MouseEvent): void {
  e.preventDefault()
  e.stopPropagation()
  resizing.value = true
  startX.value = e.clientX
  startWidth.value = imgRef.value?.offsetWidth ?? 300

  const onMouseMove = (ev: MouseEvent) => {
    const diff = ev.clientX - startX.value
    const newWidth = Math.max(50, startWidth.value + diff)
    props.updateAttributes({ width: `${newWidth}px` })
  }

  const onMouseUp = () => {
    resizing.value = false
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}
</script>

<template>
  <NodeViewWrapper class="image-resize-wrapper" :class="{ 'is-selected': selected }">
    <div class="image-resize-container" :style="{ width: imgWidth }">
      <img
        ref="imgRef"
        :src="node.attrs.src"
        :alt="node.attrs.alt ?? ''"
        :title="node.attrs.title ?? ''"
        :style="{ width: imgWidth }"
        class="image-resize-img"
        draggable="false"
      />
      <div
        v-if="selected"
        class="resize-handle resize-handle-right"
        @mousedown="onMouseDown"
      />
      <div
        v-if="selected"
        class="resize-handle resize-handle-bottom-right"
        @mousedown="onMouseDown"
      />
    </div>
  </NodeViewWrapper>
</template>

<style scoped>
.image-resize-wrapper {
  display: inline-block;
  position: relative;
  line-height: 0;
}

.image-resize-container {
  display: inline-block;
  position: relative;
  max-width: 100%;
}

.image-resize-img {
  display: block;
  max-width: 100%;
  border-radius: 6px;
  cursor: default;
}

.is-selected .image-resize-img {
  outline: 2px solid rgb(var(--color-primary, 99 102 241));
  outline-offset: 2px;
}

.resize-handle {
  position: absolute;
  background: rgb(var(--color-primary, 99 102 241));
  border: 2px solid white;
  border-radius: 3px;
  z-index: 10;
}

.resize-handle-right {
  width: 8px;
  height: 24px;
  right: -4px;
  top: 50%;
  transform: translateY(-50%);
  cursor: ew-resize;
}

.resize-handle-bottom-right {
  width: 10px;
  height: 10px;
  right: -5px;
  bottom: -5px;
  cursor: nwse-resize;
  border-radius: 2px;
}
</style>
