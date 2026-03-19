<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'

const route = useRoute()
const username = (route.params.username as string)

useHead({ title: `Posts - @${username}` })

const USER_POSTS_QUERY = `
  query UserPosts($userName: String, $sort: SortType, $page: Int, $limit: Int) {
    listPosts(userName: $userName, sort: $sort, page: $page, limit: $limit) {
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

interface UserPostsResponse {
  listPosts: Post[]
}

const { execute, loading, error } = useGraphQL<UserPostsResponse>()
const posts = ref<Post[]>([])
const page = ref(1)
const sort = ref('new')
const limit = 25
const hasMore = ref(false)

async function fetchPosts (): Promise<void> {
  const result = await execute(USER_POSTS_QUERY, {
    variables: { userName: username, sort: sort.value, page: page.value, limit: limit + 1 },
  })
  if (result?.listPosts) {
    hasMore.value = result.listPosts.length > limit
    posts.value = result.listPosts.slice(0, limit)
  }
}

async function setSort (newSort: string): Promise<void> {
  sort.value = newSort
  page.value = 1
  await fetchPosts()
}

async function nextPage (): Promise<void> {
  if (hasMore.value) { page.value++; await fetchPosts() }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) { page.value--; await fetchPosts() }
}

await fetchPosts()
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 py-4">
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
      <h2 class="text-sm font-semibold text-gray-900">
        Posts by @{{ username }}
      </h2>
      <CommonSortSelector v-model="sort" @update:model-value="setSort" />
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />
    <PostList :posts="posts" :loading="loading" />

    <CommonPagination
      v-if="posts.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
