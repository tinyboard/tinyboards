<script setup lang="ts">
import type { User } from '~/types/generated'

defineProps<{
  user: User
}>()
</script>

<template>
  <NuxtLink
    :to="`/@${user.name}`"
    class="flex items-center gap-3 p-3 bg-white border border-gray-200 rounded hover:border-gray-300 no-underline"
  >
    <CommonAvatar :src="user.avatar ?? undefined" :name="user.name" size="md" />
    <div class="flex-1 min-w-0">
      <p class="font-medium text-gray-900 text-sm truncate">
        {{ user.displayName ?? user.name }}
      </p>
      <p class="text-xs text-gray-500">
        @{{ user.name }} &middot; {{ user.postScore + user.commentScore }} score
      </p>
    </div>
  </NuxtLink>
</template>
