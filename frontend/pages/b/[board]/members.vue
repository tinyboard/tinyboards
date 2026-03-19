<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Members - ${boardName}` })

interface BoardModerator {
  id: string
  boardId: string
  user: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
    isAdmin: boolean
  }
  createdAt: string
  permissions: number
  rank: number
  isInviteAccepted: boolean
}

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id subscribers }
  }
`

const MODERATORS_QUERY = `
  query GetBoardModerators($boardId: ID!) {
    getBoardModerators(boardId: $boardId) {
      id
      boardId
      user {
        id name displayName avatar isAdmin
      }
      createdAt
      permissions
      rank
      isInviteAccepted
    }
  }
`

const loading = ref(true)
const moderators = ref<BoardModerator[]>([])
const subscriberCount = ref(0)

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string; subscribers: number } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) {
    loading.value = false
    return
  }

  subscriberCount.value = boardResult.board.subscribers

  const { execute: execMods } = useGraphQL<{ getBoardModerators: BoardModerator[] }>()
  const modsResult = await execMods(MODERATORS_QUERY, { variables: { boardId: boardResult.board.id } })
  if (modsResult?.getBoardModerators) {
    moderators.value = modsResult.getBoardModerators.filter(m => m.isInviteAccepted)
  }
  loading.value = false
})

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<template>
  <div>
    <div class="bg-white rounded-lg border border-gray-200 px-4 py-3 mb-4">
      <h2 class="text-base font-semibold text-gray-900">
        Members
      </h2>
      <p class="text-sm text-gray-500 mt-0.5">
        {{ subscriberCount }} {{ subscriberCount === 1 ? 'subscriber' : 'subscribers' }}
      </p>
    </div>

    <CommonLoadingSpinner v-if="loading" />

    <template v-else>
      <div v-if="moderators.length > 0" class="mb-6">
        <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3 px-1">Moderators</h3>
        <div class="space-y-2">
          <div
            v-for="mod in moderators"
            :key="mod.id"
            class="flex items-center gap-3 bg-white rounded-lg border border-gray-200 p-3"
          >
            <CommonAvatar :src="mod.user.avatar ?? undefined" :name="mod.user.name" size="md" />
            <div class="flex-1 min-w-0">
              <NuxtLink :to="`/@${mod.user.name}`" class="text-sm font-medium text-gray-900 hover:underline">
                {{ mod.user.displayName || mod.user.name }}
              </NuxtLink>
              <p class="text-xs text-gray-500">
                @{{ mod.user.name }}
                <span v-if="mod.rank === 0" class="ml-1 text-primary font-medium">Owner</span>
                <span v-else class="ml-1">Mod</span>
                &middot; Joined {{ formatDate(mod.createdAt) }}
              </p>
            </div>
          </div>
        </div>
      </div>

      <div v-if="moderators.length === 0 && subscriberCount === 0" class="text-center py-8">
        <p class="text-sm text-gray-500">No members yet.</p>
      </div>
    </template>
  </div>
</template>
