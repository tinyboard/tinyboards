<script setup lang="ts">
import type { Board } from '~/types/generated'

defineProps<{
  board: Board & { mode?: string }
}>()
</script>

<template>
  <!-- Forum mode: list row with activity stats -->
  <NuxtLink
    v-if="board.mode === 'forum'"
    :to="`/b/${board.name}`"
    class="flex items-center gap-3 bg-white border border-gray-200 rounded px-4 py-3 hover:bg-gray-50 transition-colors no-underline"
  >
    <CommonAvatar :src="board.icon ?? undefined" :name="board.name" size="sm" />
    <div class="flex-1 min-w-0">
      <h3 class="font-semibold text-gray-900 text-sm truncate">{{ board.title }}</h3>
      <p v-if="board.description" class="text-xs text-gray-500 truncate mt-0.5">{{ board.description }}</p>
    </div>
    <div class="hidden sm:flex items-center gap-4 text-xs text-gray-400 shrink-0">
      <span title="Discussions">{{ board.posts ?? 0 }} threads</span>
      <span title="Replies">{{ board.comments ?? 0 }} replies</span>
      <span title="Members">{{ board.subscribers }} members</span>
    </div>
  </NuxtLink>

  <!-- Feed mode: visual card -->
  <NuxtLink
    v-else
    :to="`/b/${board.name}`"
    class="block bg-white border border-gray-200 rounded overflow-hidden hover:border-gray-300 transition-colors no-underline"
  >
    <div v-if="board.banner" class="h-20 bg-gray-100 overflow-hidden">
      <img :src="board.banner" :alt="board.title" class="w-full h-full object-cover" />
    </div>
    <div class="p-4">
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
    </div>
  </NuxtLink>
</template>
