<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  modelValue: string
  options?: Array<{ label: string; value: string }>
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const isOpen = ref(false)

const defaultOptions = [
  { label: 'Hot', value: 'hot' },
  { label: 'New', value: 'new' },
  { label: 'Top', value: 'topDay' },
  { label: 'Rising', value: 'active' },
  { label: 'Controversial', value: 'mostComments' },
]

const currentOptions = computed(() => props.options ?? defaultOptions)
const currentLabel = computed(() => {
  const opt = currentOptions.value.find(o => o.value === props.modelValue)
  return opt?.label ?? 'Sort'
})

function select (value: string): void {
  emit('update:modelValue', value)
  isOpen.value = false
}
</script>

<template>
  <div class="relative">
    <button
      class="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-600 hover:text-gray-900 bg-white border border-gray-200 rounded-md shadow-sm transition-colors"
      @click="isOpen = !isOpen"
      @blur="isOpen = false"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4h13M3 8h9m-9 4h6m4 0l4-4m0 0l4 4m-4-4v12" />
      </svg>
      Sort: <span class="font-medium">{{ currentLabel }}</span>
      <svg
        class="w-3 h-3 ml-0.5 transition-transform duration-150"
        :class="isOpen ? 'rotate-180' : ''"
        fill="none" stroke="currentColor" viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <div
      v-if="isOpen"
      class="absolute left-0 top-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg z-20 min-w-[140px] py-1 dropdown-enter"
    >
      <button
        v-for="opt in currentOptions"
        :key="opt.value"
        class="w-full text-left px-3 py-1.5 text-sm transition-colors"
        :class="modelValue === opt.value
          ? 'text-primary font-medium bg-primary/5'
          : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'"
        @mousedown.prevent="select(opt.value)"
      >
        {{ opt.label }}
      </button>
    </div>
  </div>
</template>
