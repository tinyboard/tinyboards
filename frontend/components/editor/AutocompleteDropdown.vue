<script setup lang="ts">
import { ref, watch, computed, nextTick } from 'vue'
import type { AutocompleteSuggestion } from '~/composables/useEditorAutocomplete'

const props = defineProps<{
  suggestions: AutocompleteSuggestion[]
  selectedIndex: number
  position: { top: number; left: number; height: number }
  containerEl?: HTMLElement | null
}>()

const emit = defineEmits<{
  select: [suggestion: AutocompleteSuggestion]
}>()

const dropdownRef = ref<HTMLElement | null>(null)

const style = computed(() => {
  const { top, left, height } = props.position

  // Position below the caret by default
  let y = top + height + 4
  let x = left

  // If we have a reference to the dropdown, check if it overflows the viewport
  if (dropdownRef.value) {
    const rect = dropdownRef.value.getBoundingClientRect()
    const viewportHeight = window.innerHeight
    const viewportWidth = window.innerWidth

    // Flip above if it would overflow below
    if (y + rect.height > viewportHeight - 8) {
      y = top - rect.height - 4
    }

    // Clamp horizontally
    if (x + rect.width > viewportWidth - 8) {
      x = viewportWidth - rect.width - 8
    }
    if (x < 8) x = 8
  }

  return {
    position: 'fixed' as const,
    top: `${y}px`,
    left: `${x}px`,
    zIndex: 9999,
  }
})

// Scroll selected item into view
watch(() => props.selectedIndex, async () => {
  await nextTick()
  const el = dropdownRef.value?.querySelector('[data-selected="true"]')
  el?.scrollIntoView({ block: 'nearest' })
})

function handleSelect (suggestion: AutocompleteSuggestion): void {
  emit('select', suggestion)
}

function isCustomEmoji (suggestion: AutocompleteSuggestion): boolean {
  return suggestion.type === 'emoji' && suggestion.secondary === 'custom'
}
</script>

<template>
  <Teleport to="body">
    <div
      ref="dropdownRef"
      :style="style"
      class="bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden max-h-64 overflow-y-auto min-w-[200px] max-w-[320px]"
      @mousedown.prevent
    >
      <ul role="listbox" class="py-1">
        <li
          v-for="(suggestion, index) in suggestions"
          :key="`${suggestion.type}-${suggestion.label}`"
          role="option"
          :aria-selected="index === selectedIndex"
          :data-selected="index === selectedIndex"
          class="px-3 py-1.5 cursor-pointer flex items-center gap-2 text-sm transition-colors"
          :class="index === selectedIndex ? 'bg-primary/10 text-gray-900' : 'text-gray-700 hover:bg-gray-50'"
          @mousedown.prevent="handleSelect(suggestion)"
        >
          <!-- Icon -->
          <span v-if="suggestion.type === 'emoji' && !isCustomEmoji(suggestion)" class="text-lg flex-shrink-0 w-6 text-center">
            {{ suggestion.icon }}
          </span>
          <img
            v-else-if="isCustomEmoji(suggestion) && suggestion.icon"
            :src="suggestion.icon"
            :alt="suggestion.label"
            class="w-5 h-5 flex-shrink-0 object-contain"
          >
          <span v-else-if="suggestion.type === 'user'" class="flex-shrink-0 w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center text-xs font-medium text-gray-600">
            {{ suggestion.label.charAt(0).toUpperCase() }}
          </span>
          <img
            v-else-if="suggestion.type === 'board' && suggestion.icon"
            :src="suggestion.icon"
            :alt="suggestion.label"
            class="w-5 h-5 flex-shrink-0 rounded object-cover"
          >
          <span v-else-if="suggestion.type === 'board'" class="flex-shrink-0 w-5 h-5 rounded bg-primary/20 flex items-center justify-center text-[10px] font-bold text-primary">
            b
          </span>

          <!-- Label and secondary text -->
          <div class="flex-1 min-w-0">
            <div class="truncate font-medium">{{ suggestion.label }}</div>
            <div v-if="suggestion.secondary && suggestion.secondary !== 'custom'" class="truncate text-xs text-gray-400">
              {{ suggestion.secondary }}
            </div>
          </div>

          <!-- Type badge for mixed results -->
          <span v-if="isCustomEmoji(suggestion)" class="text-[10px] text-gray-400 flex-shrink-0">
            custom
          </span>
        </li>
      </ul>
    </div>
  </Teleport>
</template>
