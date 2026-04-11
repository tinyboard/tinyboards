<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import { usePosts } from '~/composables/usePosts'
import { timeAgo, formatDate } from '~/utils/date'
import type { ListingType, Post } from '~/types/generated'

const authStore = useAuthStore()
const siteStore = useSiteStore()

useHead({ title: 'Home' })
useSeoMeta({
  title: computed(() => `Home | ${siteStore.name || 'TinyBoards'}`),
  ogTitle: computed(() => `Home | ${siteStore.name || 'TinyBoards'}`),
  description: computed(() => siteStore.description || 'A community-driven discussion platform.'),
  ogDescription: computed(() => siteStore.description || 'A community-driven discussion platform.'),
  ogImage: computed(() => siteStore.icon || undefined),
  ogType: 'website',
})
const route = useRoute()

// Determine which tabs to show based on subscribed board modes
const hasFeedBoards = computed(() =>
  authStore.subscribedBoards.some(b => !b.mode || b.mode === 'feed'),
)
const hasForumBoards = computed(() =>
  authStore.subscribedBoards.some(b => b.mode === 'forum'),
)
const showTabs = computed(() => authStore.isLoggedIn && hasFeedBoards.value && hasForumBoards.value)

// Default tab: feed if subscribed to feed boards, otherwise threads
const activeTab = ref<'feed' | 'threads'>(
  hasFeedBoards.value ? 'feed' : (hasForumBoards.value ? 'threads' : 'feed'),
)

// Feed posts composable
const feedPosts = usePosts({
  listingType: (authStore.isLoggedIn ? 'subscribed' : 'all') as ListingType,
  postType: authStore.isLoggedIn && hasForumBoards.value ? 'feed' : undefined,
  basePath: '/home',
})

// Thread posts composable (only used when logged in with forum boards)
const threadPosts = authStore.isLoggedIn && hasForumBoards.value
  ? usePosts({
      listingType: 'subscribed' as ListingType,
      postType: 'thread',
    })
  : null

// Group threads by board for the threads tab
interface BoardGroup {
  boardName: string
  boardTitle: string
  boardIcon: string | null
  threads: Post[]
}

const threadsByBoard = computed<BoardGroup[]>(() => {
  if (!threadPosts) return []
  const map = new Map<string, BoardGroup>()
  for (const post of threadPosts.posts.value) {
    const key = post.board?.name ?? 'unknown'
    if (!map.has(key)) {
      map.set(key, {
        boardName: key,
        boardTitle: post.board?.title ?? key,
        boardIcon: post.board?.icon ?? null,
        threads: [],
      })
    }
    map.get(key)!.threads.push(post)
  }
  // Sort pinned threads to the top within each board group
  for (const group of map.values()) {
    group.threads.sort((a, b) => (b.isFeaturedBoard ? 1 : 0) - (a.isFeaturedBoard ? 1 : 0))
  }
  return Array.from(map.values())
})

// Initialize sort from URL param
if (route.params.sort && typeof route.params.sort === 'string') {
  feedPosts.sort.value = route.params.sort
}

// Fetch initial data
if (activeTab.value === 'feed' || !authStore.isLoggedIn) {
  await feedPosts.fetchPosts()
} else if (threadPosts) {
  await threadPosts.fetchPosts()
}

async function switchTab (tab: 'feed' | 'threads'): Promise<void> {
  activeTab.value = tab
  if (tab === 'feed' && feedPosts.posts.value.length === 0) {
    await feedPosts.fetchPosts()
  } else if (tab === 'threads' && threadPosts && threadPosts.posts.value.length === 0) {
    threadPosts.sort.value = 'newComments'
    await threadPosts.fetchPosts()
  }
}
</script>

