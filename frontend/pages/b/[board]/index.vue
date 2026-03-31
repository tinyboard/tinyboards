<script setup lang="ts">
import { usePosts } from '~/composables/usePosts'
import { useBoard } from '~/composables/useBoard'
import { timeAgo, formatDate } from '~/utils/date'

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

// Forum boards sort by activity (newest comment) by default
if (boardMode.value === 'forum') {
  sort.value = 'newComments'
}

const pinnedThreads = computed(() => posts.value.filter(p => p.isFeaturedBoard))
const unpinnedThreads = computed(() => posts.value.filter(p => !p.isFeaturedBoard))

await fetchPosts()
</script>

<template>
  <div>
    <!-- Forum mode: classic forum thread list -->
    <template v-if="boardMode === 'forum'">
      <div class="flex items-center justify-end mb-4">
        <NuxtLink
          :to="`/b/${boardName}/submit?type=thread`"
          class="button button-sm primary no-underline"
        >
          New Discussion
        </NuxtLink>
      </div>

      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />

      <div v-if="!loading && posts.length === 0" class="bg-white border border-gray-200 rounded py-12 text-center">
        <div class="inline-flex w-12 h-12 rounded-full bg-primary/10 items-center justify-center mb-3">
          <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
          </svg>
        </div>
        <p class="text-sm font-medium text-gray-600 mb-1">No discussions yet</p>
        <p class="text-xs text-gray-400">Start a discussion!</p>
      </div>

      <div v-else-if="!loading" class="forum-thread-list">
        <!-- Table header -->
        <div class="forum-header">
          <div class="forum-header-topic">Topic</div>
          <div class="forum-header-stats">Replies</div>
          <div class="forum-header-activity">Last Post</div>
        </div>

        <!-- Pinned threads -->
        <template v-if="pinnedThreads.length > 0">
          <NuxtLink
            v-for="thread in pinnedThreads"
            :key="thread.id"
            :to="`/b/${boardName}/${thread.id}/${thread.slug || ''}`"
            class="forum-thread forum-thread-pinned no-underline"
          >
            <div class="forum-thread-avatar">
              <CommonAvatar
                :src="thread.creator?.avatar ?? undefined"
                :name="thread.creator?.displayName || thread.creator?.name || '?'"
                size="lg"
              />
            </div>
            <div class="forum-thread-content">
              <div class="forum-thread-title-row">
                <span class="forum-pin-badge">Pinned</span>
                <span v-if="thread.isLocked" class="forum-lock-badge" title="Locked">
                  <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
                </span>
                <h3 class="forum-thread-title">{{ thread.title }}</h3>
              </div>
              <p class="forum-thread-meta">
                by <span class="forum-thread-author">{{ thread.creator?.displayName || thread.creator?.name || 'unknown' }}</span>
                &middot;
                <time :datetime="thread.createdAt" :title="thread.createdAt">{{ formatDate(thread.createdAt) }}</time>
              </p>
            </div>
            <div class="forum-thread-stats">
              <span class="forum-stat-number">{{ thread.commentCount }}</span>
              <span class="forum-stat-label">{{ thread.commentCount === 1 ? 'reply' : 'replies' }}</span>
            </div>
            <div class="forum-thread-last-post">
              <template v-if="thread.newestCommentTime && thread.commentCount > 0">
                <span class="forum-last-post-time">{{ timeAgo(thread.newestCommentTime) }}</span>
              </template>
              <span v-else class="forum-last-post-time">&mdash;</span>
            </div>
          </NuxtLink>
        </template>

        <!-- Regular threads -->
        <NuxtLink
          v-for="thread in unpinnedThreads"
          :key="thread.id"
          :to="`/b/${boardName}/${thread.id}/${thread.slug || ''}`"
          class="forum-thread no-underline"
        >
          <div class="forum-thread-avatar">
            <CommonAvatar
              :src="thread.creator?.avatar ?? undefined"
              :name="thread.creator?.displayName || thread.creator?.name || '?'"
              size="lg"
            />
          </div>
          <div class="forum-thread-content">
            <div class="forum-thread-title-row">
              <span v-if="thread.isLocked" class="forum-lock-badge" title="Locked">
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
              </span>
              <h3 class="forum-thread-title">{{ thread.title }}</h3>
            </div>
            <p class="forum-thread-meta">
              by <span class="forum-thread-author">{{ thread.creator?.displayName || thread.creator?.name || 'unknown' }}</span>
              &middot;
              <time :datetime="thread.createdAt" :title="thread.createdAt">{{ formatDate(thread.createdAt) }}</time>
            </p>
          </div>
          <div class="forum-thread-stats">
            <span class="forum-stat-number">{{ thread.commentCount }}</span>
            <span class="forum-stat-label">{{ thread.commentCount === 1 ? 'reply' : 'replies' }}</span>
          </div>
          <div class="forum-thread-last-post">
            <template v-if="thread.newestCommentTime && thread.commentCount > 0">
              <span class="forum-last-post-time">{{ timeAgo(thread.newestCommentTime) }}</span>
            </template>
            <span v-else class="forum-last-post-time">&mdash;</span>
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
