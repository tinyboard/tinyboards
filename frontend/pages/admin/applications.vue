<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ layout: 'admin', middleware: 'guards' })
useHead({ title: 'Registration Applications' })

const toast = useToast()

interface RegistrationApp {
  id: string
  userId: string
  answer: string
  adminId: string | null
  denyReason: string | null
  createdAt: string
}

const APPS_QUERY = `
  query ListRegistrationApplications($limit: Int, $offset: Int) {
    listRegistrationApplications(limit: $limit, offset: $offset) {
      id userId answer adminId denyReason createdAt
    }
  }
`

const APPROVE_MUTATION = `
  mutation ApproveApplication($applicationId: ID!) {
    approveApplication(applicationId: $applicationId)
  }
`

const DENY_MUTATION = `
  mutation DenyApplication($applicationId: ID!, $reason: String) {
    denyApplication(applicationId: $applicationId, reason: $reason)
  }
`

const applications = ref<RegistrationApp[]>([])
const loading = ref(true)
const denyingId = ref<string | null>(null)
const denyReason = ref('')

async function loadApplications (): Promise<void> {
  loading.value = true
  const { execute } = useGraphQL<{ listRegistrationApplications: RegistrationApp[] }>()
  const result = await execute(APPS_QUERY, { variables: { limit: 50, offset: 0 } })
  applications.value = result?.listRegistrationApplications ?? []
  loading.value = false
}

onMounted(loadApplications)

const pendingApps = computed(() =>
  applications.value.filter(a => a.adminId === null),
)

const resolvedApps = computed(() =>
  applications.value.filter(a => a.adminId !== null),
)

async function approveApp (appId: string): Promise<void> {
  const { execute } = useGraphQL()
  const result = await execute(APPROVE_MUTATION, { variables: { applicationId: appId } })
  if (result) {
    toast.success('Application approved')
    await loadApplications()
  } else {
    toast.error('Failed to approve application')
  }
}

async function denyApp (appId: string): Promise<void> {
  const { execute } = useGraphQL()
  const result = await execute(DENY_MUTATION, {
    variables: { applicationId: appId, reason: denyReason.value || null },
  })
  if (result) {
    toast.success('Application denied')
    denyingId.value = null
    denyReason.value = ''
    await loadApplications()
  } else {
    toast.error('Failed to deny application')
  }
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
</script>

<template>
  <div>
    <h2 class="text-base font-semibold text-gray-900 mb-4">Registration Applications</h2>

    <CommonLoadingSpinner v-if="loading" />

    <div v-else-if="applications.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500">No registration applications.</p>
    </div>

    <template v-else>
      <!-- Pending applications -->
      <div v-if="pendingApps.length > 0" class="mb-8">
        <h3 class="text-sm font-medium text-gray-700 mb-3">
          Pending ({{ pendingApps.length }})
        </h3>
        <div class="space-y-3">
          <div
            v-for="app in pendingApps"
            :key="app.id"
            class="bg-white border border-gray-200 rounded-lg p-4"
          >
            <div class="flex items-start justify-between">
              <div class="flex-1 min-w-0">
                <p class="text-xs text-gray-500 mb-2">
                  Applied {{ formatDate(app.createdAt) }}
                </p>
                <div class="bg-gray-50 rounded p-3 mb-3">
                  <p class="text-sm font-medium text-gray-700 mb-1">Application Answer:</p>
                  <p class="text-sm text-gray-600 whitespace-pre-wrap">{{ app.answer }}</p>
                </div>

                <!-- Deny reason input -->
                <div v-if="denyingId === app.id" class="mb-3">
                  <label class="block text-sm font-medium text-gray-700 mb-1">Denial Reason (optional)</label>
                  <input v-model="denyReason" type="text" class="form-input" placeholder="Reason for denial..." />
                </div>

                <div class="flex gap-2">
                  <button class="button button-sm primary" @click="approveApp(app.id)">
                    Approve
                  </button>
                  <button
                    v-if="denyingId !== app.id"
                    class="button button-sm bg-red-600 text-white hover:bg-red-700"
                    @click="denyingId = app.id"
                  >
                    Deny
                  </button>
                  <template v-else>
                    <button
                      class="button button-sm bg-red-600 text-white hover:bg-red-700"
                      @click="denyApp(app.id)"
                    >
                      Confirm Deny
                    </button>
                    <button class="button button-sm white" @click="denyingId = null; denyReason = ''">
                      Cancel
                    </button>
                  </template>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Resolved applications -->
      <div v-if="resolvedApps.length > 0">
        <h3 class="text-sm font-medium text-gray-700 mb-3">
          Resolved ({{ resolvedApps.length }})
        </h3>
        <div class="space-y-3">
          <div
            v-for="app in resolvedApps"
            :key="app.id"
            class="bg-white border border-gray-200 rounded-lg p-4 opacity-75"
          >
            <div class="flex items-center gap-2 mb-2">
              <span
                class="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium"
                :class="app.denyReason ? 'bg-red-100 text-red-800' : 'bg-green-100 text-green-800'"
              >
                {{ app.denyReason ? 'Denied' : 'Approved' }}
              </span>
              <span class="text-xs text-gray-500">{{ formatDate(app.createdAt) }}</span>
            </div>
            <p class="text-sm text-gray-600">{{ app.answer }}</p>
            <p v-if="app.denyReason" class="text-sm text-red-600 mt-1">
              Denial reason: {{ app.denyReason }}
            </p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
