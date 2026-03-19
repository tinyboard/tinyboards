<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Content' })

interface Post {
  id: string
  title: string
  body: string
  creatorId: string
  isRemoved: boolean
  createdAt: string
  creator: {
    name: string
  }
}

interface ListPostsResponse {
  listPosts: Post[]
}

interface RestorePostResponse {
  restorePost: { id: string }
}

const page = ref(1)
const limit = 20

const { execute, loading, error, data } = useGraphQL<ListPostsResponse>()
const { execute: executeRestore, loading: restoring } = useGraphQLMutation<RestorePostResponse>()

const LIST_REMOVED_POSTS = `
  query ListRemovedPosts($page: Int, $limit: Int) {
    listPosts(removedOnly: true, page: $page, limit: $limit) {
      id
      title
      body
      creatorId
      isRemoved
      createdAt
      creator {
        name
      }
    }
  }
`

const RESTORE_POST = `
  mutation RestorePost($postId: ID!) {
    restorePost(postId: $postId) {
      id
    }
  }
`

async function fetchPosts () {
  await execute(LIST_REMOVED_POSTS, {
    variables: { page: page.value, limit },
  })
}

async function restorePost (id: string) {
  await executeRestore(RESTORE_POST, { variables: { postId: id } })
  await fetchPosts()
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function handlePageChange (newPage: number) {
  page.value = newPage
  await fetchPosts()
}

onMounted(() => {
  fetchPosts()
})

const posts = computed(() => data.value?.listPosts ?? [])
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Content Moderation
    </h2>
    <p class="text-sm text-gray-500 mb-6">
      Review and restore removed posts and comments.
    </p>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="posts.length === 0" class="text-sm text-gray-500">
      No removed content found.
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="post in posts"
        :key="post.id"
        class="bg-white rounded-lg border border-gray-200 p-4"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-medium text-gray-900 truncate">
              {{ post.title }}
            </h3>
            <p v-if="post.body" class="mt-1 text-sm text-gray-600 line-clamp-2">
              {{ post.body }}
            </p>
            <div class="mt-2 flex items-center gap-3 text-xs text-gray-500">
              <span>
                by <span class="font-medium">{{ post.creator.name }}</span>
              </span>
              <span>{{ formatDate(post.createdAt) }}</span>
            </div>
          </div>
          <button
            class="button button-sm white ml-4 shrink-0"
            :disabled="restoring"
            @click="restorePost(post.id)"
          >
            Restore
          </button>
        </div>
      </div>

      <CommonPagination
        :page="page"
        :has-more="posts.length === limit"
        @prev="page > 1 ? (page--, fetchPosts()) : null"
        @next="page++; fetchPosts()"
      />
    </div>
  </div>
</template>
