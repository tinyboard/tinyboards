<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Board } from '~/types/generated'

const authStore = useAuthStore()
const siteStore = useSiteStore()

const TRENDING_BOARDS_QUERY = `
  query TrendingBoards($limit: Int, $sort: SortType) {
    listBoards(limit: $limit, sort: $sort) {
      id
      name
      title
      icon
      subscribers
      usersActiveDay
    }
  }
`

const { execute } = useGraphQL<{ listBoards: Board[] }>()
const trendingBoards = ref<Board[]>([])

async function fetchTrending (): Promise<void> {
  const result = await execute(TRENDING_BOARDS_QUERY, {
    variables: { limit: 5, sort: 'active' },
  })
  if (result?.listBoards) {
    trendingBoards.value = result.listBoards
  }
}

await fetchTrending()
</script>

<template>
  <div class="space-y-5">
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

    <!-- Trending boards -->
    <div v-if="trendingBoards.length > 0">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Trending
      </h4>
      <ol class="space-y-0.5">
        <li v-for="(board, i) in trendingBoards" :key="board.id">
          <NuxtLink
            :to="`/b/${board.name}`"
            class="flex items-center gap-2.5 px-2 py-1.5 rounded-md hover:bg-gray-100 no-underline transition-colors group"
          >
            <span class="text-[10px] font-bold text-gray-300 w-3 text-right">{{ i + 1 }}</span>
            <CommonAvatar
              :src="board.icon ?? undefined"
              :name="board.name"
              size="xs"
            />
            <div class="flex-1 min-w-0">
              <span class="text-sm text-gray-700 group-hover:text-gray-900 truncate block">{{ board.title }}</span>
              <span class="text-[10px] text-gray-400">{{ board.subscribers }} members</span>
            </div>
          </NuxtLink>
        </li>
      </ol>
    </div>

    <!-- Create board CTA -->
    <div v-if="authStore.isLoggedIn && (!siteStore.boardCreationAdminOnly || authStore.isAdmin)">
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
