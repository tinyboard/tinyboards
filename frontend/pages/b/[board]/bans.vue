<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Banned Users - b/${boardName}` })

interface BannedUser {
  id: string
  user: {
    id: string
    name: string
    displayName: string
    avatar: string | null
  }
  boardId: string
  banDate: string
  expires: string | null
}

const { execute, loading, error, data } = useGraphQL<{ getBoardBannedUsers: BannedUser[] }>()
const { execute: executeUnban, loading: unbanning } = useGraphQLMutation<{ unbanUserFromBoard: { success: boolean } }>()

const boardId = ref<string | null>(null)
const page = ref(1)
const limit = 25

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
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

const UNBAN_MUTATION = `
  mutation UnbanUserFromBoard($userId: ID!, $boardId: ID!) {
    unbanUserFromBoard(userId: $userId, boardId: $boardId) { success message }
  }
`

async function loadBans () {
  if (!boardId.value) return
  await execute(BANNED_USERS_QUERY, {
    variables: { boardId: boardId.value, page: page.value, limit },
  })
}

async function unbanUser (userId: string) {
  if (!boardId.value) return
  await executeUnban(UNBAN_MUTATION, { variables: { userId, boardId: boardId.value } })
  await loadBans()
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric', month: 'short', day: 'numeric',
  })
}

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const result = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (result?.board) {
    boardId.value = result.board.id
    await loadBans()
  }
})

const bans = computed(() => data.value?.getBoardBannedUsers ?? [])
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 py-4">
    <nav class="text-sm text-gray-500 mb-2">
      <NuxtLink :to="`/b/${boardName}`" class="hover:text-primary no-underline">b/{{ boardName }}</NuxtLink>
      <span class="mx-1">/</span>
      <span class="text-gray-700">Bans</span>
    </nav>

    <div class="bg-white rounded-lg border border-gray-200 mb-4 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-200">
        <h1 class="text-lg font-semibold text-gray-900">Banned Users</h1>
      </div>
    </div>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="bans.length === 0" class="text-sm text-gray-500">
      No banned users in this board.
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="ban in bans"
        :key="ban.id"
        class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
      >
        <div class="flex items-center gap-3">
          <CommonAvatar
            :src="ban.user.avatar ?? undefined"
            :name="ban.user.displayName || ban.user.name"
            size="sm"
          />
          <div>
            <div class="text-sm font-medium text-gray-900">
              {{ ban.user.displayName || ban.user.name }}
            </div>
            <div class="text-xs text-gray-500">
              @{{ ban.user.name }}
              &middot; Banned {{ formatDate(ban.banDate) }}
              <template v-if="ban.expires">
                &middot; Expires {{ formatDate(ban.expires) }}
              </template>
              <template v-else>
                &middot; Permanent
              </template>
            </div>
          </div>
        </div>
        <button
          class="button button-sm white shrink-0"
          :disabled="unbanning"
          @click="unbanUser(ban.user.id)"
        >
          Unban
        </button>
      </div>

      <CommonPagination
        :page="page"
        :has-more="bans.length === limit"
        @prev="page > 1 && (page--, loadBans())"
        @next="page++; loadBans()"
      />
    </div>
  </div>
</template>
