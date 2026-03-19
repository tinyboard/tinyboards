<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const username = computed(() => route.params.username as string)
const authStore = useAuthStore()

useHead({ title: computed(() => `Saved - @${username.value}`) })

const isOwnProfile = computed(() => authStore.user?.name === username.value)

const SAVED_POSTS_QUERY = `
  query SavedPosts($savedOnly: Boolean, $page: Int, $limit: Int) {
    listPosts(savedOnly: $savedOnly, page: $page, limit: $limit) {
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
const limit = 25
const hasMore = ref(false)

async function fetchSaved (): Promise<void> {
  const result = await execute(SAVED_POSTS_QUERY, {
    variables: { savedOnly: true, page: page.value, limit: limit + 1 },
  })
  if (result?.listPosts) {
    hasMore.value = result.listPosts.length > limit
    posts.value = result.listPosts.slice(0, limit)
  }
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
  <div class="max-w-5xl mx-auto px-4 py-4">
    <template v-if="isOwnProfile">
      <div class="bg-white rounded-lg border border-gray-200 px-4 py-3 mb-4">
        <h2 class="text-base font-semibold text-gray-900">
          Saved
        </h2>
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

      <p v-if="!loading && posts.length === 0" class="text-sm text-gray-500 text-center py-8">
        No saved posts yet.
      </p>
    </template>
    <CommonErrorDisplay
      v-else
      title="Not available"
      message="You can only view your own saved items."
      :retryable="false"
    />
  </div>
</template>
