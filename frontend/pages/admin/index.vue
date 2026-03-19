<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin Dashboard' })

interface SiteStats {
  users: number
  posts: number
  comments: number
  boards: number
  usersActiveDay: number
  usersActiveWeek: number
  usersActiveMonth: number
  usersActiveHalfYear: number
}

interface SiteStatsResponse {
  siteStats: SiteStats
}

const { execute, loading, error, data } = useGraphQL<SiteStatsResponse>()

const STATS_QUERY = `
  query {
    siteStats {
      users
      posts
      comments
      boards
      usersActiveDay
      usersActiveWeek
      usersActiveMonth
      usersActiveHalfYear
    }
  }
`

onMounted(async () => {
  await execute(STATS_QUERY)
})

const stats = computed(() => data.value?.siteStats ?? null)

const overviewCards = computed(() => {
  if (!stats.value) return []
  return [
    { label: 'Total Users', value: stats.value.users, icon: 'users' },
    { label: 'Total Posts', value: stats.value.posts, icon: 'posts' },
    { label: 'Total Comments', value: stats.value.comments, icon: 'comments' },
    { label: 'Total Boards', value: stats.value.boards, icon: 'boards' },
  ]
})

const activityCards = computed(() => {
  if (!stats.value) return []
  return [
    { label: 'Active Today', value: stats.value.usersActiveDay },
    { label: 'Active This Week', value: stats.value.usersActiveWeek },
    { label: 'Active This Month', value: stats.value.usersActiveMonth },
    { label: 'Active (6 Months)', value: stats.value.usersActiveHalfYear },
  ]
})
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Dashboard
    </h2>

    <div v-if="loading" class="text-sm text-gray-500">
      Loading site statistics...
    </div>

    <div v-else-if="error" class="rounded-md bg-red-50 p-4 text-sm text-red-700">
      Failed to load statistics: {{ error.message }}
    </div>

    <template v-else-if="stats">
      <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-3">
        Overview
      </h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-8">
        <div
          v-for="card in overviewCards"
          :key="card.label"
          class="bg-white rounded-lg border border-gray-200 p-5"
        >
          <p class="text-sm font-medium text-gray-500">{{ card.label }}</p>
          <p class="mt-1 text-2xl font-semibold text-gray-900">
            {{ card.value.toLocaleString() }}
          </p>
        </div>
      </div>

      <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-3">
        User Activity
      </h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div
          v-for="card in activityCards"
          :key="card.label"
          class="bg-white rounded-lg border border-gray-200 p-5"
        >
          <p class="text-sm font-medium text-gray-500">{{ card.label }}</p>
          <p class="mt-1 text-2xl font-semibold text-gray-900">
            {{ card.value.toLocaleString() }}
          </p>
        </div>
      </div>
    </template>
  </div>
</template>
