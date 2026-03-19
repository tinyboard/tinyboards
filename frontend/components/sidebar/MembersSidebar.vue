<script setup lang="ts">
import { useSiteStore } from '~/stores/site'
import { useAuthStore } from '~/stores/auth'

const siteStore = useSiteStore()
const authStore = useAuthStore()
</script>

<template>
  <div class="space-y-5">
    <!-- Members info -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-cyan-500 to-blue-600" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <svg class="w-5 h-5 text-cyan-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">Community</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Members of {{ siteStore.name }}. Search to find people and view their profiles.
        </p>
      </div>
    </div>

    <!-- Your boards for context -->
    <div v-if="authStore.isLoggedIn && authStore.subscribedBoards?.length > 0">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Your Boards
      </h4>
      <ul class="space-y-0.5">
        <li v-for="board in authStore.subscribedBoards.slice(0, 5)" :key="board.name">
          <NuxtLink
            :to="`/b/${board.name}/members`"
            class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-700 rounded-md hover:bg-gray-100 no-underline transition-colors"
          >
            <CommonAvatar
              :src="board.icon ?? undefined"
              :name="board.name"
              size="xs"
            />
            <span class="truncate flex-1">{{ board.title }} members</span>
          </NuxtLink>
        </li>
      </ul>
    </div>
  </div>
</template>
