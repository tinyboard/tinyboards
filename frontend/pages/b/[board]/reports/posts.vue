<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Post Reports - b/${boardName}` })

interface PostReportView {
  id: string
  originalPostTitle: string
  reason: string
  status: string
  createdAt: string
}

const { execute, loading, error, data } = useGraphQL<{ getPostReports: PostReportView[] }>()
const { execute: executeAction, loading: actioning } = useGraphQLMutation()

const statusFilter = ref('pending')
const boardId = ref<string | null>(null)

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const POST_REPORTS_QUERY = `
  query GetPostReports($boardId: ID, $statusFilter: String, $limit: Int, $offset: Int) {
    getPostReports(boardId: $boardId, statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id originalPostTitle reason status createdAt
    }
  }
`

const RESOLVE_REPORT = `
  mutation ResolveReport($reportId: ID!, $reportType: String!) {
    resolveReport(reportId: $reportId, reportType: $reportType) { success }
  }
`

const DISMISS_REPORT = `
  mutation DismissReport($reportId: ID!, $reportType: String!) {
    dismissReport(reportId: $reportId, reportType: $reportType) { success }
  }
`

async function loadReports () {
  if (!boardId.value) return
  await execute(POST_REPORTS_QUERY, {
    variables: { boardId: boardId.value, statusFilter: statusFilter.value, limit: 20, offset: 0 },
  })
}

async function resolveReport (reportId: string) {
  await executeAction(RESOLVE_REPORT, { variables: { reportId, reportType: 'post' } })
  await loadReports()
}

async function dismissReport (reportId: string) {
  await executeAction(DISMISS_REPORT, { variables: { reportId, reportType: 'post' } })
  await loadReports()
}

function changeFilter (filter: string) {
  statusFilter.value = filter
  loadReports()
}

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
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

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const result = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (result?.board) {
    boardId.value = result.board.id
    await loadReports()
  }
})

const reports = computed(() => data.value?.getPostReports ?? [])
</script>

<template>
  <div class="p-4">
    <h2 class="text-base font-semibold text-gray-900 mb-4">
      Post Reports
    </h2>

    <div class="flex gap-2 mb-4">
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

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="reports.length === 0" class="text-sm text-gray-500">
      No post reports found.
    </div>

    <div v-else class="space-y-3">
      <div
        v-for="report in reports"
        :key="report.id"
        class="bg-white rounded-lg border border-gray-200 p-4"
      >
        <div class="flex items-start justify-between">
          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-medium text-gray-900 truncate">
              {{ report.originalPostTitle }}
            </h3>
            <p class="mt-1 text-sm text-gray-600">
              Reason: {{ report.reason }}
            </p>
            <p class="mt-1 text-xs text-gray-500">
              {{ formatDate(report.createdAt) }}
            </p>
          </div>
          <div class="ml-4 flex items-center gap-2 shrink-0">
            <template v-if="report.status === 'pending'">
              <button class="button button-sm primary" :disabled="actioning" @click="resolveReport(report.id)">
                Resolve
              </button>
              <button class="button button-sm white" :disabled="actioning" @click="dismissReport(report.id)">
                Dismiss
              </button>
            </template>
            <span
              v-else
              class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
              :class="statusBadgeClass(report.status)"
            >
              {{ report.status }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
