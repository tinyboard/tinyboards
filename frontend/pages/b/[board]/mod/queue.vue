<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Mod Queue - b/${boardName}` })

interface PostReportView {
  id: string
  postId: string
  originalPostTitle: string
  reason: string
  status: string
  createdAt: string
}

interface CommentReportView {
  id: string
  commentId: string
  originalCommentText: string
  reason: string
  status: string
  createdAt: string
}

const activeTab = ref<'posts' | 'comments'>('posts')
const statusFilter = ref('pending')
const boardId = ref<string | null>(null)
const isMod = ref(false)

const postReports = ref<PostReportView[]>([])
const commentReports = ref<CommentReportView[]>([])
const loadingReports = ref(false)

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

const POST_REPORTS_QUERY = `
  query GetPostReports($boardId: ID, $statusFilter: String, $limit: Int, $offset: Int) {
    getPostReports(boardId: $boardId, statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id postId originalPostTitle reason status createdAt
    }
  }
`

const COMMENT_REPORTS_QUERY = `
  query GetCommentReports($boardId: ID, $statusFilter: String, $limit: Int, $offset: Int) {
    getCommentReports(boardId: $boardId, statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id commentId originalCommentText reason status createdAt
    }
  }
`

const RESOLVE_REPORT_MUTATION = `
  mutation ResolveReport($reportId: ID!, $reportType: String!) {
    resolveReport(reportId: $reportId, reportType: $reportType) { success }
  }
`

const DISMISS_REPORT_MUTATION = `
  mutation DismissReport($reportId: ID!, $reportType: String!) {
    dismissReport(reportId: $reportId, reportType: $reportType) { success }
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

  await loadReports()
})

async function loadReports () {
  if (!boardId.value) return
  loadingReports.value = true

  const variables = {
    boardId: boardId.value,
    statusFilter: statusFilter.value,
    limit: 20,
    offset: 0,
  }

  if (activeTab.value === 'posts') {
    const { execute } = useGraphQL<{ getPostReports: PostReportView[] }>()
    const result = await execute(POST_REPORTS_QUERY, { variables })
    postReports.value = result?.getPostReports ?? []
  } else {
    const { execute } = useGraphQL<{ getCommentReports: CommentReportView[] }>()
    const result = await execute(COMMENT_REPORTS_QUERY, { variables })
    commentReports.value = result?.getCommentReports ?? []
  }

  loadingReports.value = false
}

async function switchTab (tab: 'posts' | 'comments') {
  activeTab.value = tab
  await loadReports()
}

async function changeFilter (filter: string) {
  statusFilter.value = filter
  await loadReports()
}

async function resolveReport (reportId: string, type: string) {
  const { execute } = useGraphQL()
  await execute(RESOLVE_REPORT_MUTATION, { variables: { reportId, reportType: type } })
  toast.success('Report resolved')
  await loadReports()
}

async function dismissReport (reportId: string, type: string) {
  const { execute } = useGraphQL()
  await execute(DISMISS_REPORT_MUTATION, { variables: { reportId, reportType: type } })
  toast.success('Report dismissed')
  await loadReports()
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

function statusBadgeClass (status: string): string {
  switch (status) {
    case 'pending': return 'bg-yellow-100 text-yellow-800'
    case 'resolved': return 'bg-green-100 text-green-800'
    case 'dismissed': return 'bg-gray-100 text-gray-800'
    default: return 'bg-gray-100 text-gray-800'
  }
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
        <h1 class="text-lg font-semibold text-gray-900">Moderation Queue</h1>
      </div>
      <div class="px-4 flex gap-0.5 border-b border-gray-100">
        <NuxtLink
          :to="`/b/${boardName}/mod/queue`"
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-primary text-primary"
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
          class="px-3 py-2 text-sm font-medium no-underline border-b-2 -mb-px transition-colors border-transparent text-gray-500 hover:text-gray-700"
        >
          Bans
        </NuxtLink>
      </div>
    </div>

    <!-- Filter bar -->
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 mb-4 flex items-center gap-3">
      <div class="flex gap-1">
        <button
          class="button button-sm"
          :class="activeTab === 'posts' ? 'primary' : 'white'"
          @click="switchTab('posts')"
        >
          Post Reports
        </button>
        <button
          class="button button-sm"
          :class="activeTab === 'comments' ? 'primary' : 'white'"
          @click="switchTab('comments')"
        >
          Comment Reports
        </button>
      </div>
      <div class="w-px h-6 bg-gray-200" />
      <div class="flex gap-1">
        <button
          v-for="filter in ['pending', 'resolved', 'dismissed']"
          :key="filter"
          class="button button-sm"
          :class="statusFilter === filter ? 'primary' : 'white'"
          @click="changeFilter(filter)"
        >
          {{ filter.charAt(0).toUpperCase() + filter.slice(1) }}
        </button>
      </div>
    </div>

    <CommonLoadingSpinner v-if="loadingReports" />

    <!-- Post reports -->
    <template v-else-if="activeTab === 'posts'">
      <div v-if="postReports.length === 0" class="text-sm text-gray-500">
        No post reports found.
      </div>
      <div v-else class="space-y-4">
        <div
          v-for="report in postReports"
          :key="report.id"
          class="bg-white rounded-lg border border-gray-200 p-4"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-medium text-gray-900 truncate">
                {{ report.originalPostTitle }}
              </h3>
              <p class="mt-1 text-sm text-gray-600">Reason: {{ report.reason }}</p>
              <p class="mt-1 text-xs text-gray-500">{{ formatDate(report.createdAt) }}</p>
            </div>
            <div class="ml-4 flex items-center gap-2">
              <span
                class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
                :class="statusBadgeClass(report.status)"
              >
                {{ report.status }}
              </span>
              <template v-if="report.status === 'pending'">
                <button class="button button-sm white text-green-700" @click="resolveReport(report.id, 'post')">
                  Resolve
                </button>
                <button class="button button-sm white text-gray-600" @click="dismissReport(report.id, 'post')">
                  Dismiss
                </button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Comment reports -->
    <template v-else>
      <div v-if="commentReports.length === 0" class="text-sm text-gray-500">
        No comment reports found.
      </div>
      <div v-else class="space-y-4">
        <div
          v-for="report in commentReports"
          :key="report.id"
          class="bg-white rounded-lg border border-gray-200 p-4"
        >
          <div class="flex items-start justify-between">
            <div class="flex-1 min-w-0">
              <p class="text-sm text-gray-900 line-clamp-2">{{ report.originalCommentText }}</p>
              <p class="mt-1 text-sm text-gray-600">Reason: {{ report.reason }}</p>
              <p class="mt-1 text-xs text-gray-500">{{ formatDate(report.createdAt) }}</p>
            </div>
            <div class="ml-4 flex items-center gap-2">
              <span
                class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
                :class="statusBadgeClass(report.status)"
              >
                {{ report.status }}
              </span>
              <template v-if="report.status === 'pending'">
                <button class="button button-sm white text-green-700" @click="resolveReport(report.id, 'comment')">
                  Resolve
                </button>
                <button class="button button-sm white text-gray-600" @click="dismissReport(report.id, 'comment')">
                  Dismiss
                </button>
              </template>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
