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
  >
    <div class="flex items-center gap-3">
      <CommonAvatar :src="board.icon ?? undefined" :name="board.name" size="md" />
      <div class="flex-1 min-w-0">
        <h3 class="font-semibold text-gray-900 text-sm truncate flex items-center gap-1.5">
          {{ board.title }}
          <span
            v-if="board.mode"
            class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0"
            :class="board.mode === 'forum' ? 'bg-purple-100 text-purple-700' : 'bg-blue-100 text-blue-700'"
          >
            {{ board.mode === 'forum' ? '💬 Forum' : '📰 Feed' }}
          </span>
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
