<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import type { Post } from '~/types/generated'
import { postUrl } from '~/utils/slug'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Create Post - ${boardName}` })

const CREATE_POST_MUTATION = `
  mutation CreatePost($title: String!, $board: String, $body: String, $link: String, $isNSFW: Boolean, $altText: String, $postType: String) {
    createPost(title: $title, board: $board, body: $body, link: $link, isNSFW: $isNSFW, altText: $altText, postType: $postType) {
      id
      slug
      board { name }
    }
  }
`

const CREATE_POST_WITH_FILE_MUTATION = `
  mutation CreatePostWithFile($title: String!, $board: String, $body: String, $link: String, $isNSFW: Boolean, $altText: String, $file: Upload, $postType: String) {
    createPost(title: $title, board: $board, body: $body, link: $link, isNSFW: $isNSFW, altText: $altText, file: $file, postType: $postType) {
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
const { executeWithFile, uploading: fileUploading, error: uploadError } = useFileUpload()

async function handleSubmit (data: { title: string; body: string; url: string; file: File | null; altText: string }): Promise<void> {
  if (!data.title.trim()) return

  let result: CreatePostResponse | null = null

  const baseVars = {
    title: data.title,
    body: data.body || undefined,
    link: data.url || undefined,
    board: boardName,
    altText: data.altText || undefined,
  }

  if (data.file) {
    const uploadResult = await executeWithFile(
      CREATE_POST_WITH_FILE_MUTATION,
      baseVars as Record<string, unknown>,
      'file',
      data.file,
    )
    result = uploadResult as CreatePostResponse | null
  } else {
    result = await execute(CREATE_POST_MUTATION, { variables: baseVars })
  }

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
        <CommonErrorDisplay v-if="uploadError" :message="uploadError.message" class="mb-4" />
        <PostForm
          :board-name="boardName"
          :submit-label="loading || fileUploading ? 'Posting...' : 'Create Post'"
          @submit="handleSubmit"
        />
      </div>
    </div>
  </div>
</template>
