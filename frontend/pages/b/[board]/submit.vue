<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'
import { postUrl } from '~/utils/slug'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Create Post - ${boardName}` })

const CREATE_POST_MUTATION = `
  mutation CreatePost($title: String!, $board: String, $body: String, $url: String) {
    createPost(title: $title, board: $board, body: $body, url: $url) {
      id
      slug
      board { name }
    }
  }
`

interface CreatePostResponse {
  createPost: Post
}

const { execute, loading, error } = useGraphQL<CreatePostResponse>()

async function handleSubmit (data: { title: string; body: string; url: string; file: File | null; altText: string }): Promise<void> {
  if (!data.title.trim()) return

  const result = await execute(CREATE_POST_MUTATION, {
    variables: {
      title: data.title,
      body: data.body || undefined,
      url: data.url || undefined,
      board: boardName,
    },
  })

  if (result?.createPost) {
    await navigateTo(postUrl(result.createPost))
  }
}
</script>

<template>
  <div>
    <div class="mb-4">
      <NuxtLink
        :to="`/b/${boardName}`"
        class="text-xs text-gray-500 hover:text-primary no-underline inline-flex items-center gap-1"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
        Back to b/{{ boardName }}
      </NuxtLink>
    </div>

    <div class="bg-white border border-gray-200 rounded-lg overflow-hidden">
      <div class="px-4 py-3 bg-green-50 border-b border-green-100 flex items-center gap-2">
        <svg class="w-4 h-4 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
        <h1 class="text-base font-semibold text-gray-900">Create Post</h1>
        <span class="text-xs text-gray-500 ml-auto">in b/{{ boardName }}</span>
      </div>

      <div class="p-4">
        <CommonErrorDisplay v-if="error" :message="error.message" class="mb-4" />
        <PostForm
          :board-name="boardName"
          :submit-label="loading ? 'Posting...' : 'Create Post'"
          @submit="handleSubmit"
        />
      </div>
    </div>
  </div>
</template>
