<script setup lang="ts">
import type { Board } from '~/types/generated'

defineProps<{
  board: Board
}>()
</script>

<template>
  <div class="space-y-6">
    <h2 class="text-lg font-semibold text-gray-900">
      Board Settings
    </h2>

    <form class="space-y-4" @submit.prevent>
      <div>
        <label for="board-title" class="block text-sm font-medium text-gray-700 mb-1">Title</label>
        <input id="board-title" type="text" class="form-input" :value="board.title">
      </div>

      <div>
        <label for="board-description" class="block text-sm font-medium text-gray-700 mb-1">Description</label>
        <textarea id="board-description" class="form-input min-h-[80px]" :value="board.description ?? ''" />
      </div>

      <div>
        <label for="board-sidebar" class="block text-sm font-medium text-gray-700 mb-1">Sidebar</label>
        <textarea id="board-sidebar" class="form-input min-h-[120px]" :value="board.sidebar ?? ''" />
      </div>

      <div class="flex items-center gap-2">
        <input id="board-nsfw" type="checkbox" :checked="board.isNSFW">
        <label for="board-nsfw" class="text-sm text-gray-700">Mark as NSFW</label>
      </div>

      <button type="submit" class="button primary">
        Save Changes
      </button>
    </form>
  </div>
</template>