<template>
  <div>
    <!-- Welcome banner for anonymous users -->
    <div v-if="!authStore.isLoggedIn" class="pt-4">
      <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div class="h-24 bg-gradient-to-br from-primary to-primary-hover" />
        <div class="px-6 py-4 -mt-6">
          <div class="w-12 h-12 rounded-xl bg-white shadow-md flex items-center justify-center border border-gray-100 mb-3">
            <img
              v-if="siteStore.icon"
              :src="siteStore.icon"
              class="w-8 h-8"
              :alt="siteStore.name"
            >
            <svg v-else class="w-7 h-7 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
            </svg>
          </div>
          <h1 class="text-xl font-bold text-gray-900 mb-1">
            Welcome to {{ siteStore.name || 'TinyBoards' }}
          </h1>
          <p class="text-sm text-gray-600 mb-3 max-w-lg">
            A community-driven platform for sharing ideas, discussions, and content.
            Join the conversation or browse what others are talking about.
          </p>
          <div class="flex items-center gap-2">
            <NuxtLink to="/register" class="button button-sm primary no-underline">
              Create Account
            </NuxtLink>
            <NuxtLink to="/boards" class="button button-sm white no-underline">
              Browse Boards
            </NuxtLink>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab bar (only when user has both feed and forum boards) -->
    <div v-if="showTabs" class="pt-4">
      <div class="flex gap-1 bg-white rounded-lg border border-gray-200 p-1">
        <button
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors"
          :class="activeTab === 'feed'
            ? 'bg-primary text-white shadow-sm'
            : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'"
          @click="switchTab('feed')"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
          </svg>
          Feed
        </button>
        <button
          class="flex-1 flex items-center justify-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors"
          :class="activeTab === 'threads'
            ? 'bg-primary text-white shadow-sm'
            : 'text-gray-600 hover:text-gray-900 hover:bg-gray-50'"
          @click="switchTab('threads')"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
          </svg>
          Threads
        </button>
      </div>
    </div>

    <!-- Feed tab content -->
    <div v-show="activeTab === 'feed' || (!showTabs && !hasForumBoards)">
      <!-- Sort bar -->
      <div class="pt-4">
        <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
          <CommonSortSelector v-model="feedPosts.sort.value" @update:model-value="feedPosts.setSort" />
          <CommonViewToggle />
        </div>
      </div>

      <!-- Content area -->
      <div class="pb-4">
        <CommonErrorDisplay v-if="feedPosts.error.value" :message="feedPosts.error.value.message" @retry="feedPosts.fetchPosts" />

        <PostList :posts="feedPosts.posts.value" :loading="feedPosts.loading.value" />

        <CommonPagination
          v-if="feedPosts.posts.value.length > 0"
          :page="feedPosts.page.value"
          :has-more="feedPosts.hasMore.value"
          @prev="feedPosts.prevPage"
          @next="feedPosts.nextPage"
        />
      </div>
    </div>

    <!-- Threads tab content -->
    <div v-if="threadPosts" v-show="activeTab === 'threads' || (!showTabs && hasForumBoards)">
      <div class="pt-4 pb-4">
        <CommonErrorDisplay v-if="threadPosts.error.value" :message="threadPosts.error.value.message" @retry="threadPosts.fetchPosts" />

        <CommonLoadingSpinner v-if="threadPosts.loading.value" />

        <div v-else-if="threadsByBoard.length === 0" class="bg-white border border-gray-200 rounded-lg py-12 text-center">
          <div class="inline-flex w-12 h-12 rounded-full bg-primary/10 items-center justify-center mb-3">
            <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
            </svg>
          </div>
          <p class="text-sm font-medium text-gray-600 mb-1">No threads yet</p>
          <p class="text-xs text-gray-400">No forum boards have any threads yet.</p>
        </div>

        <!-- Threads grouped by board -->
        <div v-else class="space-y-6">
          <div v-for="group in threadsByBoard" :key="group.boardName">
            <!-- Board header -->
            <div class="flex items-center gap-2.5 mb-2">
              <NuxtLink :to="`/b/${group.boardName}`" class="flex items-center gap-2.5 no-underline group">
                <CommonAvatar
                  :src="group.boardIcon ?? undefined"
                  :name="group.boardName"
                  size="sm"
                />
                <h3 class="text-sm font-semibold text-gray-900 group-hover:text-primary transition-colors">
                  {{ group.boardTitle }}
                </h3>
              </NuxtLink>
              <NuxtLink
                :to="`/b/${group.boardName}`"
                class="text-[10px] text-gray-400 hover:text-primary no-underline ml-auto"
              >
                View board
              </NuxtLink>
            </div>

            <!-- Thread list for this board -->
            <div class="forum-thread-list">
              <div class="forum-header">
                <div class="forum-header-topic">Topic</div>
                <div class="forum-header-stats">Replies</div>
                <div class="forum-header-activity">Last Post</div>
              </div>

              <NuxtLink
                v-for="thread in group.threads"
                :key="thread.id"
                :to="`/b/${group.boardName}/${thread.id}/${thread.slug || ''}`"
                class="forum-thread no-underline"
                :class="{ 'forum-thread-pinned': thread.isFeaturedBoard }"
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
                    <span v-if="thread.isFeaturedBoard" class="forum-pin-badge">Pinned</span>
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
          </div>
        </div>

        <CommonPagination
          v-if="threadPosts.posts.value.length > 0"
          :page="threadPosts.page.value"
          :has-more="threadPosts.hasMore.value"
          @prev="threadPosts.prevPage"
          @next="threadPosts.nextPage"
        />
      </div>
    </div>
  </div>
</template>
