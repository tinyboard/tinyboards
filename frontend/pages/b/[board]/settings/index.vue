<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Settings - b/${boardName}` })

interface BoardData {
  id: string
  name: string
  title: string
  description: string | null
  sidebar: string | null
  icon: string | null
  banner: string | null
  isNSFW: boolean
  isPostingRestrictedToMods: boolean
  isHidden: boolean
  excludeFromAll: boolean
  wikiEnabled: boolean
  mode: string
}

interface BoardSettingsResponse {
  getBoardSettings: {
    board: BoardData
    isOwner: boolean
    moderatorPermissions: number | null
  }
}

interface UpdateResponse {
  updateBoardSettings: { board: BoardData }
}

const GET_BOARD_SETTINGS = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      board {
        id name title description sidebar icon banner
        isNSFW isPostingRestrictedToMods isHidden excludeFromAll wikiEnabled mode
      }
      isOwner
      moderatorPermissions
    }
  }
`

const UPDATE_BOARD_SETTINGS = `
  mutation UpdateBoardSettings($input: UpdateBoardSettingsInput!) {
    updateBoardSettings(input: $input) {
      board { id name title description sidebar icon banner isNSFW isPostingRestrictedToMods mode }
    }
  }
`

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const { execute, loading, error } = useGraphQL<BoardSettingsResponse>()
const { execute: executeMutation, loading: saving } = useGraphQLMutation<UpdateResponse>()

const boardId = ref<string | null>(null)
const isOwner = ref(false)
const hasExistingPosts = ref(false)
const showModeChangeConfirm = ref(false)
const pendingMode = ref<string | null>(null)

const form = reactive({
  title: '',
  description: '',
  sidebar: '',
  isNsfw: false,
  postingRestrictedToMods: false,
  isHidden: false,
  excludeFromAll: false,
  wikiEnabled: false,
  mode: 'feed',
})

const originalMode = ref('feed')

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string; posts: number } }>()
  const boardResult = await execBoard(`query GetBoard($name: String!) { board(name: $name) { id posts } }`, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id
  hasExistingPosts.value = (boardResult.board.posts ?? 0) > 0

  const result = await execute(GET_BOARD_SETTINGS, { variables: { boardId: boardId.value } })
  if (result?.getBoardSettings) {
    const { board, isOwner: owner } = result.getBoardSettings
    isOwner.value = owner
    form.title = board.title
    form.description = board.description ?? ''
    form.sidebar = board.sidebar ?? ''
    form.isNsfw = board.isNSFW
    form.postingRestrictedToMods = board.isPostingRestrictedToMods
    form.isHidden = board.isHidden
    form.excludeFromAll = board.excludeFromAll
    form.wikiEnabled = board.wikiEnabled
    form.mode = board.mode ?? 'feed'
    originalMode.value = form.mode
  }
})

function requestModeChange (newMode: string) {
  if (hasExistingPosts.value && newMode !== originalMode.value) {
    pendingMode.value = newMode
    showModeChangeConfirm.value = true
  } else {
    form.mode = newMode
  }
}

function confirmModeChange () {
  if (pendingMode.value) {
    form.mode = pendingMode.value
  }
  showModeChangeConfirm.value = false
  pendingMode.value = null
}

function cancelModeChange () {
  showModeChangeConfirm.value = false
  pendingMode.value = null
}

async function saveSettings () {
  if (!boardId.value) return

  const result = await executeMutation(UPDATE_BOARD_SETTINGS, {
    variables: {
      input: {
        boardId: boardId.value,
        title: form.title,
        description: form.description || null,
        sidebar: form.sidebar || null,
        isNsfw: form.isNsfw,
        postingRestrictedToMods: form.postingRestrictedToMods,
        isHidden: form.isHidden,
        excludeFromAll: form.excludeFromAll,
        wikiEnabled: form.wikiEnabled,
        mode: form.mode,
      },
    },
  })

  if (result?.updateBoardSettings) {
    originalMode.value = form.mode
    toast.success('Board settings saved')
  }
}
</script>

