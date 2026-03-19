<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Removed Comments' })

interface Comment {
  id: string
  body: string
  isRemoved: boolean
  createdAt: string
  creator: { name: string } | null
  post: { title: string }
}

const { execute, loading, error, data } = useGraphQL<{ comments: Comment[] }>()
const { execute: executeRestore, loading: restoring } = useGraphQLMutation()
const page = ref(1)
const limit = 20

const QUERY = `
  query ListRemovedComments($page: Int, $limit: Int) {
    comments(removedOnly: true, page: $page, limit: $limit) {
      id body isRemoved createdAt
      creator { name }
      post { title }
    }
  }
`

const RESTORE = `
  mutation RestoreComment($commentId: ID!) {
    restoreComment(commentId: $commentId) { id }
  }
`

async function fetchComments () {
  await execute(QUERY, { variables: { page: page.value, limit } })
}

async function restoreComment (id: string) {
  await executeRestore(RESTORE, { variables: { commentId: id } })
  await fetchComments()
}

onMounted(() => { fetchComments() })

const comments = computed(() => data.value?.comments ?? [])
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Removed Comments
    </h2>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="comments.length === 0" class="text-sm text-gray-500">
      No removed comments.
    </div>

    <div v-else class="space-y-3">
      <div v-for="comment in comments" :key="comment.id" class="bg-white rounded-lg border border-gray-200 p-4 flex items-start justify-between">
        <div class="flex-1 min-w-0">
          <p class="text-sm text-gray-900 line-clamp-2">{{ comment.body }}</p>
          <p class="text-xs text-gray-500 mt-1">
            by {{ comment.creator?.name ?? 'unknown' }} on "{{ comment.post.title }}"
          </p>
        </div>
        <button class="button button-sm white ml-4 shrink-0" :disabled="restoring" @click="restoreComment(comment.id)">
          Restore
        </button>
      </div>

      <CommonPagination :page="page" :has-more="comments.length === limit" @prev="page > 1 && (page--, fetchComments())" @next="page++; fetchComments()" />
    </div>
  </div>
</template>
