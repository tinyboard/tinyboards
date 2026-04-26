<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Board } from '~/types/generated'

const authStore = useAuthStore()

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

const { execute } = useGraphQL<TrendingResponse>()
const trendingBoards = ref<Board[]>([])
const siteStats = ref<SiteStatsResponse['siteStats'] | null>(null)

async function fetchTrending (): Promise<void> {
  const result = await execute(TRENDING_BOARDS_QUERY, {
    variables: { limit: 8, sort: 'active' },
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
    <!-- New Post button for logged-in users -->
    <NuxtLink
      v-if="authStore.isLoggedIn"
      to="/submit"
      class="flex items-center justify-center gap-2 w-full rounded-lg bg-primary text-white py-2.5 text-sm font-medium hover:opacity-90 transition-opacity no-underline"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Post
    </NuxtLink>

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

    <!-- Active boards -->
    <div v-if="trendingBoards.length > 0">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 px-1">
        Most Active
      </h4>
      <ul class="space-y-0.5">
        <li v-for="board in trendingBoards" :key="board.id">
          <NuxtLink
            :to="`/b/${board.name}`"
            class="flex items-center gap-2.5 px-2 py-1.5 rounded-md hover:bg-gray-100 no-underline transition-colors group"
          >
            <CommonAvatar
              :src="board.icon ?? undefined"
              :name="board.name"
              size="xs"
            />
            <div class="flex-1 min-w-0">
              <span class="text-sm text-gray-700 group-hover:text-gray-900 truncate block">{{ board.title }}</span>
            </div>
            <div class="flex items-center gap-1 text-[10px] text-gray-400">
              <span class="inline-block w-1.5 h-1.5 rounded-full bg-green-400" />
              {{ board.usersActiveDay ?? 0 }}
            </div>
          </NuxtLink>
        </li>
      </ul>
    </div>

    <!-- Footer links -->
    <div class="border-t border-gray-200 pt-3">
      <p class="text-[10px] text-gray-400 leading-relaxed">
        Powered by TinyBoards &mdash; a self-hosted community platform.
      </p>
    </div>
  </div>
</template>
