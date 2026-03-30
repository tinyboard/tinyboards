<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import { useFlairs } from '~/composables/useFlairs'
import type { Post, Board } from '~/types/generated'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Submit' })

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

const ASSIGN_POST_FLAIR_MUTATION = `
  mutation AssignPostFlair($input: AssignPostFlairInput!) {
    assignPostFlair(input: $input) {
      id postId flairTemplateId
    }
  }
`

const LIST_BOARDS_QUERY = `
  query ListBoards($searchTerm: String, $limit: Int) {
    listBoards(searchTerm: $searchTerm, limit: $limit) {
      id
      name
      title
      icon
      mode
    }
  }
`

interface CreatePostResponse {
  createPost: Post
}

const { execute, loading, error } = useGraphQL<CreatePostResponse>()
const { executeWithFile, uploading: fileUploading, error: uploadError } = useFileUpload()
const { flairs, fetchFlairs } = useFlairs()

const boardName = ref((useRoute().query.board as string) ?? '')
const boardSearch = ref('')
const boardResults = ref<(Board & { mode?: string })[]>([])
const showBoardDropdown = ref(false)
const selectedBoard = ref<(Board & { mode?: string }) | null>(null)
const isNSFW = ref(false)
const selectedFlairId = ref<string | null>(null)
const boardId = ref<string | null>(null)

// Derive post type automatically from board mode
const postType = computed(() => {
  if (!selectedBoard.value) return 'feed'
  return selectedBoard.value.mode === 'forum' ? 'thread' : 'feed'
})

// Preselect type from query param (e.g. ?type=thread)
const queryType = useRoute().query.type as string | undefined

// Search boards as user types
let boardSearchTimeout: ReturnType<typeof setTimeout> | null = null
watch(boardSearch, (term) => {
  if (boardSearchTimeout) clearTimeout(boardSearchTimeout)
  if (!term.trim()) {
    boardResults.value = []
    showBoardDropdown.value = false
    return
  }
  boardSearchTimeout = setTimeout(async () => {
    const { execute: execSearch } = useGraphQL<{ listBoards: (Board & { mode?: string })[] }>()
    const result = await execSearch(LIST_BOARDS_QUERY, {
      variables: { searchTerm: term.trim(), limit: 10 },
    })
    if (result?.listBoards) {
      boardResults.value = result.listBoards
      showBoardDropdown.value = true
    }
  }, 300)
})

async function selectBoard (board: Board & { mode?: string }) {
  selectedBoard.value = board
  boardName.value = board.name
  boardSearch.value = board.name
  boardId.value = board.id
  showBoardDropdown.value = false
  // Fetch flairs for the selected board
  selectedFlairId.value = null
  flairs.value = []
  await fetchFlairs(board.id)
}

// If board was provided via query param, fetch it
onMounted(async () => {
  if (boardName.value) {
    boardSearch.value = boardName.value
    const { execute: execSearch } = useGraphQL<{ listBoards: (Board & { mode?: string })[] }>()
    const result = await execSearch(LIST_BOARDS_QUERY, {
      variables: { searchTerm: boardName.value, limit: 1 },
    })
    if (result?.listBoards?.length) {
      selectBoard(result.listBoards[0])
    }
  }
})

function modeBadgeClass (mode: string | undefined): string {
  if (mode === 'forum') return 'bg-purple-100 text-purple-700'
  return 'bg-blue-100 text-blue-700'
}

function modeBadgeLabel (mode: string | undefined): string {
  if (mode === 'forum') return '💬 Forum'
  return '📰 Feed'
}

async function handleSubmit (data: { title: string; body: string; url: string; file: File | null; altText: string }): Promise<void> {
  let result: CreatePostResponse | null = null

  const baseVars = {
    title: data.title,
    body: data.body || undefined,
    link: data.url || undefined,
    board: boardName.value || undefined,
    isNSFW: isNSFW.value,
    altText: data.altText || undefined,
    postType: postType.value,
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
    const post = result.createPost
    // Assign flair if selected
    if (selectedFlairId.value && post.id) {
      const { execute: execFlair } = useGraphQL()
      await execFlair(ASSIGN_POST_FLAIR_MUTATION, {
        variables: {
          input: {
            postId: post.id,
            flairTemplateId: selectedFlairId.value,
          },
        },
      })
    }

    const board = post.board?.name ?? boardName.value
    if (board) {
      await navigateTo(`/b/${board}/${post.id}/${post.slug || ''}`)
    } else {
      await navigateTo('/home')
    }
  }
}
</script>

<template>
  <div class="max-w-2xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-4">
      Create a Post
    </h1>

    <div class="mb-4 space-y-3">
      <!-- Board selector with search -->
      <div class="relative">
        <label for="board-search" class="block text-sm font-medium text-gray-700 mb-1">Board</label>
        <input
          id="board-search"
          v-model="boardSearch"
          type="text"
          class="form-input"
          placeholder="Search for a board..."
          autocomplete="off"
          @focus="boardSearch.trim() && boardResults.length ? showBoardDropdown = true : null"
        >
        <!-- Dropdown results -->
        <div
          v-if="showBoardDropdown && boardResults.length"
          class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg max-h-60 overflow-auto"
        >
          <button
            v-for="b in boardResults"
            :key="b.id"
            type="button"
            class="w-full px-3 py-2 text-left hover:bg-gray-50 flex items-center gap-2 text-sm"
            @click="selectBoard(b)"
          >
            <CommonAvatar v-if="b.icon" :src="b.icon" :name="b.name" size="sm" />
            <div class="flex-1 min-w-0">
              <div class="font-medium text-gray-900 flex items-center gap-1.5">
                b/{{ b.name }}
                <span
                  class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium"
                  :class="modeBadgeClass(b.mode)"
                >
                  {{ modeBadgeLabel(b.mode) }}
                </span>
              </div>
              <div v-if="b.title && b.title !== b.name" class="text-xs text-gray-500">{{ b.title }}</div>
            </div>
          </button>
        </div>
      </div>

      <!-- Selected board mode indicator -->
      <div v-if="selectedBoard" class="flex items-center gap-2 text-sm text-gray-600">
        <span
          class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
          :class="modeBadgeClass(selectedBoard.mode)"
        >
          {{ modeBadgeLabel(selectedBoard.mode) }}
        </span>
        <span v-if="selectedBoard.mode === 'forum'">
          This is a forum board &mdash; your post will be a discussion thread.
        </span>
        <span v-else>
          This is a feed board &mdash; share links, images, or text.
        </span>
      </div>

      <!-- Post flair selector -->
      <div v-if="flairs.length > 0">
        <label class="block text-sm font-medium text-gray-700 mb-1">Post Flair</label>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="flair in flairs"
            :key="flair.id"
            type="button"
            class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium border-2 transition-colors cursor-pointer"
            :class="selectedFlairId === flair.id ? 'border-gray-900' : 'border-transparent'"
            :style="{ color: flair.textColor, backgroundColor: flair.backgroundColor }"
            @click="selectedFlairId = selectedFlairId === flair.id ? null : flair.id"
          >
            {{ flair.textDisplay || flair.templateName }}
          </button>
        </div>
      </div>

      <label class="flex items-center gap-2">
        <input v-model="isNSFW" type="checkbox" class="form-checkbox" />
        <span class="text-sm text-gray-700">Mark as NSFW</span>
      </label>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" />
    <CommonErrorDisplay v-if="uploadError" :message="uploadError.message" />

    <PostForm
      :board-name="boardName"
      :submit-label="loading || fileUploading ? 'Submitting...' : 'Submit'"
      @submit="handleSubmit"
    />
  </div>
</template>
