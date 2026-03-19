<script setup lang="ts">
import { useModeration } from '~/composables/useModeration'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Post Reports' })

const { postReports, loading, error, fetchPostReports, removePost, resolveReport, dismissReport } = useModeration()

const statusFilter = ref<string>('pending')
const offset = ref(0)
const limit = 20

async function loadReports () {
  offset.value = 0
  await fetchPostReports({ statusFilter: statusFilter.value, limit, offset: offset.value })
}

async function loadMore () {
  offset.value += limit
  await fetchPostReports({ statusFilter: statusFilter.value, limit, offset: offset.value })
}

async function handleResolve (reportId: string) {
  await resolveReport(reportId, 'post')
  await loadReports()
}

async function handleDismiss (reportId: string) {
  await dismissReport(reportId, 'post')
  await loadReports()
}

async function handleRemovePost (postId: string, reportId: string) {
  if (!confirm('Remove this post? It will be hidden from public view.')) return
  await removePost(postId, 'Removed via report review')
  await resolveReport(reportId, 'post')
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

onMounted(() => { loadReports() })
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Post Reports
    </h2>

    <div class="flex gap-2 mb-6">
      <button
        v-for="filter in [{ value: 'pending', label: 'Open' }, { value: 'resolved', label: 'Resolved' }, { value: '', label: 'All' }]"
        :key="filter.value"
        class="button button-sm"
        :class="statusFilter === filter.value ? 'primary' : 'white'"
        @click="changeFilter(filter.value)"
      >
        {{ filter.label }}
      </button>
    </div>

    <CommonLoadingSpinner v-if="loading" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="loadReports" />

    <div v-else-if="postReports.length === 0" class="py-12 text-center text-sm text-gray-500">
      No post reports found.
    </div>

    <div v-else class="space-y-4">
      <div
        v-for="report in postReports"
        :key="report.id"
        class="bg-white rounded-lg border border-gray-200 p-4"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <h3 class="text-sm font-medium text-gray-900 truncate">
                {{ report.originalPostTitle }}
              </h3>
              <span
                class="shrink-0 inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
                :class="statusBadgeClass(report.status)"
              >
                {{ report.status }}
              </span>
            </div>
            <p class="text-sm text-gray-600 mb-1">
              <span class="font-medium">Reason:</span> {{ report.reason }}
            </p>
            <p v-if="report.originalPostBody" class="text-xs text-gray-500 line-clamp-2 mb-1">
              {{ report.originalPostBody }}
            </p>
            <p class="text-xs text-gray-400">
              Reported {{ formatDate(report.createdAt) }}
            </p>
          </div>

          <div v-if="report.status === 'pending'" class="flex items-center gap-2 shrink-0">
            <button
              class="button button-sm primary"
              @click="handleResolve(report.id)"
            >
              Resolve
            </button>
            <button
              class="button button-sm text-red-600 border-red-200 hover:bg-red-50"
              @click="handleRemovePost(report.postId, report.id)"
            >
              Remove Post
            </button>
            <button
              class="button button-sm white"
              @click="handleDismiss(report.id)"
            >
              Dismiss
            </button>
          </div>
        </div>
      </div>

      <div v-if="postReports.length >= limit" class="flex justify-center pt-2">
        <button class="button button-sm white" @click="loadMore">
          Load More
        </button>
      </div>
    </div>
  </div>
</template>
