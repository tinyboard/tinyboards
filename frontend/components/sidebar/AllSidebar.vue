<script setup lang="ts">
import { useSiteStore } from '~/stores/site'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Board } from '~/types/generated'

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

const { execute } = useGraphQL<TrendingResponse>()
const trendingBoards = ref<Board[]>([])

async function fetchTrending (): Promise<void> {
  const result = await execute(TRENDING_BOARDS_QUERY, {
    variables: { limit: 8, sort: 'active' },
  })
  if (result?.listBoards) {
    trendingBoards.value = result.listBoards
  }
}

await fetchTrending()
</script>

<template>
  <div class="space-y-5">
    <!-- All posts info card -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-blue-500 to-indigo-600" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">All Posts</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Everything happening across {{ siteStore.name }}. Posts from every board in one feed.
        </p>
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

    <!-- Quick links -->
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
            Browse Boards
          </NuxtLink>
        </li>
        <li>
          <NuxtLink to="/streams" class="flex items-center gap-2.5 px-2 py-1.5 text-sm text-gray-600 rounded-md hover:bg-gray-100 hover:text-gray-900 no-underline transition-colors">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            Custom Streams
          </NuxtLink>
        </li>
      </ul>
    </div>
  </div>
</template>
