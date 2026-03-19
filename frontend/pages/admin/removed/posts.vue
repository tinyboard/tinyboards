<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Removed Posts' })

interface Post {
  id: string
  title: string
  body: string
  isRemoved: boolean
  createdAt: string
  creator: { name: string } | null
}

const { execute, loading, error, data } = useGraphQL<{ listPosts: Post[] }>()
const { execute: executeRestore, loading: restoring } = useGraphQLMutation()
const page = ref(1)
const limit = 20

const QUERY = `
  query ListRemovedPosts($page: Int, $limit: Int) {
    listPosts(removedOnly: true, page: $page, limit: $limit) {
      id title body isRemoved createdAt
      creator { name }
    }
  }
`

const RESTORE = `
  mutation RestorePost($postId: ID!) {
    restorePost(postId: $postId) { id }
  }
`

async function fetchPosts () {
  await execute(QUERY, { variables: { page: page.value, limit } })
}

async function restorePost (id: string) {
  await executeRestore(RESTORE, { variables: { postId: id } })
  await fetchPosts()
}

onMounted(() => { fetchPosts() })

const posts = computed(() => data.value?.listPosts ?? [])
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Removed Posts
    </h2>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="posts.length === 0" class="text-sm text-gray-500">
      No removed posts.
    </div>

    <div v-else class="space-y-3">
      <div v-for="post in posts" :key="post.id" class="bg-white rounded-lg border border-gray-200 p-4 flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <h3 class="text-sm font-medium text-gray-900 truncate">{{ post.title }}</h3>
          <p class="text-xs text-gray-500 mt-1">by {{ post.creator?.name ?? 'unknown' }}</p>
        </div>
        <button class="button button-sm white ml-4 shrink-0" :disabled="restoring" @click="restorePost(post.id)">
          Restore
        </button>
      </div>

      <CommonPagination :page="page" :has-more="posts.length === limit" @prev="page > 1 && (page--, fetchPosts())" @next="page++; fetchPosts()" />
    </div>
  </div>
</template>
