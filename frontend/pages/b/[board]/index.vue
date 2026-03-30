<script setup lang="ts">
import { usePosts } from '~/composables/usePosts'
import { useBoard } from '~/composables/useBoard'

const route = useRoute()
const boardName = route.params.board as string

const { board } = useBoard()

// Determine post type filter based on board mode
const boardMode = computed(() => board.value?.mode ?? 'feed')
const postTypeFilter = computed<'feed' | 'thread' | undefined>(() => {
  if (boardMode.value === 'forum') return 'thread'
  return 'feed'
})

const { posts, loading, error, page, sort, hasMore, fetchPosts, nextPage, prevPage, setSort } = usePosts({
  boardName,
  postType: postTypeFilter.value,
})

await fetchPosts()
</script>

<template>
  <div>
    <!-- Forum mode: thread-style list -->
    <template v-if="boardMode === 'forum'">
      <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
        <CommonSortSelector v-model="sort" @update:model-value="setSort" />
        <NuxtLink
          :to="`/b/${boardName}/submit?type=thread`"
          class="button button-sm primary no-underline"
        >
          New Discussion
        </NuxtLink>
      </div>

      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />

      <div v-if="!loading && posts.length === 0" class="bg-white border border-gray-200 rounded-lg py-12 text-center">
        <div class="inline-flex w-12 h-12 rounded-full bg-primary/10 items-center justify-center mb-3">
          <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
          </svg>
        </div>
        <p class="text-sm font-medium text-gray-600 mb-1">No discussions yet</p>
        <p class="text-xs text-gray-400">Start a discussion!</p>
      </div>

      <div v-else-if="!loading" class="bg-white border border-gray-200 rounded-lg overflow-hidden divide-y divide-gray-100">
        <NuxtLink
          v-for="thread in posts"
          :key="thread.id"
          :to="`/b/${boardName}/${thread.id}/${thread.slug || ''}`"
          class="flex items-center gap-4 px-4 py-3 hover:bg-gray-50 transition-colors no-underline group"
        >
          <div class="shrink-0">
            <svg v-if="thread.isLocked" class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
            </svg>
            <svg v-else class="w-5 h-5 text-gray-300 group-hover:text-primary transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span
                v-if="thread.isFeaturedBoard"
                class="text-[10px] font-bold uppercase tracking-wider text-primary bg-primary/10 px-1.5 py-0.5 rounded"
              >
                Pinned
              </span>
              <h3 class="text-sm font-medium text-gray-900 group-hover:text-primary truncate transition-colors">
                {{ thread.title }}
              </h3>
            </div>
            <p class="text-xs text-gray-500 mt-0.5">
              by {{ thread.creator?.displayName || thread.creator?.name || 'unknown' }}
            </p>
          </div>
          <div class="shrink-0 text-center min-w-[60px]">
            <div class="text-sm font-semibold text-gray-700">{{ thread.commentCount }}</div>
            <div class="text-[10px] text-gray-400 uppercase tracking-wider">
              {{ thread.commentCount === 1 ? 'Reply' : 'Replies' }}
            </div>
          </div>
        </NuxtLink>
      </div>

      <CommonLoadingSpinner v-if="loading" />
    </template>

    <!-- Feed mode: standard post list -->
    <template v-else>
      <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
        <CommonSortSelector v-model="sort" @update:model-value="setSort" />
        <CommonViewToggle />
      </div>

      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />
      <PostList :posts="posts" :loading="loading" />
    </template>

    <CommonPagination v-if="posts.length > 0" :page="page" :has-more="hasMore" @prev="prevPage" @next="nextPage" />
  </div>
</template>
