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

const SITE_STATS_QUERY = `
  query { siteStats { users posts comments boards usersActiveDay } }
`

interface TrendingResponse {
  listBoards: Board[]
}

interface SiteStatsResponse {
  siteStats: { users: number; posts: number; comments: number; boards: number; usersActiveDay: number }
}

const { execute, loading } = useGraphQL<TrendingResponse>()
const trendingBoards = ref<Board[]>([])
const siteStats = ref<SiteStatsResponse['siteStats'] | null>(null)

async function fetchTrending (): Promise<void> {
  const result = await execute(TRENDING_BOARDS_QUERY, {
    variables: { limit: 5, sort: 'active' },
  })
  if (result?.listBoards) {
    trendingBoards.value = result.listBoards
  }
}

async function fetchStats (): Promise<void> {
  const { execute: execStats } = useGraphQL<SiteStatsResponse>()
  const result = await execStats(SITE_STATS_QUERY)
  if (result?.siteStats) {
    siteStats.value = result.siteStats
  }
}

await Promise.all([fetchTrending(), fetchStats()])

function formatCount (n: number): string {
  if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`
  return String(n)
}
</script>

<template>
  <div class="space-y-4">
    <!-- Site identity card -->
    <div class="bg-white rounded-lg border border-gray-200 p-4">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
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
        <h3 class="font-semibold text-sm text-gray-900">{{ siteStore.name || 'TinyBoards' }}</h3>
      </div>
      <p v-if="siteStore.description" class="text-xs text-gray-500 leading-relaxed">
        {{ siteStore.description }}
      </p>
      <div v-if="!authStore.isLoggedIn" class="flex items-center gap-2 mt-3">
        <NuxtLink to="/register" class="button button-sm primary no-underline flex-1 text-center">
          Sign Up
        </NuxtLink>
        <NuxtLink to="/login" class="button button-sm white no-underline flex-1 text-center">
          Log In
        </NuxtLink>
      </div>
    </div>

    <!-- Site Stats -->
    <div v-if="siteStats" class="bg-white rounded-lg border border-gray-200 p-3">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
        Site Stats
      </h4>
      <div class="grid grid-cols-2 gap-2">
        <div class="text-center py-1">
          <div class="text-lg font-semibold text-gray-900">{{ formatCount(siteStats.users) }}</div>
          <div class="text-xs text-gray-500">Members</div>
        </div>
        <div class="text-center py-1">
          <div class="text-lg font-semibold text-gray-900">{{ formatCount(siteStats.posts) }}</div>
          <div class="text-xs text-gray-500">Posts</div>
        </div>
        <div class="text-center py-1">
          <div class="text-lg font-semibold text-gray-900">{{ formatCount(siteStats.comments) }}</div>
          <div class="text-xs text-gray-500">Comments</div>
        </div>
        <div class="text-center py-1">
          <div class="text-lg font-semibold text-gray-900">{{ formatCount(siteStats.boards) }}</div>
          <div class="text-xs text-gray-500">Boards</div>
        </div>
      </div>
      <div v-if="siteStats.usersActiveDay > 0" class="mt-2 pt-2 border-t border-gray-100 flex items-center justify-center gap-1.5 text-xs text-gray-500">
        <span class="inline-block w-1.5 h-1.5 rounded-full bg-green-400" />
        {{ siteStats.usersActiveDay }} online now
      </div>
    </div>

    <!-- Your boards -->
    <div v-if="authStore.isLoggedIn && authStore.subscribedBoards?.length > 0">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Your Boards
      </h4>
      <ul class="space-y-0.5">
        <li v-for="board in authStore.subscribedBoards.slice(0, 8)" :key="board.name">
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
      <NuxtLink
        v-if="authStore.subscribedBoards.length > 8"
        to="/boards"
        class="block text-xs text-primary hover:underline mt-1 px-2 no-underline"
      >
        View all boards
      </NuxtLink>
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

    <!-- Footer links -->
    <div class="border-t border-gray-200 pt-3">
      <p class="text-[10px] text-gray-400 leading-relaxed">
        Powered by TinyBoards &mdash; a self-hosted community platform.
      </p>
    </div>
  </div>
</template>
