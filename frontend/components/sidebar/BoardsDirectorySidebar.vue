<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'

const authStore = useAuthStore()
const siteStore = useSiteStore()
</script>

<template>
  <div class="space-y-5">
    <!-- Boards directory info -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-emerald-500 to-teal-600" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <svg class="w-5 h-5 text-emerald-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">Board Directory</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Discover communities on {{ siteStore.name }}. Find boards that match your interests.
        </p>
      </div>
    </div>

    <!-- Your boards -->
    <div v-if="authStore.isLoggedIn && authStore.subscribedBoards?.length > 0">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Your Boards
      </h4>
      <ul class="space-y-0.5">
        <li v-for="board in authStore.subscribedBoards" :key="board.name">
          <NuxtLink
            :to="`/b/${board.name}`"
            class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-700 rounded-md hover:bg-gray-100 no-underline transition-colors"
          >
            <CommonAvatar
              :src="board.icon ?? undefined"
              :name="board.name"
              size="xs"
            />
            <span class="truncate flex-1">{{ board.title }}</span>
          </NuxtLink>
        </li>
      </ul>
    </div>

    <!-- Create board CTA -->
    <div v-if="authStore.isLoggedIn && !siteStore.boardCreationAdminOnly">
      <NuxtLink
        to="/boards/create"
        class="button white w-full text-center no-underline flex items-center justify-center gap-2 text-sm"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Create a Board
      </NuxtLink>
    </div>
  </div>
</template>
