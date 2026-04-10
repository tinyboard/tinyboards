<script setup lang="ts">
import { useUser } from '~/composables/useUser'
import { formatDate } from '~/utils/date'

const route = useRoute()

const username = computed(() => {
  const params = route.params
  if (typeof params.username === 'string') return params.username
  return ''
})

const { user, loading, fetchUser } = useUser()

watch(username, async (name) => {
  if (name) await fetchUser(name)
}, { immediate: true })

function formatNumber (n: number): string {
  if (n >= 1000000) return `${(n / 1000000).toFixed(1)}M`
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`
  return n.toString()
}
</script>

<template>
  <div class="space-y-5">
    <template v-if="user && !loading">
      <!-- User card -->
      <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div
          class="h-16"
          :class="user.banner ? '' : 'bg-gradient-to-br from-violet-500 to-purple-600'"
        >
          <img
            v-if="user.banner"
            :src="user.banner"
            :alt="`${user.name} banner`"
            class="w-full h-full object-cover"
          >
        </div>
        <div class="px-4 py-3 -mt-6">
          <CommonAvatar
            :src="user.avatar ?? undefined"
            :name="user.name"
            size="lg"
            class="border-3 border-white shadow-sm mb-2"
          />
          <h3 class="font-semibold text-sm text-gray-900">{{ user.displayName ?? user.name }}</h3>
          <p class="text-xs text-gray-500">@{{ user.name }}</p>
        </div>
      </div>

      <!-- User stats -->
      <div class="bg-white rounded-lg border border-gray-200 p-4">
        <h4 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Activity
        </h4>
        <div class="grid grid-cols-2 gap-3 text-center sm:text-left">
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ formatNumber(user.postCount ?? 0) }}</div>
            <div class="text-[11px] text-gray-500">Posts</div>
          </div>
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ formatNumber(user.commentCount ?? 0) }}</div>
            <div class="text-[11px] text-gray-500">Comments</div>
          </div>
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ formatNumber(user.postScore ?? 0) }}</div>
            <div class="text-[11px] text-gray-500">Post Karma</div>
          </div>
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ formatNumber(user.commentScore ?? 0) }}</div>
            <div class="text-[11px] text-gray-500">Comment Karma</div>
          </div>
        </div>
        <div class="mt-3 pt-3 border-t border-gray-100 text-xs text-gray-500 flex items-center gap-1.5">
          <svg class="w-3.5 h-3.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          Joined {{ formatDate(user.createdAt) }}
        </div>
      </div>
    </template>

    <!-- Loading -->
    <div v-else-if="loading" class="space-y-4">
      <div class="bg-white rounded-lg border border-gray-200 h-32 animate-pulse" />
      <div class="bg-white rounded-lg border border-gray-200 h-24 animate-pulse" />
    </div>
  </div>
</template>
