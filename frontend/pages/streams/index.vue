<script setup lang="ts">
import { useStreams, type StreamData } from '~/composables/useStreams'
import { useAuthStore } from '~/stores/auth'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Discover Streams' })

const authStore = useAuthStore()
const { streams, loading, error, fetchDiscoverStreams, fetchMyStreams, fetchFollowedStreams, searchStreams } = useStreams()

type Tab = 'discover' | 'my' | 'following'
const activeTab = ref<Tab>('discover')
const searchQuery = ref('')
const sortBy = ref<string>('Popular')

function formatCount (count: number | null | undefined): string {
  if (!count) return '0'
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K`
  return count.toString()
}

async function loadTab () {
  if (activeTab.value === 'discover') {
    await fetchDiscoverStreams(sortBy.value, 50)
  } else if (activeTab.value === 'my') {
    await fetchMyStreams()
  } else if (activeTab.value === 'following') {
    await fetchFollowedStreams()
  }
}

async function handleSearch () {
  if (!searchQuery.value.trim()) {
    await loadTab()
    return
  }
  await searchStreams(searchQuery.value.trim())
}

async function switchTab (tab: Tab) {
  activeTab.value = tab
  searchQuery.value = ''
  await loadTab()
}

await loadTab()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-lg font-semibold text-gray-900">Streams</h1>
        <p class="text-sm text-gray-600 mt-1">
          Custom feeds that combine multiple boards into a single timeline.
        </p>
      </div>
      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/streams/create"
        class="button primary"
      >
        Create Stream
      </NuxtLink>
    </div>

    <!-- Tabs -->
    <div class="flex gap-1 mb-4 border-b border-gray-200">
      <button
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
        :class="activeTab === 'discover'
          ? 'border-primary text-primary'
          : 'border-transparent text-gray-500 hover:text-gray-700'"
        @click="switchTab('discover')"
      >
        Discover
      </button>
      <button
        v-if="authStore.isLoggedIn"
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
        :class="activeTab === 'my'
          ? 'border-primary text-primary'
          : 'border-transparent text-gray-500 hover:text-gray-700'"
        @click="switchTab('my')"
      >
        My Streams
      </button>
      <button
        v-if="authStore.isLoggedIn"
        class="px-4 py-2 text-sm font-medium transition-colors border-b-2 -mb-px"
        :class="activeTab === 'following'
          ? 'border-primary text-primary'
          : 'border-transparent text-gray-500 hover:text-gray-700'"
        @click="switchTab('following')"
      >
        Following
      </button>
    </div>

    <!-- Search and sort -->
    <div class="flex gap-3 mb-4">
      <div class="flex-1">
        <input
          v-model="searchQuery"
          type="text"
          class="form-input w-full"
          placeholder="Search streams..."
          @keyup.enter="handleSearch"
        />
      </div>
      <select
        v-if="activeTab === 'discover'"
        v-model="sortBy"
        class="form-input w-auto"
        @change="loadTab"
      >
        <option value="Popular">Popular</option>
        <option value="New">New</option>
        <option value="Trending">Trending</option>
      </select>
    </div>

    <!-- Stream list -->
    <CommonErrorDisplay v-if="error" :message="error.message" @retry="loadTab" />
    <CommonLoadingSpinner v-else-if="loading && streams.length === 0" size="lg" />

    <div v-else-if="streams.length > 0" class="space-y-3">
      <NuxtLink
        v-for="s in streams"
        :key="s.id"
        :to="s.creator ? `/streams/@${s.creator.name}/${s.slug}` : `/streams/${s.id}`"
        class="block bg-white border border-gray-200 rounded-lg p-4 hover:border-gray-300 transition-colors no-underline"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-semibold text-gray-900 truncate">
                {{ s.name }}
              </h3>
              <span v-if="!s.isPublic" class="text-xs bg-gray-100 text-gray-500 px-1.5 py-0.5 rounded">
                Private
              </span>
            </div>
            <p v-if="s.creator" class="text-xs text-gray-500 mt-0.5">
              by @{{ s.creator.name }}
            </p>
            <p v-if="s.description" class="text-sm text-gray-600 mt-1 line-clamp-2">
              {{ s.description }}
            </p>

            <!-- Subscribed boards preview -->
            <div v-if="s.boardSubscriptions && s.boardSubscriptions.length > 0" class="flex flex-wrap gap-1 mt-2">
              <span
                v-for="sub in s.boardSubscriptions.slice(0, 5)"
                :key="sub.boardId"
                class="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-600"
              >
                +{{ sub.board?.name ?? 'unknown' }}
              </span>
              <span
                v-if="s.boardSubscriptions.length > 5"
                class="inline-flex items-center rounded-full bg-gray-100 px-2 py-0.5 text-xs text-gray-500"
              >
                +{{ s.boardSubscriptions.length - 5 }} more
              </span>
            </div>
          </div>

          <!-- Stats -->
          <div class="flex items-center gap-4 text-xs text-gray-400 shrink-0 ml-4">
            <span v-if="s.followerCount != null" :title="`${s.followerCount} followers`">
              {{ formatCount(s.followerCount) }} followers
            </span>
            <span v-if="s.boardSubscriptionCount != null">
              {{ s.boardSubscriptionCount }} boards
            </span>
          </div>
        </div>
      </NuxtLink>
    </div>

    <div v-else class="text-center py-12">
      <p class="text-sm text-gray-500 mb-4">
        <template v-if="activeTab === 'discover'">
          No public streams to discover yet.
        </template>
        <template v-else-if="activeTab === 'my'">
          You haven't created any streams yet.
        </template>
        <template v-else>
          You aren't following any streams yet.
        </template>
      </p>
      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/streams/create"
        class="text-sm text-primary hover:underline"
      >
        Create your first stream
      </NuxtLink>
    </div>
  </div>
</template>
