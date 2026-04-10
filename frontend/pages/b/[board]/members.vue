<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useBoard } from '~/composables/useBoard'
import { useAuthStore } from '~/stores/auth'
import { timeAgo } from '~/utils/date'

const route = useRoute()
const boardName = route.params.board as string
const authStore = useAuthStore()

const { board } = useBoard()

useHead({ title: `Members - ${boardName}` })

// ---------- types ----------

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

interface BoardContributor {
  user: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  }
  postScore: number
  commentScore: number
  totalScore: number
}

interface WikiContributor {
  user: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  }
  editCount: number
}

interface BannedUser {
  id: string
  user: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  }
  banDate: string
  expires: string | null
}

// ---------- queries ----------

const MODERATORS_QUERY = `
  query GetBoardModerators($boardId: ID!) {
    getBoardModerators(boardId: $boardId) {
      id boardId
      user { id name displayName avatar isAdmin }
      createdAt permissions rank isInviteAccepted
    }
  }
`

const TOP_CONTRIBUTORS_QUERY = `
  query GetTopContributors($boardId: ID!, $limit: Int) {
    getTopContributors(boardId: $boardId, limit: $limit) {
      user { id name displayName avatar }
      postScore commentScore totalScore
    }
  }
`

const WIKI_CONTRIBUTORS_QUERY = `
  query GetWikiContributors($boardId: ID!, $limit: Int) {
    getWikiContributors(boardId: $boardId, limit: $limit) {
      user { id name displayName avatar }
      editCount
    }
  }
`

const BANNED_USERS_QUERY = `
  query GetBoardBannedUsers($boardId: ID!, $limit: Int) {
    getBoardBannedUsers(boardId: $boardId, limit: $limit) {
      id
      user { id name displayName avatar }
      banDate expires
    }
  }
`

// ---------- state ----------

const loading = ref(true)
const moderators = ref<BoardModerator[]>([])
const topContributors = ref<BoardContributor[]>([])
const wikiContributors = ref<WikiContributor[]>([])
const bannedUsers = ref<BannedUser[]>([])
const isMod = ref(false)
const bannedExpanded = ref(false)

const wikiEnabled = computed(() => board.value?.wikiEnabled ?? false)
const subscriberCount = computed(() => board.value?.subscribers ?? 0)
const boardCreatedAt = computed(() => board.value?.createdAt ?? '')

// ---------- fetch ----------

