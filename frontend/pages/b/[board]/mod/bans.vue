<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Bans - b/${boardName}` })

interface BannedUser {
  id: string
  user: { id: string; name: string; displayName: string | null; avatar: string | null }
  boardId: string
  banDate: string
  expires: string | null
}

const boardId = ref<string | null>(null)
const isMod = ref(false)
const bannedUsers = ref<BannedUser[]>([])
const loadingBans = ref(true)

const showBanForm = ref(false)
const searchQuery = ref('')
const searchResults = ref<string[]>([])
const searchTimeout = ref<ReturnType<typeof setTimeout> | null>(null)

const banForm = reactive({
  username: '',
  reason: '',
  duration: 'permanent',
})
const banning = ref(false)

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

const BANNED_USERS_QUERY = `
  query GetBoardBannedUsers($boardId: ID!, $page: Int, $limit: Int) {
    getBoardBannedUsers(boardId: $boardId, page: $page, limit: $limit) {
      id
      user { id name displayName avatar }
      boardId
      banDate
      expires
    }
  }
`

const SEARCH_USERNAMES_QUERY = `
  query SearchUsernames($query: String!, $limit: Int) {
    searchUsernames(query: $query, limit: $limit)
  }
`

const USER_QUERY = `
  query GetUser($username: String!) {
    user(username: $username) { id }
  }
`

const BAN_USER_MUTATION = `
  mutation BanUserFromBoard($input: BoardBanUserInput!) {
    banUserFromBoard(input: $input) {
      success
      banId
      message
    }
  }
`

const UNBAN_USER_MUTATION = `
  mutation UnbanUserFromBoard($boardId: ID!, $userId: ID!) {
    unbanUserFromBoard(boardId: $boardId, userId: $userId) {
      success
      message
    }
  }
