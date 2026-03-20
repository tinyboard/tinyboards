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
  sectionConfig: number
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
        isNSFW isPostingRestrictedToMods isHidden excludeFromAll wikiEnabled sectionConfig
      }
      isOwner
      moderatorPermissions
    }
  }
`

const UPDATE_BOARD_SETTINGS = `
  mutation UpdateBoardSettings($input: UpdateBoardSettingsInput!) {
    updateBoardSettings(input: $input) {
      board { id name title description sidebar icon banner isNSFW isPostingRestrictedToMods }
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

const form = reactive({
  title: '',
  description: '',
  sidebar: '',
  isNsfw: false,
  postingRestrictedToMods: false,
  isHidden: false,
  excludeFromAll: false,
  wikiEnabled: false,
  feedEnabled: true,
  threadsEnabled: false,
  defaultSection: 'feed' as 'feed' | 'threads',
})

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

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
    form.feedEnabled = (board.sectionConfig & 1) === 1
    form.threadsEnabled = (board.sectionConfig & 2) === 2
    form.defaultSection = form.threadsEnabled && !form.feedEnabled ? 'threads' : 'feed'
  }
})

async function saveSettings () {
  if (!boardId.value) return

  const sectionConfig = (form.feedEnabled ? 1 : 0) | (form.threadsEnabled ? 2 : 0)

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
        sectionConfig: sectionConfig || 1,
        defaultSection: form.defaultSection,
      },
    },
  })

  if (result?.updateBoardSettings) {
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

      <!-- Section Configuration -->
      <div class="bg-white border border-gray-200 rounded-lg p-5">
        <h3 class="text-sm font-medium text-gray-900 mb-3">Board Sections</h3>
        <p class="text-xs text-gray-500 mb-3">
          Choose which content sections are available on this board. At least one must be enabled.
        </p>

        <div class="space-y-3 mb-4">
          <label class="flex items-center gap-2">
            <input
              v-model="form.feedEnabled"
              type="checkbox"
              class="form-checkbox"
              :disabled="!form.threadsEnabled"
            />
            <span class="text-sm text-gray-700">Feed</span>
            <span class="text-xs text-gray-400">— Link posts, images, text posts</span>
          </label>

          <label class="flex items-center gap-2">
            <input
              v-model="form.threadsEnabled"
              type="checkbox"
              class="form-checkbox"
              :disabled="!form.feedEnabled"
            />
            <span class="text-sm text-gray-700">Threads</span>
            <span class="text-xs text-gray-400">— Discussion-first threads</span>
          </label>
        </div>

        <div v-if="form.feedEnabled && form.threadsEnabled">
          <label class="block text-sm font-medium text-gray-700 mb-1">Default Section</label>
          <select v-model="form.defaultSection" class="form-input w-48">
            <option value="feed">Feed</option>
            <option value="threads">Threads</option>
          </select>
          <p class="text-xs text-gray-400 mt-1">Which section visitors see first</p>
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