onMounted(async () => {
  if (!board.value?.id) {
    loading.value = false
    return
  }

  const boardId = board.value.id

  // Fire all public queries in parallel.
  const modsPromise = (async () => {
    const { execute } = useGraphQL<{ getBoardModerators: BoardModerator[] }>()
    const res = await execute(MODERATORS_QUERY, { variables: { boardId } })
    if (res?.getBoardModerators) {
      moderators.value = res.getBoardModerators.filter(m => m.isInviteAccepted)
    }
  })()

  const contribPromise = (async () => {
    const { execute } = useGraphQL<{ getTopContributors: BoardContributor[] }>()
    const res = await execute(TOP_CONTRIBUTORS_QUERY, { variables: { boardId, limit: 10 } })
    if (res?.getTopContributors) {
      topContributors.value = res.getTopContributors
    }
  })()

  const wikiPromise = wikiEnabled.value
    ? (async () => {
        const { execute } = useGraphQL<{ getWikiContributors: WikiContributor[] }>()
        const res = await execute(WIKI_CONTRIBUTORS_QUERY, { variables: { boardId, limit: 10 } })
        if (res?.getWikiContributors) {
          wikiContributors.value = res.getWikiContributors
        }
      })()
    : Promise.resolve()

  // Banned users — only if the user might be a mod (logged in).
  // The query itself enforces permissions; if the user is not a mod the API
  // returns an error which we silently swallow.
  const bannedPromise = authStore.isLoggedIn
    ? (async () => {
        try {
          const { execute } = useGraphQL<{ getBoardBannedUsers: BannedUser[] }>()
          const res = await execute(BANNED_USERS_QUERY, { variables: { boardId, limit: 25 } })
          if (res?.getBoardBannedUsers) {
            bannedUsers.value = res.getBoardBannedUsers
            isMod.value = true
          }
        } catch {
          // Not a mod — hide the section.
        }
      })()
    : Promise.resolve()

  await Promise.all([modsPromise, contribPromise, wikiPromise, bannedPromise])
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
    <!-- Stat bar -->
    <div class="flex items-center gap-3 bg-white rounded-lg border border-gray-200 px-4 py-3 mb-5 text-sm text-gray-600">
      <div class="flex items-center gap-1.5">
        <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        <span class="font-medium text-gray-900">{{ subscriberCount.toLocaleString() }}</span>
        {{ subscriberCount === 1 ? 'subscriber' : 'subscribers' }}
      </div>
      <span v-if="boardCreatedAt" class="text-gray-300">&middot;</span>
      <div v-if="boardCreatedAt" class="flex items-center gap-1.5">
        <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        Created {{ timeAgo(boardCreatedAt) }}
      </div>
    </div>

    <CommonLoadingSpinner v-if="loading" />

    <template v-else>
      <!-- ============ Moderators ============ -->
      <section class="mb-6">
        <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3 px-1">
          Moderators
        </h3>
        <div v-if="moderators.length > 0" class="space-y-2">
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
                &middot; Since {{ formatDate(mod.createdAt) }}
              </p>
            </div>
          </div>
        </div>
        <p v-else class="text-sm text-gray-400 px-1">No moderators.</p>
      </section>

      <!-- ============ Top Contributors ============ -->
      <section v-if="topContributors.length > 0" class="mb-6">
        <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3 px-1">
          Top Contributors
          <span class="font-normal normal-case tracking-normal text-gray-300 ml-1">last 30 days</span>
        </h3>
        <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
          <div
            v-for="(contributor, idx) in topContributors"
            :key="contributor.user.id"
            class="flex items-center gap-3 px-3 py-2.5"
          >
            <span class="w-5 text-center text-xs font-semibold text-gray-400">#{{ idx + 1 }}</span>
            <CommonAvatar :src="contributor.user.avatar ?? undefined" :name="contributor.user.name" size="sm" />
            <div class="flex-1 min-w-0">
              <NuxtLink :to="`/@${contributor.user.name}`" class="text-sm font-medium text-gray-900 hover:underline">
                {{ contributor.user.displayName || contributor.user.name }}
              </NuxtLink>
            </div>
            <div class="flex items-center gap-2 text-xs text-gray-500 shrink-0">
              <span title="Post score">{{ contributor.postScore.toLocaleString() }} post</span>
              <span class="text-gray-300">&middot;</span>
              <span title="Comment score">{{ contributor.commentScore.toLocaleString() }} comment</span>
              <span class="text-gray-300">&middot;</span>
              <span class="font-medium text-gray-700" title="Total score">{{ contributor.totalScore.toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- ============ Banned Members (mod-only, collapsible) ============ -->
      <section v-if="isMod" class="mb-6">
        <button
          class="flex items-center gap-1.5 text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3 px-1 hover:text-gray-600 transition-colors"
          @click="bannedExpanded = !bannedExpanded"
        >
          <svg
            class="w-3.5 h-3.5 transition-transform"
            :class="bannedExpanded ? 'rotate-90' : ''"
            fill="none" stroke="currentColor" viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
          Banned Members
          <span v-if="bannedUsers.length > 0" class="font-normal normal-case tracking-normal text-gray-300 ml-0.5">({{ bannedUsers.length }})</span>
        </button>

        <template v-if="bannedExpanded">
          <div v-if="bannedUsers.length > 0" class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
            <div
              v-for="ban in bannedUsers"
              :key="ban.id"
              class="flex items-center gap-3 px-3 py-2.5"
            >
              <CommonAvatar :src="ban.user.avatar ?? undefined" :name="ban.user.name" size="sm" />
              <div class="flex-1 min-w-0">
                <NuxtLink :to="`/@${ban.user.name}`" class="text-sm font-medium text-gray-900 hover:underline">
                  {{ ban.user.displayName || ban.user.name }}
                </NuxtLink>
                <p class="text-xs text-gray-500">
                  Banned {{ formatDate(ban.banDate) }}
                  <template v-if="ban.expires">
                    &middot; Expires {{ formatDate(ban.expires) }}
                  </template>
                  <template v-else>
                    &middot; Permanent
                  </template>
                </p>
              </div>
            </div>
          </div>
          <p v-else class="text-sm text-gray-400 px-1">No banned members.</p>
        </template>
      </section>

      <!-- ============ Wiki Contributors ============ -->
      <section v-if="wikiEnabled && wikiContributors.length > 0" class="mb-6">
        <h3 class="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3 px-1">
          Wiki Contributors
        </h3>
        <div class="bg-white rounded-lg border border-gray-200 divide-y divide-gray-100">
          <div
            v-for="contributor in wikiContributors"
            :key="contributor.user.id"
            class="flex items-center gap-3 px-3 py-2.5"
          >
            <CommonAvatar :src="contributor.user.avatar ?? undefined" :name="contributor.user.name" size="sm" />
            <div class="flex-1 min-w-0">
              <NuxtLink :to="`/@${contributor.user.name}`" class="text-sm font-medium text-gray-900 hover:underline">
                {{ contributor.user.displayName || contributor.user.name }}
              </NuxtLink>
            </div>
            <span class="text-xs text-gray-500 shrink-0">
              {{ contributor.editCount }} {{ contributor.editCount === 1 ? 'edit' : 'edits' }}
            </span>
          </div>
        </div>
      </section>

      <!-- Empty state -->
      <div
        v-if="moderators.length === 0 && topContributors.length === 0 && subscriberCount === 0"
        class="text-center py-8"
      >
        <p class="text-sm text-gray-500">No members yet.</p>
      </div>
    </template>
  </div>
</template>
