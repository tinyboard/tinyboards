<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'

const authStore = useAuthStore()
const siteStore = useSiteStore()
</script>

<template>
  <div class="space-y-6">
    <!-- Site info -->
    <div>
      <h3 class="font-bold leading-5 text-base text-gray-900 mb-3 pb-1 border-b">
        {{ siteStore.name }}
      </h3>
      <p v-if="siteStore.description" class="text-xs text-gray-500 mb-3">
        {{ siteStore.description }}
      </p>
      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/submit"
        class="button primary w-full text-center no-underline flex items-center justify-center"
      >
        <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Create Post
      </NuxtLink>
    </div>

    <!-- Subscribed boards (when in sidebar slot) -->
    <div v-if="authStore.isLoggedIn && authStore.subscribedBoards.length > 0">
      <h3 class="font-bold leading-5 text-base text-gray-900 mb-3 pb-1 border-b">
        My Boards
      </h3>
      <ul class="space-y-1">
        <li v-for="board in authStore.subscribedBoards" :key="board.name">
          <NuxtLink
            :to="`/b/${board.name}`"
            class="flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900 no-underline"
          >
            <CommonAvatar :src="board.icon ?? undefined" :name="board.name" size="xs" />
            <span class="truncate">{{ board.title }}</span>
            <span
              v-if="board.mode === 'forum'"
              class="text-[10px] text-purple-500 shrink-0"
              title="Forum"
            >💬</span>
            <span
              v-else-if="board.mode === 'feed'"
              class="text-[10px] text-blue-500 shrink-0"
              title="Feed"
            >📰</span>
            <span class="text-xs text-gray-400 ml-auto">{{ board.subscribers }}</span>
          </NuxtLink>
        </li>
      </ul>
    </div>
  </div>
</template>
