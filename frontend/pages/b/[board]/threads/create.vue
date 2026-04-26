<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useBoard } from '~/composables/useBoard'
import type { Post } from '~/types/generated'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const { board: currentBoard } = useBoard()

useHead({ title: `New Thread - ${boardName}` })

const CREATE_POST_MUTATION = `
  mutation CreatePost($title: String!, $board: String, $body: String, $postType: String) {
    createPost(title: $title, board: $board, body: $body, postType: $postType) {
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

const title = ref('')
const body = ref('')

async function handleSubmit (): Promise<void> {
  if (!title.value.trim()) return

  const result = await execute(CREATE_POST_MUTATION, {
    variables: {
      title: title.value,
      body: body.value || undefined,
      board: boardName,
      postType: 'thread',
    },
  })

  if (result?.createPost) {
    const post = result.createPost
    const board = post.board?.name ?? boardName
    await navigateTo(`/b/${board}/${post.id}/${post.slug}`)
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
        Back to Threads
      </NuxtLink>
    </div>

    <div class="bg-white border border-gray-200 rounded-lg overflow-hidden">
      <div class="px-4 py-3 bg-primary/5 border-b border-primary/10 flex items-center gap-2">
        <svg class="w-4 h-4 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
        </svg>
        <h1 class="text-base font-semibold text-gray-900">Create New Thread</h1>
        <span class="text-xs text-gray-500 ml-auto">in b/{{ boardName }}</span>
      </div>

      <form class="p-4 space-y-4" @submit.prevent="handleSubmit">
        <div>
          <label for="thread-title" class="block text-sm font-medium text-gray-700 mb-1">Thread Title</label>
          <input
            id="thread-title"
            v-model="title"
            type="text"
            class="form-input"
            required
            placeholder="What do you want to discuss?"
            autofocus
          >
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Opening Post</label>
          <EditorRichTextEditor
            v-model="body"
            :board-id="currentBoard?.id"
            placeholder="Write the opening post for your thread..."
            min-height="200px"
          />
        </div>

        <CommonErrorDisplay v-if="error" :message="error.message" />

        <div class="flex items-center justify-end gap-3">
          <NuxtLink
            :to="`/b/${boardName}`"
            class="button white button-sm no-underline"
          >
            Cancel
          </NuxtLink>
          <button
            type="submit"
            class="button primary"
            :disabled="loading || !title.trim()"
          >
            {{ loading ? 'Creating...' : 'Create Thread' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
