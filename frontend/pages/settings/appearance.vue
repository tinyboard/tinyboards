<script setup lang="ts">
import { useUIStore, type ThemeMode } from '~/stores/ui'

definePageMeta({ layout: 'settings', middleware: 'guards' })
useHead({ title: 'Appearance Settings' })

const uiStore = useUIStore()

const themes = [
  { value: 'light', label: 'Light', description: 'Clean and bright' },
  { value: 'dark', label: 'Dark', description: 'Easy on the eyes' },
  { value: 'ocean', label: 'Ocean', description: 'Deep blue tones' },
  { value: 'forest', label: 'Forest', description: 'Natural greens' },
  { value: 'sunset', label: 'Sunset', description: 'Warm orange hues' },
  { value: 'purple', label: 'Purple', description: 'Rich violet tones' },
]

function selectTheme (theme: string): void {
  uiStore.setTheme(theme as ThemeMode)
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Appearance
    </h2>

    <div class="max-w-md">
      <p class="text-sm text-gray-600 mb-4">Choose a theme for your experience.</p>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <button
          v-for="theme in themes"
          :key="theme.value"
          class="p-4 rounded-lg border-2 text-left transition-colors"
          :class="uiStore.theme === theme.value
            ? 'border-primary bg-primary/5'
            : 'border-gray-200 hover:border-gray-300'"
          @click="selectTheme(theme.value)"
        >
          <p class="font-medium text-sm text-gray-900">{{ theme.label }}</p>
          <p class="text-xs text-gray-500 mt-0.5">{{ theme.description }}</p>
        </button>
      </div>
    </div>
  </div>
</template>
