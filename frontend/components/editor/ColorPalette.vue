<script setup lang="ts">
const props = defineProps<{
  currentColor?: string
  type: 'text' | 'highlight'
}>()

const emit = defineEmits<{
  select: [color: string]
  clear: []
}>()

const textColors = [
  { name: 'Black', value: '#000000' },
  { name: 'Dark Gray', value: '#4b5563' },
  { name: 'Gray', value: '#9ca3af' },
  { name: 'Red', value: '#dc2626' },
  { name: 'Orange', value: '#ea580c' },
  { name: 'Amber', value: '#d97706' },
  { name: 'Green', value: '#16a34a' },
  { name: 'Teal', value: '#0d9488' },
  { name: 'Blue', value: '#2563eb' },
  { name: 'Indigo', value: '#4f46e5' },
  { name: 'Purple', value: '#9333ea' },
  { name: 'Pink', value: '#db2777' },
  { name: 'Brown', value: '#92400e' },
  { name: 'Navy', value: '#1e3a5f' },
  { name: 'White', value: '#ffffff' },
]

const highlightColors = [
  { name: 'Yellow', value: '#fef08a' },
  { name: 'Green', value: '#bbf7d0' },
  { name: 'Blue', value: '#bfdbfe' },
  { name: 'Purple', value: '#e9d5ff' },
  { name: 'Pink', value: '#fbcfe8' },
  { name: 'Orange', value: '#fed7aa' },
  { name: 'Red', value: '#fecaca' },
  { name: 'Teal', value: '#99f6e4' },
]

const colors = computed(() => props.type === 'highlight' ? highlightColors : textColors)
</script>

<template>
  <div class="color-palette">
    <div class="color-grid">
      <button
        v-for="c in colors"
        :key="c.value"
        type="button"
        class="color-swatch"
        :class="{ active: currentColor === c.value }"
        :style="{ backgroundColor: c.value }"
        :title="c.name"
        @click="emit('select', c.value)"
      />
    </div>
    <button
      type="button"
      class="color-clear"
      @click="emit('clear')"
    >
      Remove color
    </button>
  </div>
</template>

<style scoped>
.color-palette {
  padding: 8px;
  width: 200px;
}

.color-grid {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 4px;
  margin-bottom: 6px;
}

.color-swatch {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: 2px solid transparent;
  cursor: pointer;
  transition: transform 0.1s, border-color 0.1s;
}

.color-swatch:hover {
  transform: scale(1.15);
  border-color: #9ca3af;
}

.color-swatch.active {
  border-color: #111827;
  box-shadow: 0 0 0 1px white, 0 0 0 3px #111827;
}

/* White swatch needs a visible border */
.color-swatch[style*="background-color: rgb(255, 255, 255)"],
.color-swatch[style*="#ffffff"] {
  border-color: #d1d5db;
}

.color-clear {
  width: 100%;
  padding: 4px 8px;
  font-size: 12px;
  color: #6b7280;
  border-radius: 4px;
  text-align: center;
  cursor: pointer;
  transition: background-color 0.15s;
}

.color-clear:hover {
  background-color: #f3f4f6;
  color: #374151;
}
</style>