<template>
  <div>
    <!-- Settings sub-navigation -->
    <div class="flex gap-1 border-b border-gray-200 mb-4">
      <NuxtLink
        :to="`/b/${boardName}/settings`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-blue-600 text-blue-600"
      >
        General
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/appearance`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Appearance
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/moderation`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Moderation
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/emojis`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Emojis
      </NuxtLink>
    </div>

    <h2 class="text-base font-semibold text-gray-900 mb-4">
      Board Settings
    </h2>

    <CommonLoadingSpinner v-if="loading" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <form v-else class="space-y-5 max-w-2xl" @submit.prevent="saveSettings">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Display Name</label>
        <input v-model="form.title" type="text" class="form-input w-full" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
        <textarea v-model="form.description" rows="3" class="form-input w-full" placeholder="Short board description" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Sidebar (Markdown)</label>
        <textarea v-model="form.sidebar" rows="6" class="form-input w-full font-mono text-sm" placeholder="Sidebar content in Markdown format" />
      </div>

      <div class="space-y-3">
        <label class="flex items-center gap-2">
          <input v-model="form.isNsfw" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">NSFW Board</span>
        </label>

        <label class="flex items-center gap-2">
          <input v-model="form.postingRestrictedToMods" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Restrict posting to moderators only</span>
        </label>

        <label class="flex items-center gap-2">
          <input v-model="form.isHidden" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Hidden (not shown in board directory)</span>
        </label>

        <label class="flex items-center gap-2">
          <input v-model="form.excludeFromAll" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Exclude from /all feed</span>
        </label>

        <label class="flex items-center gap-2">
          <input v-model="form.wikiEnabled" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Enable wiki</span>
        </label>
      </div>

      <!-- Board Mode -->
      <div class="bg-white border border-gray-200 rounded-lg p-5">
        <h3 class="text-sm font-medium text-gray-900 mb-2">Board Mode</h3>
        <p class="text-xs text-gray-500 mb-3">
          Controls the type of content this board accepts.
        </p>
        <div class="grid grid-cols-2 gap-3">
          <button
            type="button"
            class="text-left rounded-lg border-2 p-4 transition-all"
            :class="form.mode === 'feed'
              ? 'border-blue-600 bg-blue-50 ring-1 ring-blue-600'
              : 'border-gray-200 bg-white hover:border-gray-300'"
            @click="requestModeChange('feed')"
          >
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-lg">📰</span>
              <span class="font-semibold text-sm text-gray-900">Feed Board</span>
            </div>
            <p class="text-xs text-gray-500 leading-relaxed">
              Share links, images, and text posts. Members vote on content.
            </p>
          </button>
          <button
            type="button"
            class="text-left rounded-lg border-2 p-4 transition-all"
            :class="form.mode === 'forum'
              ? 'border-blue-600 bg-blue-50 ring-1 ring-blue-600'
              : 'border-gray-200 bg-white hover:border-gray-300'"
            @click="requestModeChange('forum')"
          >
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-lg">💬</span>
              <span class="font-semibold text-sm text-gray-900">Forum Board</span>
            </div>
            <p class="text-xs text-gray-500 leading-relaxed">
              Threaded discussions. Great for Q&amp;A, support, or structured topics.
            </p>
          </button>
        </div>
      </div>

      <!-- Mode change confirmation dialog -->
      <div v-if="showModeChangeConfirm" class="bg-amber-50 border border-amber-200 rounded-lg p-4">
        <h4 class="text-sm font-medium text-amber-800 mb-1">Change board mode?</h4>
        <p class="text-xs text-amber-700 mb-3">
          This board already has posts. Existing posts will not be affected &mdash; only new posts will follow the new mode.
        </p>
        <div class="flex gap-2">
          <button type="button" class="button button-sm primary" @click="confirmModeChange">
            Confirm
          </button>
          <button type="button" class="button button-sm white" @click="cancelModeChange">
            Cancel
          </button>
        </div>
      </div>

      <div>
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Settings' }}
        </button>
      </div>
    </form>
  </div>
</template>
