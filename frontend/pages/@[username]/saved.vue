<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const username = computed(() => route.params.username as string)
const authStore = useAuthStore()

const isOwnProfile = computed(() => authStore.user?.name === username.value)

const SAVED_POSTS_QUERY = `
  query SavedPosts($savedOnly: Boolean, $sort: SortType, $page: Int, $limit: Int) {
    listPosts(savedOnly: $savedOnly, sort: $sort, page: $page, limit: $limit) {
      id
      title
      body
      url
      createdAt
      updatedAt
      isDeleted
      isRemoved
      isLocked
      isFeaturedBoard
      isNSFW
      slug
      score
      upvotes
      downvotes
      commentCount
      myVote
      board { id name title icon }
      creator { id name displayName avatar }
    }
  }
`

interface SavedPostsResponse {
  listPosts: Post[]
}

const { execute, loading, error } = useGraphQL<SavedPostsResponse>()
const posts = ref<Post[]>([])
const page = ref(1)
const sort = ref('new')
const limit = 25
const hasMore = ref(false)

async function fetchSaved (): Promise<void> {
  const result = await execute(SAVED_POSTS_QUERY, {
    variables: { savedOnly: true, sort: sort.value, page: page.value, limit: limit + 1 },
  })
  if (result?.listPosts) {
    hasMore.value = result.listPosts.length > limit
    posts.value = result.listPosts.slice(0, limit)
  }
}

async function setSort (newSort: string): Promise<void> {
  sort.value = newSort
  page.value = 1
  await fetchSaved()
}

async function nextPage (): Promise<void> {
  if (hasMore.value) { page.value++; await fetchSaved() }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) { page.value--; await fetchSaved() }
}

if (isOwnProfile.value) {
  await fetchSaved()
}
</script>

<template>
  <div>
    <template v-if="isOwnProfile">
      <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
        <h2 class="text-sm font-semibold text-gray-900">
          Saved Posts
        </h2>
        <CommonSortSelector v-model="sort" @update:model-value="setSort" />
      </div>

      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchSaved" />
      <PostList :posts="posts" :loading="loading" />

      <CommonPagination
        v-if="posts.length > 0"
        :page="page"
        :has-more="hasMore"
        @prev="prevPage"
        @next="nextPage"
      />

      <div v-if="!loading && posts.length === 0" class="bg-white rounded-lg border border-gray-200 py-12 text-center">
        <div class="inline-flex w-12 h-12 rounded-xl bg-gray-100 items-center justify-center mb-3">
          <svg class="w-6 h-6 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
          </svg>
        </div>
        <p class="text-sm font-medium text-gray-600 mb-1">No saved posts yet</p>
        <p class="text-xs text-gray-400">Click the save button on any post to bookmark it here.</p>
      </div>
    </template>
    <CommonErrorDisplay
      v-else
      title="Not available"
      message="You can only view your own saved items."
      :retryable="false"
    />
  </div>
</template>
