<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Moderators - b/${boardName}` })

interface Moderator {
  id: string
  boardId: string
  user: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  }
  createdAt: string
  permissions: number
  rank: number
  isInviteAccepted: boolean
}

interface ModeratorsResponse {
  getBoardModerators: Moderator[]
}

interface BoardSettingsResponse {
  getBoardSettings: {
    board: { id: string }
    isOwner: boolean
  }
}

const GET_MODERATORS = `
  query GetBoardModerators($boardId: ID!) {
    getBoardModerators(boardId: $boardId) {
      id boardId
      user { id name displayName avatar }
      createdAt permissions rank isInviteAccepted
    }
  }
`

const GET_BOARD_SETTINGS = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      board { id }
      isOwner
    }
  }
`

const ADD_MOD_MUTATION = `
  mutation AddModerator($boardId: ID!, $userId: ID!) {
    addModerator(boardId: $boardId, userId: $userId) { success }
  }
`

const REMOVE_MOD_MUTATION = `
  mutation RemoveBoardModerator($boardId: ID!, $userId: ID!) {
    removeBoardModerator(boardId: $boardId, userId: $userId) { success message }
  }
`

const SEARCH_USERS = `
  query SearchUsernames($query: String!, $limit: Int) {
    searchUsernames(query: $query, limit: $limit)
  }
`

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const { execute, loading, error } = useGraphQL<ModeratorsResponse>()
const { execute: executeMutation } = useGraphQLMutation()

const boardId = ref<string | null>(null)
const isOwner = ref(false)
const moderators = ref<Moderator[]>([])
const newModUsername = ref('')
const searchResults = ref<string[]>([])
const showSearch = ref(false)
const actionLoading = ref(false)

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<BoardSettingsResponse>()
  const settingsResult = await execSettings(GET_BOARD_SETTINGS, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings) {
    isOwner.value = settingsResult.getBoardSettings.isOwner
  }

  await fetchModerators()
})

async function fetchModerators () {
  if (!boardId.value) return
  const result = await execute(GET_MODERATORS, { variables: { boardId: boardId.value } })
  if (result?.getBoardModerators) {
    moderators.value = result.getBoardModerators
  }
}

let searchTimeout: ReturnType<typeof setTimeout> | null = null
async function onSearchInput () {
  if (searchTimeout) clearTimeout(searchTimeout)
  if (newModUsername.value.length < 2) {
    searchResults.value = []
    showSearch.value = false
    return
  }
  searchTimeout = setTimeout(async () => {
    const { execute: exec } = useGraphQL<{ searchUsernames: string[] }>()
    const result = await exec(SEARCH_USERS, { variables: { query: newModUsername.value, limit: 5 } })
    if (result?.searchUsernames) {
      searchResults.value = result.searchUsernames
      showSearch.value = true
    }
  }, 300)
}

function selectUser (username: string) {
  newModUsername.value = username
  showSearch.value = false
}

async function addModerator () {
  if (!boardId.value || !newModUsername.value.trim()) return
  actionLoading.value = true

  const { execute: execUser } = useGraphQL<{ user: { id: string } }>()
  const userResult = await execUser(`
    query GetUser($username: String!) { user(username: $username) { id } }
  `, { variables: { username: newModUsername.value.trim() } })

  if (!userResult?.user) {
    toast.error('User not found')
    actionLoading.value = false
    return
  }

  await executeMutation(ADD_MOD_MUTATION, {
    variables: { boardId: boardId.value, userId: userResult.user.id },
  })

  toast.success(`${newModUsername.value} added as moderator`)
  newModUsername.value = ''
  await fetchModerators()
  actionLoading.value = false
}

async function removeModerator (userId: string, username: string) {
  if (!boardId.value) return
  if (!confirm(`Remove ${username} as moderator?`)) return

  actionLoading.value = true
  await executeMutation(REMOVE_MOD_MUTATION, {
    variables: { boardId: boardId.value, userId },
  })
  toast.success(`${username} removed as moderator`)
  await fetchModerators()
  actionLoading.value = false
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}
</script>

<template>
  <div>
    <!-- Settings sub-navigation -->
    <div class="flex gap-1 border-b border-gray-200 mb-4">
      <NuxtLink
        :to="`/b/${boardName}/settings`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
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
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-blue-600 text-blue-600"
      >
        Moderation
      </NuxtLink>
    </div>

    <h2 class="text-base font-semibold text-gray-900 mb-4">
      Moderators
    </h2>

    <CommonLoadingSpinner v-if="loading && moderators.length === 0" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="fetchModerators" />

    <div v-else class="max-w-2xl">
      <div v-if="isOwner" class="mb-6">
        <label class="block text-sm font-medium text-gray-700 mb-1">Add Moderator</label>
        <div class="relative">
          <div class="flex gap-2">
            <input
              v-model="newModUsername"
              type="text"
              class="form-input flex-1"
              placeholder="Search by username..."
              @input="onSearchInput"
              @keydown.enter.prevent="addModerator"
            >
            <button
              class="button button-sm primary"
              :disabled="actionLoading || !newModUsername.trim()"
              @click="addModerator"
            >
              Add
            </button>
          </div>
          <div
            v-if="showSearch && searchResults.length > 0"
            class="absolute z-10 mt-1 w-full bg-white border border-gray-200 rounded-md shadow-lg"
          >
            <button
              v-for="username in searchResults"
              :key="username"
              class="block w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-50"
              @click="selectUser(username)"
            >
              @{{ username }}
            </button>
          </div>
        </div>
      </div>

      <div class="space-y-3">
        <div
          v-for="mod in moderators"
          :key="mod.id"
          class="flex items-center justify-between p-3 bg-white border rounded-lg"
        >
          <div class="flex items-center gap-3">
            <CommonAvatar
              :src="mod.user.avatar ?? undefined"
              :name="mod.user.displayName || mod.user.name"
              size="sm"
            />
            <div>
              <div class="text-sm font-medium text-gray-900">
                {{ mod.user.displayName || mod.user.name }}
              </div>
              <div class="text-xs text-gray-500">
                @{{ mod.user.name }}
                <span v-if="mod.rank === 0" class="ml-1 text-blue-600 font-medium">Owner</span>
                <span class="ml-1">&middot; Added {{ formatDate(mod.createdAt) }}</span>
              </div>
            </div>
          </div>

          <button
            v-if="isOwner && mod.rank !== 0"
            class="button button-sm text-red-600 border-red-200 hover:bg-red-50"
            :disabled="actionLoading"
            @click="removeModerator(mod.user.id, mod.user.name)"
          >
            Remove
          </button>
        </div>
      </div>

      <p v-if="moderators.length === 0 && !loading" class="text-sm text-gray-500 py-4">
        No moderators found.
      </p>
    </div>
  </div>
</template>
