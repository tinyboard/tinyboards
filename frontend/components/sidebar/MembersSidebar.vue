<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'

const authStore = useAuthStore()
</script>

<template>
  <div class="space-y-5">
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
