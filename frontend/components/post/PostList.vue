<script setup lang="ts">
import type { Post } from '~/types/generated'
import { useUIStore } from '~/stores/ui'

defineProps<{
  posts: Post[]
  loading: boolean
}>()

const uiStore = useUIStore()
const isCompact = computed(() => uiStore.postViewMode === 'compact')
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="loading && posts.length === 0" size="lg" />

    <div v-else-if="posts.length === 0" class="bg-white rounded-lg border border-gray-200 py-16 text-center">
      <div class="inline-flex w-16 h-16 rounded-2xl bg-gray-100 items-center justify-center mb-4">
        <svg class="w-8 h-8 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
        </svg>
      </div>
      <p class="text-sm font-medium text-gray-600 mb-1">No posts yet</p>
      <p class="text-xs text-gray-400">Be the first to start a conversation!</p>
    </div>

    <div v-else :class="isCompact ? 'space-y-1' : 'space-y-3'" class="relative">
      <div v-if="loading" class="absolute inset-0 bg-white/50 z-10 flex items-start justify-center pt-8 rounded-lg">
        <CommonLoadingSpinner size="md" />
      </div>
      <PostCard
        v-for="(post, index) in posts"
        :key="post.id"
        :post="post"
        :compact="isCompact"
        class="post-card-enter"
        :style="{ animationDelay: `${Math.min(index * 30, 300)}ms` }"
      />
    </div>
  </div>
</template>
