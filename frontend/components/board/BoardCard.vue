<script setup lang="ts">
import type { Board } from '~/types/generated'

defineProps<{
  board: Board & { mode?: string }
}>()
</script>

<template>
  <NuxtLink
    :to="`/b/${board.name}`"
    class="block bg-white border border-gray-200 rounded p-4 hover:border-gray-300 transition-colors no-underline"
    :class="board.mode === 'forum' ? 'border-l-2 border-l-purple-400' : 'border-l-2 border-l-blue-400'"
  >
    <div class="flex items-center gap-3">
      <CommonAvatar :src="board.icon ?? undefined" :name="board.name" size="md" />
      <div class="flex-1 min-w-0">
        <h3 class="font-semibold text-gray-900 text-sm truncate">
          {{ board.title }}
        </h3>
        <p class="text-xs text-gray-500">
          b/{{ board.name }} &middot; {{ board.subscribers }} members
        </p>
      </div>
    </div>
    <p v-if="board.description" class="mt-2 text-xs text-gray-600 line-clamp-2">
      {{ board.description }}
    </p>
  </NuxtLink>
</template>
