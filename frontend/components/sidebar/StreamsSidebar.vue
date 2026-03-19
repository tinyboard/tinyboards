<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useStreams } from '~/composables/useStreams'

const authStore = useAuthStore()
const { streams, loading, fetchMyStreams, fetchFollowedStreams } = useStreams()

const activeTab = ref<'my' | 'following'>('my')

async function loadSidebar () {
  if (!authStore.isLoggedIn) return
  if (activeTab.value === 'my') {
    await fetchMyStreams(10)
  } else {
    await fetchFollowedStreams(10)
  }
}

watch(activeTab, loadSidebar)

onMounted(loadSidebar)
</script>

<template>
  <div class="space-y-5">
    <!-- Streams info -->
    <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
      <div class="h-16 bg-gradient-to-br from-rose-400 to-pink-600" />
      <div class="px-4 py-3 -mt-4">
        <div class="w-10 h-10 rounded-lg bg-white shadow-sm border border-gray-100 flex items-center justify-center mb-2">
          <svg class="w-5 h-5 text-rose-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </div>
        <h3 class="font-semibold text-sm text-gray-900">Streams</h3>
        <p class="text-xs text-gray-500 mt-1 leading-relaxed">
          Create custom feeds by combining multiple boards into a single timeline.
        </p>
      </div>
    </div>

    <!-- Followed / My streams list -->
    <div v-if="authStore.isLoggedIn" class="bg-white rounded-lg border border-gray-200 p-4">
      <div class="flex gap-2 mb-3">
        <button
          class="text-xs font-medium px-2 py-1 rounded transition-colors"
          :class="activeTab === 'my' ? 'bg-gray-100 text-gray-900' : 'text-gray-500 hover:text-gray-700'"
          @click="activeTab = 'my'"
        >
          My Streams
        </button>
        <button
          class="text-xs font-medium px-2 py-1 rounded transition-colors"
          :class="activeTab === 'following' ? 'bg-gray-100 text-gray-900' : 'text-gray-500 hover:text-gray-700'"
          @click="activeTab = 'following'"
        >
          Following
        </button>
      </div>

      <CommonLoadingSpinner v-if="loading" size="sm" />
      <div v-else-if="streams.length > 0" class="space-y-1">
        <NuxtLink
          v-for="s in streams"
          :key="s.id"
          :to="s.creator ? `/streams/@${s.creator.name}/${s.slug}` : `/streams/${s.id}`"
          class="flex items-center gap-2 px-2 py-1.5 rounded text-sm text-gray-700 hover:bg-gray-50 no-underline transition-colors"
        >
          <svg class="w-3.5 h-3.5 text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          <span class="truncate">{{ s.name }}</span>
          <span v-if="s.boardSubscriptionCount" class="text-xs text-gray-400 shrink-0">{{ s.boardSubscriptionCount }}</span>
        </NuxtLink>
      </div>
      <p v-else class="text-xs text-gray-400">
        {{ activeTab === 'my' ? 'No streams created yet.' : 'Not following any streams.' }}
      </p>
    </div>

    <!-- How it works (for non-logged-in users) -->
    <div v-if="!authStore.isLoggedIn" class="bg-white rounded-lg border border-gray-200 p-4">
      <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
        How It Works
      </h4>
      <ol class="space-y-3 text-xs text-gray-600">
        <li class="flex items-start gap-2.5">
          <span class="inline-flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-bold shrink-0 mt-0.5">1</span>
          <span>Pick the boards you want to combine</span>
        </li>
        <li class="flex items-start gap-2.5">
          <span class="inline-flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-bold shrink-0 mt-0.5">2</span>
          <span>Give your stream a name</span>
        </li>
        <li class="flex items-start gap-2.5">
          <span class="inline-flex items-center justify-center w-4 h-4 rounded-full bg-primary/10 text-primary text-[10px] font-bold shrink-0 mt-0.5">3</span>
          <span>Browse all selected boards in one unified feed</span>
        </li>
      </ol>
    </div>

    <!-- Create stream CTA -->
    <NuxtLink
      v-if="authStore.isLoggedIn"
      to="/streams/create"
      class="button primary w-full text-center no-underline flex items-center justify-center gap-2 text-sm"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      Create Stream
    </NuxtLink>
  </div>
</template>
