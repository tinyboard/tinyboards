<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Comment } from '~/types/generated'
import { timeAgo } from '~/utils/date'

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
      post { id title slug board { id name } }
      board { id name title }
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
        class="bg-white border border-gray-200 rounded-lg overflow-hidden"
      >
        <!-- Context header: which post & board this comment is on -->
        <div v-if="comment.post || comment.board" class="px-3 py-2 bg-gray-50 border-b border-gray-100 flex items-center gap-1.5 text-xs text-gray-500">
          <svg class="w-3.5 h-3.5 text-gray-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
          </svg>
          <span class="text-gray-400">commented on</span>
          <NuxtLink
            v-if="comment.post"
            :to="`/b/${comment.post.board?.name || comment.board?.name || 'unknown'}/feed/${comment.postId}/${comment.post.slug || ''}`"
            class="font-medium text-gray-700 no-underline hover:text-primary truncate"
          >
            {{ comment.post.title }}
          </NuxtLink>
          <span v-if="comment.board || comment.post?.board" class="text-gray-400 shrink-0">in</span>
          <NuxtLink
            v-if="comment.board || comment.post?.board"
            :to="`/b/${comment.post?.board?.name || comment.board?.name}`"
            class="font-medium text-primary no-underline hover:underline shrink-0"
          >
            b/{{ comment.post?.board?.name || comment.board?.name }}
          </NuxtLink>
        </div>

        <!-- Comment content -->
        <div class="p-3">
          <div class="flex items-center gap-2 text-xs text-gray-500 mb-1.5">
            <span class="inline-flex items-center gap-1" :class="comment.score > 0 ? 'text-primary' : comment.score < 0 ? 'text-red-400' : ''">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
              </svg>
              {{ comment.score }} {{ comment.score === 1 ? 'point' : 'points' }}
            </span>
            <span>&middot;</span>
            <time :datetime="comment.createdAt" :title="comment.createdAt">{{ timeAgo(comment.createdAt) }}</time>
            <template v-if="comment.replyCount > 0">
              <span>&middot;</span>
              <span class="inline-flex items-center gap-1">
                <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
                {{ comment.replyCount }} {{ comment.replyCount === 1 ? 'reply' : 'replies' }}
              </span>
            </template>
          </div>
          <p class="text-sm text-gray-800 leading-relaxed">{{ comment.body }}</p>
        </div>
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