`

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  } else {
    await navigateTo(`/b/${boardName}`)
    return
  }

  await loadBannedUsers()
})

async function loadBannedUsers () {
  if (!boardId.value) return
  loadingBans.value = true

  const { execute } = useGraphQL<{ getBoardBannedUsers: BannedUser[] }>()
  const result = await execute(BANNED_USERS_QUERY, { variables: { boardId: boardId.value, page: 1, limit: 50 } })
  bannedUsers.value = result?.getBoardBannedUsers ?? []
  loadingBans.value = false
}

function onSearchInput () {
  if (searchTimeout.value) clearTimeout(searchTimeout.value)
  if (searchQuery.value.length < 2) {
    searchResults.value = []
    return
  }
  searchTimeout.value = setTimeout(async () => {
    const { execute } = useGraphQL<{ searchUsernames: string[] }>()
    const result = await execute(SEARCH_USERNAMES_QUERY, { variables: { query: searchQuery.value, limit: 5 } })
    searchResults.value = result?.searchUsernames ?? []
  }, 300)
}

function selectUser (username: string) {
  banForm.username = username
  searchQuery.value = username
  searchResults.value = []
}

function getExpiresDays (): number | null {
  switch (banForm.duration) {
    case '1day': return 1
    case '1week': return 7
    case '1month': return 30
    default: return null
  }
}

async function handleBan () {
  if (!boardId.value || !banForm.username) return

  banning.value = true

  // Resolve username to user ID
  const { execute: execUser } = useGraphQL<{ user: { id: string } }>()
  const userResult = await execUser(USER_QUERY, { variables: { username: banForm.username } })
  if (!userResult?.user) {
    toast.error('User not found')
    banning.value = false
    return
  }

  const { execute: execBan } = useGraphQLMutation<{ banUserFromBoard: { success: boolean; banId: string; message: string } }>()
  const result = await execBan(BAN_USER_MUTATION, {
    variables: {
      input: {
        userId: userResult.user.id,
        boardId: boardId.value,
        reason: banForm.reason || null,
        expiresDays: getExpiresDays(),
      },
    },
  })
  banning.value = false

  if (result?.banUserFromBoard?.success) {
    toast.success(`${banForm.username} banned from b/${boardName}`)
    banForm.username = ''
    banForm.reason = ''
    banForm.duration = 'permanent'
    searchQuery.value = ''
    showBanForm.value = false
    await loadBannedUsers()
  } else {
    toast.error('Failed to ban user')
  }
}

async function handleUnban (ban: BannedUser) {
  if (!boardId.value) return

  const { execute } = useGraphQLMutation()
  await execute(UNBAN_USER_MUTATION, {
    variables: { boardId: boardId.value, userId: ban.user.id },
  })

  bannedUsers.value = bannedUsers.value.filter(b => b.id !== ban.id)
  toast.success(`${ban.user.name} unbanned from b/${boardName}`)
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function formatExpiry (expires: string | null): string {
  if (!expires) return 'Permanent'
  const date = new Date(expires)
  if (date < new Date()) return 'Expired'
  return formatDate(expires)
}
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 py-4">
    <!-- Breadcrumb -->
    <nav class="text-sm text-gray-500 mb-2">
      <NuxtLink :to="`/b/${boardName}`" class="hover:text-primary no-underline">b/{{ boardName }}</NuxtLink>
      <span class="mx-1">/</span>
      <span class="text-gray-700">Moderation</span>
    </nav>

    <!-- Header card -->
    <div class="bg-white rounded-lg border border-gray-200 mb-4 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-200 flex items-center justify-between">
        <h1 class="text-lg font-semibold text-gray-900">Board Bans</h1>
        <button
          v-if="!showBanForm"
          class="button primary button-sm"
          @click="showBanForm = true"
        >
          Ban user
        </button>
      </div>
      <div class="px-4 flex gap-0.5">
        <NuxtLink
          :to="`/b/${boardName}/mod/queue`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-transparent text-gray-500 hover:text-gray-700"
        >
          Reports
        </NuxtLink>
        <NuxtLink
          :to="`/b/${boardName}/mod/log`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-transparent text-gray-500 hover:text-gray-700"
        >
          Mod Log
        </NuxtLink>
        <NuxtLink
          :to="`/b/${boardName}/mod/bans`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-primary text-primary"
        >
          Bans
        </NuxtLink>
      </div>
    </div>

    <!-- Ban form -->
    <div v-if="showBanForm" class="bg-white rounded-lg border border-gray-200 p-4 mb-6">
      <h3 class="text-sm font-medium text-gray-900 mb-4">Ban User</h3>
      <form class="space-y-4" @submit.prevent="handleBan">
        <div class="relative">
          <label class="block text-sm font-medium text-gray-700 mb-1">Username</label>
          <input
            v-model="searchQuery"
            type="text"
            class="form-input w-full"
            placeholder="Search for a user..."
            @input="onSearchInput"
          />
          <div
            v-if="searchResults.length > 0"
            class="absolute z-10 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg"
          >
            <button
              v-for="username in searchResults"
              :key="username"
              type="button"
              class="w-full text-left px-3 py-2 text-sm hover:bg-gray-50"
              @click="selectUser(username)"
            >
              {{ username }}
            </button>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
          <textarea v-model="banForm.reason" rows="2" class="form-input w-full" placeholder="Ban reason (optional)" />
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Duration</label>
          <select v-model="banForm.duration" class="form-input w-full">
            <option value="1day">1 Day</option>
            <option value="1week">1 Week</option>
            <option value="1month">1 Month</option>
            <option value="permanent">Permanent</option>
          </select>
        </div>

        <div class="flex gap-3">
          <button type="submit" class="button primary button-sm" :disabled="banning || !banForm.username">
            {{ banning ? 'Banning...' : 'Ban User' }}
          </button>
          <button type="button" class="button white button-sm" @click="showBanForm = false">Cancel</button>
        </div>
      </form>
    </div>

    <CommonLoadingSpinner v-if="loadingBans" />

    <div v-else-if="bannedUsers.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500">No banned users.</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="ban in bannedUsers"
        :key="ban.id"
        class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
      >
        <div class="flex items-center gap-3">
          <div class="h-8 w-8 rounded-full bg-gray-200 overflow-hidden">
            <img v-if="ban.user.avatar" :src="ban.user.avatar" :alt="ban.user.name" class="h-full w-full object-cover" />
          </div>
          <div>
            <p class="text-sm font-medium text-gray-900">
              {{ ban.user.displayName || ban.user.name }}
              <span class="text-gray-500 font-normal">@{{ ban.user.name }}</span>
            </p>
            <p class="text-xs text-gray-500">
              Banned {{ formatDate(ban.banDate) }}
              &middot; Expires: {{ formatExpiry(ban.expires) }}
            </p>
          </div>
        </div>
        <button class="button white button-sm" @click="handleUnban(ban)">
          Unban
        </button>
      </div>
    </div>
  </div>
</template>
