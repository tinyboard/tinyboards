<script setup lang="ts">
import { useUIStore } from '~/stores/ui'

const uiStore = useUIStore()
</script>

<template>
  <div class="flex items-center border border-gray-200 rounded-md overflow-hidden">
    <button
      class="p-1.5 transition-colors"
      :class="uiStore.postViewMode === 'expanded'
        ? 'bg-primary/10 text-primary'
        : 'text-gray-400 hover:text-gray-600 hover:bg-gray-50'"
      aria-label="Expanded view"
      @click="uiStore.setPostViewMode('expanded')"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
      </svg>
    </button>
    <button
      class="p-1.5 border-l border-gray-200 transition-colors"
      :class="uiStore.postViewMode === 'compact'
        ? 'bg-primary/10 text-primary'
        : 'text-gray-400 hover:text-gray-600 hover:bg-gray-50'"
      aria-label="Compact view"
      @click="uiStore.setPostViewMode('compact')"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </button>
  </div>
</template>
