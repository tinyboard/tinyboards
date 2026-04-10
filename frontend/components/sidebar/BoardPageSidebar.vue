<script setup lang="ts">
import { useBoard } from '~/composables/useBoard'
import { useBoardActivity } from '~/composables/useBoardActivity'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import { formatDate, timeAgo } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'

const route = useRoute()
const authStore = useAuthStore()

const boardName = computed(() => {
  const params = route.params
  if (typeof params.board === 'string') return params.board
  return ''
})

const { board, loading, fetchBoard } = useBoard()

watch(boardName, async (name) => {
  if (name) await fetchBoard(name)
}, { immediate: true })

// Check mod status
const isMod = ref(false)

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

onMounted(async () => {
  if (!board.value || !authStore.isLoggedIn) return

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: board.value.id } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  }
})

// Mode-aware create button
const createButtonLabel = computed(() => {
  if (board.value?.mode === 'forum') return 'New Discussion'
  return 'Create Post'
})

const createButtonLink = computed(() => {
  if (!board.value) return ''
  if (board.value.mode === 'forum') return `/b/${board.value.name}/submit?type=thread`
  return `/b/${board.value.name}/submit`
})

// Latest activity for forum boards
const isForumBoard = computed(() => board.value?.mode === 'forum')
const activityComposable = boardName.value ? useBoardActivity(boardName.value) : null
const latestActivity = computed(() => activityComposable?.latestActivity.value ?? [])
const activityLoading = computed(() => activityComposable?.loading.value ?? false)

watch(isForumBoard, async (isForum) => {
  if (isForum && activityComposable) {
    await activityComposable.fetchActivity()
  }
}, { immediate: true })
</script>

<template>
  <div class="space-y-4">
    <template v-if="board && !loading">
      <!-- Create button — context-aware, matches board primary color -->
      <NuxtLink
        v-if="authStore.isLoggedIn"
        :to="createButtonLink"
        class="button primary w-full text-center no-underline flex items-center justify-center gap-2 text-sm"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        {{ createButtonLabel }}
      </NuxtLink>

      <!-- About section -->
      <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div class="px-4 py-2 bg-primary text-white">
          <h3 class="font-semibold text-sm">About b/{{ board.name }}</h3>
        </div>
        <div class="p-4 space-y-3">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div
            v-if="board.sidebarHTML"
            class="prose prose-sm text-gray-700 max-w-none"
            v-html="sanitizeHtml(board.sidebarHTML)"
          />
          <p v-else-if="board.description" class="text-sm text-gray-600">
            {{ board.description }}
          </p>

          <!-- Board creation date -->
          <div class="flex items-center gap-2 text-xs text-gray-500 pt-2 border-t border-gray-100">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            Board since {{ formatDate(board.createdAt) }}
          </div>
        </div>
      </div>

      <!-- Moderation section -->
      <div v-if="isMod" class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div class="px-4 py-2 border-b border-gray-200">
          <h3 class="font-semibold text-sm text-gray-900">Moderation</h3>
        </div>
        <nav class="p-2">
          <NuxtLink
            :to="`/b/${board.name}/settings`"
            class="flex items-center gap-2 px-2 py-1.5 text-sm text-primary no-underline hover:bg-gray-50 rounded transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Settings
          </NuxtLink>
          <NuxtLink
            :to="`/b/${board.name}/mod/queue`"
            class="flex items-center gap-2 px-2 py-1.5 text-sm text-primary no-underline hover:bg-gray-50 rounded transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
            </svg>
            Mod Queue
          </NuxtLink>
          <NuxtLink
            :to="`/b/${board.name}/mod/log`"
            class="flex items-center gap-2 px-2 py-1.5 text-sm text-primary no-underline hover:bg-gray-50 rounded transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01" />
            </svg>
            Moderation Log
          </NuxtLink>
        </nav>
      </div>

      <!-- Stats -->
      <div class="bg-white rounded-lg border border-gray-200 p-4">
        <div class="grid grid-cols-2 gap-3 text-center sm:text-left">
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ board.subscribers }}</div>
            <div class="text-[11px] text-gray-500">Members</div>
          </div>
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums flex items-center gap-1.5">
              {{ board.usersActiveDay ?? 0 }}
              <span class="inline-block w-2 h-2 rounded-full bg-green-400" />
            </div>
            <div class="text-[11px] text-gray-500">Online</div>
          </div>
          <div>
            <div class="text-lg font-bold text-gray-900 tabular-nums">{{ board.posts }}</div>
            <div class="text-[11px] text-gray-500">Posts</div>
          </div>
          <div>
            <div class="text-sm font-medium text-gray-900">{{ formatDate(board.createdAt) }}</div>
            <div class="text-[11px] text-gray-500">Created</div>
          </div>
        </div>
      </div>

      <!-- Latest Activity (forum boards only) -->
      <div v-if="isForumBoard" class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div class="px-4 py-2 border-b border-gray-200 flex items-center gap-2">
          <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
          </svg>
          <h3 class="font-semibold text-sm text-gray-900">Latest Activity</h3>
        </div>
        <div v-if="activityLoading" class="p-3 space-y-3">
          <div v-for="n in 3" :key="n" class="flex items-center gap-2.5">
            <div class="w-5 h-5 rounded-full bg-gray-200 animate-pulse shrink-0" />
            <div class="flex-1 space-y-1">
              <div class="h-3 bg-gray-200 rounded animate-pulse w-3/4" />
              <div class="h-2.5 bg-gray-100 rounded animate-pulse w-1/3" />
            </div>
          </div>
        </div>
        <div v-else-if="latestActivity.length === 0" class="p-4 text-center">
          <p class="text-xs text-gray-400">No recent activity</p>
        </div>
        <div v-else class="divide-y divide-gray-100">
          <NuxtLink
            v-for="entry in latestActivity"
            :key="entry.postId"
            :to="`/b/${board.name}/${entry.postId}/${entry.postSlug || ''}`"
            class="flex items-center gap-2.5 px-4 py-2.5 hover:bg-gray-50 transition-colors no-underline"
          >
            <CommonAvatar
              :src="entry.commenterAvatar ?? undefined"
              :name="entry.commenterName"
              size="xs"
            />
            <div class="flex-1 min-w-0">
              <p class="text-sm text-gray-800 font-medium truncate leading-tight">{{ entry.postTitle }}</p>
              <p class="text-[11px] text-gray-400 leading-tight mt-0.5">
                {{ entry.commenterName }} &middot; {{ timeAgo(entry.createdAt) }}
              </p>
            </div>
          </NuxtLink>
        </div>
      </div>
    </template>

    <!-- Loading state -->
    <div v-else-if="loading" class="space-y-4">
      <div class="bg-white rounded-lg border border-gray-200 h-32 animate-pulse" />
      <div class="bg-white rounded-lg border border-gray-200 h-24 animate-pulse" />
    </div>
  </div>
</template>
