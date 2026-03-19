<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Mod Log - b/${boardName}` })

interface ModerationLogEntry {
  id: string
  moderatorId: string
  moderatorName: string
  actionType: string
  targetType: string
  targetId: string
  boardId: string | null
  reason: string | null
  createdAt: string
}

interface ModerationLogResponse {
  entries: ModerationLogEntry[]
  totalCount: number
}

const boardId = ref<string | null>(null)
const isMod = ref(false)
const loading = ref(true)
const entries = ref<ModerationLogEntry[]>([])
const actionFilter = ref('')

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

const MOD_LOG_QUERY = `
  query GetModerationLog($boardId: ID, $actionType: String, $limit: Int, $offset: Int) {
    getModerationLog(boardId: $boardId, actionType: $actionType, limit: $limit, offset: $offset) {
      entries {
        id moderatorId moderatorName actionType targetType targetId
        boardId reason createdAt
      }
      totalCount
    }
  }
`

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) {
    loading.value = false
    return
  }

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  } else {
    await navigateTo(`/b/${boardName}`)
    return
  }

  await loadLog()
})

async function loadLog () {
  if (!boardId.value) return
  loading.value = true

  const { execute } = useGraphQL<{ getModerationLog: ModerationLogResponse }>()
  const result = await execute(MOD_LOG_QUERY, {
    variables: {
      boardId: boardId.value,
      actionType: actionFilter.value || null,
      limit: 50,
      offset: 0,
    },
  })

  entries.value = result?.getModerationLog?.entries ?? []
  loading.value = false
}

async function changeFilter (filter: string) {
  actionFilter.value = filter
  await loadLog()
}

const actionTypes = [
  { value: '', label: 'All' },
  { value: 'ban_from_board', label: 'Board Bans' },
  { value: 'unban_from_board', label: 'Board Unbans' },
  { value: 'remove_post', label: 'Post Removals' },
  { value: 'restore_post', label: 'Post Restorations' },
  { value: 'remove_comment', label: 'Comment Removals' },
  { value: 'restore_comment', label: 'Comment Restorations' },
  { value: 'lock_post', label: 'Post Locks' },
  { value: 'feature_post', label: 'Featured Posts' },
  { value: 'add_mod', label: 'Mod Added' },
  { value: 'remove_mod', label: 'Mod Removed' },
]

function formatAction (actionType: string): string {
  return actionType.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

function actionBadgeClass (actionType: string): string {
  if (actionType.startsWith('ban') || actionType.startsWith('remove') || actionType.startsWith('lock')) {
    return 'bg-red-100 text-red-800'
  }
  if (actionType.startsWith('unban') || actionType.startsWith('restore') || actionType.startsWith('unlock')) {
    return 'bg-green-100 text-green-800'
  }
  return 'bg-gray-100 text-gray-800'
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

    <!-- Header card with tabs -->
    <div class="bg-white rounded-lg border border-gray-200 mb-4 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-200">
        <h1 class="text-lg font-semibold text-gray-900">Moderation Log</h1>
      </div>
      <div class="px-4 flex gap-0.5 border-b border-gray-100">
        <NuxtLink
          :to="`/b/${boardName}/mod/queue`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-transparent text-gray-500 hover:text-gray-700"
        >
          Reports
        </NuxtLink>
        <NuxtLink
          :to="`/b/${boardName}/mod/log`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-primary text-primary"
        >
          Mod Log
        </NuxtLink>
        <NuxtLink
          :to="`/b/${boardName}/mod/bans`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-transparent text-gray-500 hover:text-gray-700"
        >
          Bans
        </NuxtLink>
      </div>
    </div>

    <!-- Action type filter -->
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex flex-wrap gap-2 mb-4">
      <button
        v-for="at in actionTypes"
        :key="at.value"
        class="button button-sm"
        :class="actionFilter === at.value ? 'primary' : 'white'"
        @click="changeFilter(at.value)"
      >
        {{ at.label }}
      </button>
    </div>

    <CommonLoadingSpinner v-if="loading" />

    <div v-else-if="entries.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500">No moderation actions recorded.</p>
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="bg-white rounded-lg border border-gray-200 p-4"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <span
                class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
                :class="actionBadgeClass(entry.actionType)"
              >
                {{ formatAction(entry.actionType) }}
              </span>
              <span class="text-xs text-gray-500">{{ entry.targetType }}</span>
            </div>
            <p class="text-sm text-gray-900">
              by <span class="font-medium">{{ entry.moderatorName }}</span>
            </p>
            <p v-if="entry.reason" class="text-sm text-gray-600 mt-0.5">
              Reason: {{ entry.reason }}
            </p>
          </div>
          <span class="text-xs text-gray-500 flex-shrink-0">
            {{ formatDate(entry.createdAt) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>
