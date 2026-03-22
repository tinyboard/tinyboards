<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Comment } from '~/types/generated'

const route = useRoute()
const username = computed(() => route.params.username as string)

const USER_COMMENTS_QUERY = `
  query UserComments($userName: String, $sort: CommentSortType, $page: Int, $limit: Int) {
    comments(userName: $userName, sort: $sort, page: $page, limit: $limit) {
      id
      body
      createdAt
      updatedAt
      score
      upvotes
      downvotes
      replyCount
      postId
      boardId
      creator { id name displayName avatar }
    }
  }
`

interface UserCommentsResponse {
  comments: Comment[]
}

const { execute, loading, error } = useGraphQL<UserCommentsResponse>()
const comments = ref<Comment[]>([])
const page = ref(1)
const sort = ref('new')
const limit = 25
const hasMore = ref(false)

async function fetchComments (): Promise<void> {
  const result = await execute(USER_COMMENTS_QUERY, {
    variables: { userName: username.value, sort: sort.value, page: page.value, limit: limit + 1 },
  })
  if (result?.comments) {
    hasMore.value = result.comments.length > limit
    comments.value = result.comments.slice(0, limit)
  }
}

async function nextPage (): Promise<void> {
  if (hasMore.value) { page.value++; await fetchComments() }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) { page.value--; await fetchComments() }
}

async function setSort (newSort: string): Promise<void> {
  sort.value = newSort
  page.value = 1
  await fetchComments()
}

const commentSortOptions = [
  { label: 'New', value: 'new' },
  { label: 'Old', value: 'old' },
  { label: 'Top', value: 'top' },
  { label: 'Hot', value: 'hot' },
]

watch(username, () => {
  page.value = 1
  fetchComments()
})
await fetchComments()
</script>

<template>
  <div>
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
      <h2 class="text-sm font-semibold text-gray-900">
        Comments by @{{ username }}
      </h2>
      <CommonSortSelector v-model="sort" :options="commentSortOptions" @update:model-value="setSort" />
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchComments" />
    <CommonLoadingSpinner v-else-if="loading && comments.length === 0" size="lg" />

    <div v-else-if="comments.length > 0" class="space-y-2">
      <div
        v-for="comment in comments"
        :key="comment.id"
        class="bg-white border border-gray-200 rounded-lg p-3"
      >
        <div class="flex items-center gap-2 text-xs text-gray-500 mb-1">
          <span>{{ comment.score }} points</span>
          <span>&middot;</span>
          <span>{{ comment.createdAt }}</span>
        </div>
        <p class="text-sm text-gray-800">{{ comment.body }}</p>
      </div>
    </div>
    <p v-else class="text-sm text-gray-500 text-center py-8">
      No comments yet.
    </p>

    <CommonPagination
      v-if="comments.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
