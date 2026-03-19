<script setup lang="ts">
import type { Board } from '~/types/generated'
import { formatDate } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'

defineProps<{
  board: Board
}>()
</script>

<template>
  <div class="space-y-6">
    <!-- About -->
    <div>
      <h3 class="font-bold leading-5 text-base text-gray-900 mb-3 pb-1 border-b">
        About b/{{ board.name }}
      </h3>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-if="board.sidebarHTML" class="prose prose-sm text-gray-900" v-html="sanitizeHtml(board.sidebarHTML)" />

      <dl class="mt-4 space-y-2 text-xs text-gray-600">
        <div class="flex justify-between">
          <dt>Members</dt>
          <dd class="font-medium text-gray-700">{{ board.subscribers }}</dd>
        </div>
        <div class="flex justify-between">
          <dt>Posts</dt>
          <dd class="font-medium text-gray-700">{{ board.posts }}</dd>
        </div>
        <div class="flex justify-between">
          <dt>Created</dt>
          <dd class="font-medium text-gray-700">{{ formatDate(board.createdAt) }}</dd>
        </div>
        <div class="flex justify-between">
          <dt>Online now</dt>
          <dd class="font-medium text-gray-700">{{ board.usersActiveDay }}</dd>
        </div>
      </dl>
    </div>
  </div>
</template>
