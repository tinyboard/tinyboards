<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Moderation Queue' })

interface PostReportView {
  id: string
  originalPostTitle: string
  reason: string
  status: string
  createdAt: string
}

interface CommentReportView {
  id: string
  originalCommentText: string
  reason: string
  status: string
  createdAt: string
}

interface PostReportsResponse {
  getPostReports: PostReportView[]
}

interface CommentReportsResponse {
  getCommentReports: CommentReportView[]
}

const activeTab = ref<'posts' | 'comments'>('posts')
const statusFilter = ref('pending')
const limit = ref(20)
const offset = ref(0)

const { execute: fetchPostReports, loading: loadingPosts, error: postError, data: postData } = useGraphQL<PostReportsResponse>()
const { execute: fetchCommentReports, loading: loadingComments, error: commentError, data: commentData } = useGraphQL<CommentReportsResponse>()
const { execute: executeAction, loading: actioning } = useGraphQLMutation<{ resolveReport: { success: boolean } | null, dismissReport: { success: boolean } | null }>()

const POST_REPORTS_QUERY = `
  query GetPostReports($statusFilter: String, $limit: Int, $offset: Int) {
    getPostReports(statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id
      originalPostTitle
      reason
      status
      createdAt
    }
  }
`

const COMMENT_REPORTS_QUERY = `
  query GetCommentReports($statusFilter: String, $limit: Int, $offset: Int) {
    getCommentReports(statusFilter: $statusFilter, limit: $limit, offset: $offset) {
      id
      originalCommentText
      reason
      status
      createdAt
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
  offset.value = 0
  const variables = {
    statusFilter: statusFilter.value,
    limit: limit.value,
    offset: offset.value,
  }

  if (activeTab.value === 'posts') {
    await fetchPostReports(POST_REPORTS_QUERY, { variables })
  } else {
    await fetchCommentReports(COMMENT_REPORTS_QUERY, { variables })
  }
}

async function resolveReport (reportId: string) {
  const reportType = activeTab.value === 'posts' ? 'post' : 'comment'
  await executeAction(RESOLVE_REPORT, { variables: { reportId, reportType } })
  await loadReports()
}

async function dismissReport (reportId: string) {
  const reportType = activeTab.value === 'posts' ? 'post' : 'comment'
  await executeAction(DISMISS_REPORT, { variables: { reportId, reportType } })
  await loadReports()
}

async function switchTab (tab: 'posts' | 'comments') {
  activeTab.value = tab
  offset.value = 0
  await loadReports()
}

async function changeFilter (filter: string) {
  statusFilter.value = filter
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

onMounted(() => {
  loadReports()
})

const postReports = computed(() => postData.value?.getPostReports ?? [])
const commentReports = computed(() => commentData.value?.getCommentReports ?? [])
const isLoading = computed(() => loadingPosts.value || loadingComments.value)
const currentError = computed(() => activeTab.value === 'posts' ? postError.value : commentError.value)
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Moderation Queue
    </h2>

    <!-- Tabs -->
    <div class="flex gap-2 mb-4">
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

    <!-- Status filter -->
    <div class="flex gap-2 mb-6">
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

    <CommonLoadingSpinner v-if="isLoading" />
    <CommonErrorDisplay v-else-if="currentError" :message="currentError.message" />

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
              <p class="text-sm text-gray-900 line-clamp-2">
                {{ report.originalCommentText }}
              </p>
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
    </template>
  </div>
</template>
