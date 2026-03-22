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

interface TrendingResponse {
  listBoards: Board[]
}

const { execute, loading } = useGraphQL<TrendingResponse>()
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
    <!-- Site welcome card -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-primary to-primary-hover" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <img
            v-if="siteStore.icon"
            :src="siteStore.icon"
            :alt="siteStore.name"
            class="w-7 h-7"
          >
          <svg v-else class="w-5 h-5 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">Home</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Your personal front page. Posts from boards you've joined appear here.
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
            <span class="text-[10px] text-gray-400 tabular-nums">{{ board.subscribers }}</span>
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
              <span class="text-[10px] text-gray-400">{{ board.usersActiveDay ?? 0 }} active today</span>
            </div>
          </NuxtLink>
        </li>
      </ol>
    </div>

    <!-- Quick nav -->
    <div>
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Explore
      </h4>
      <ul class="space-y-0.5">
        <li>
          <NuxtLink to="/boards" class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-600 rounded-md hover:bg-gray-100 hover:text-gray-900 no-underline transition-colors">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
            </svg>
            All Boards
          </NuxtLink>
        </li>
        <li>
          <NuxtLink to="/all" class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-600 rounded-md hover:bg-gray-100 hover:text-gray-900 no-underline transition-colors">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            All Posts
          </NuxtLink>
        </li>
        <li>
          <NuxtLink to="/members" class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-600 rounded-md hover:bg-gray-100 hover:text-gray-900 no-underline transition-colors">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
            </svg>
            Members
          </NuxtLink>
        </li>
      </ul>
    </div>
  </div>
</template>
