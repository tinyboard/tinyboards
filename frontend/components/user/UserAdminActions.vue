<script setup lang="ts">
import { ref, computed } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'
import { useAuthStore } from '~/stores/auth'
import type { User } from '~/types/generated'

const props = defineProps<{
  user: User
}>()

const emit = defineEmits<{
  (e: 'updated'): void
}>()

const toast = useToast()
const authStore = useAuthStore()
const { execute, loading } = useGraphQL()

const showBanModal = ref(false)
const showAdminModal = ref(false)
const banReason = ref('')
const banDays = ref<number | null>(null)
const selectedAdminLevel = ref(0)

const isAdmin = computed(() => authStore.user?.isAdmin)
const myAdminLevel = computed(() => authStore.user?.adminLevel ?? 0)
const canManageUser = computed(() => {
  if (!isAdmin.value) return false
  // Cannot manage users at your level or above
  if (props.user.isAdmin && props.user.adminLevel >= myAdminLevel.value) return false
  return true
})

const canBan = computed(() => isAdmin.value && myAdminLevel.value >= 3 && canManageUser.value)
const canSetAdmin = computed(() => isAdmin.value && myAdminLevel.value >= 6 && canManageUser.value)

const BAN_MUTATION = `
  mutation BanUserFromSite($input: BanUserInput!) {
    banUserFromSite(input: $input) { success message }
  }
`

const UNBAN_MUTATION = `
  mutation UnbanUserFromSite($userId: ID!, $reason: String) {
    unbanUserFromSite(userId: $userId, reason: $reason) { success message }
  }
`

const SET_ADMIN_MUTATION = `
  mutation SetUserAdminLevel($userId: ID!, $adminLevel: Int!) {
    setUserAdminLevel(userId: $userId, adminLevel: $adminLevel) {
      id name isAdmin adminLevel
    }
  }
`

async function banUser () {
  const input: Record<string, unknown> = { userId: props.user.id }
  if (banReason.value) input.reason = banReason.value
  if (banDays.value && banDays.value > 0) input.expiresDays = banDays.value

  const result = await execute(BAN_MUTATION, { variables: { input } })
  if (result) {
    toast.success(`${props.user.name} has been banned`)
    showBanModal.value = false
    banReason.value = ''
    banDays.value = null
    emit('updated')
  }
}

async function unbanUser () {
  if (!confirm(`Unban ${props.user.name}?`)) return
  const result = await execute(UNBAN_MUTATION, {
    variables: { userId: props.user.id },
  })
  if (result) {
    toast.success(`${props.user.name} has been unbanned`)
    emit('updated')
  }
}

function openAdminModal () {
  selectedAdminLevel.value = props.user.adminLevel
  showAdminModal.value = true
}

async function setAdminLevel () {
  const result = await execute(SET_ADMIN_MUTATION, {
    variables: { userId: props.user.id, adminLevel: selectedAdminLevel.value },
  })
  if (result) {
    const label = selectedAdminLevel.value === 0 ? 'Admin privileges removed' : `Admin level set to ${selectedAdminLevel.value}`
    toast.success(`${props.user.name}: ${label}`)
    showAdminModal.value = false
    emit('updated')
  }
}

function adminLevelLabel (level: number): string {
  switch (level) {
    case 0: return 'None (regular user)'
    case 1: return 'Level 1 - Appearance'
    case 2: return 'Level 2 - Config'
    case 3: return 'Level 3 - Content Mod'
    case 4: return 'Level 4 - User Management'
    case 5: return 'Level 5 - Board Management'
    case 6: return 'Level 6 - Full Admin'
    case 7: return 'Level 7 - Owner'
    default: return `Level ${level}`
  }
}
</script>

<template>
  <div v-if="isAdmin && (canBan || canSetAdmin)" class="border border-red-200 rounded-lg p-4 bg-red-50/50">
    <h3 class="text-sm font-semibold text-red-800 mb-3 flex items-center gap-1.5">
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      Admin Actions
    </h3>

    <div class="flex flex-wrap gap-2">
      <!-- Ban / Unban -->
      <template v-if="canBan">
        <button
          v-if="user.isBanned"
          class="button button-sm bg-green-600 text-white hover:bg-green-700"
          :disabled="loading"
          @click="unbanUser"
        >
          Unban User
        </button>
        <button
          v-else
          class="button button-sm bg-red-600 text-white hover:bg-red-700"
          :disabled="loading"
          @click="showBanModal = true"
        >
          Ban User
        </button>
      </template>

      <!-- Admin Level -->
      <button
        v-if="canSetAdmin"
        class="button button-sm bg-blue-600 text-white hover:bg-blue-700"
        :disabled="loading"
        @click="openAdminModal"
      >
        {{ user.isAdmin ? 'Change Admin Level' : 'Make Admin' }}
      </button>
    </div>

    <!-- Current status badges -->
    <div class="mt-3 flex flex-wrap gap-2 text-xs">
      <span v-if="user.isBanned" class="inline-flex items-center px-2 py-0.5 rounded font-medium bg-red-100 text-red-800">
        Banned
        <template v-if="user.unbanDate"> (expires {{ new Date(user.unbanDate).toLocaleDateString() }})</template>
      </span>
      <span v-if="user.isAdmin" class="inline-flex items-center px-2 py-0.5 rounded font-medium bg-blue-100 text-blue-800">
        Admin Level {{ user.adminLevel }}
      </span>
    </div>

    <!-- Ban Modal -->
    <div v-if="showBanModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showBanModal = false">
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <h3 class="text-lg font-semibold mb-4">Ban {{ user.displayName || user.name }}</h3>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Reason (optional)</label>
          <textarea
            v-model="banReason"
            class="form-input w-full"
            rows="3"
            placeholder="Reason for banning..."
          />
        </div>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Duration</label>
          <select v-model="banDays" class="form-select w-full">
            <option :value="null">Permanent</option>
            <option :value="1">1 day</option>
            <option :value="3">3 days</option>
            <option :value="7">7 days</option>
            <option :value="14">14 days</option>
            <option :value="30">30 days</option>
            <option :value="90">90 days</option>
            <option :value="365">1 year</option>
          </select>
        </div>

        <div class="flex justify-end gap-2">
          <button class="button gray button-sm" @click="showBanModal = false">Cancel</button>
          <button
            class="button button-sm bg-red-600 text-white hover:bg-red-700"
            :disabled="loading"
            @click="banUser"
          >
            Confirm Ban
          </button>
        </div>
      </div>
    </div>

    <!-- Admin Level Modal -->
    <div v-if="showAdminModal" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showAdminModal = false">
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <h3 class="text-lg font-semibold mb-4">Set Admin Level for {{ user.displayName || user.name }}</h3>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Admin Level</label>
          <select v-model="selectedAdminLevel" class="form-select w-full">
            <option v-for="level in Array.from({ length: myAdminLevel }, (_, i) => i)" :key="level" :value="level">
              {{ adminLevelLabel(level) }}
            </option>
          </select>
          <p class="text-xs text-gray-500 mt-1">
            You can assign levels below your own (Level {{ myAdminLevel }}).
          </p>
        </div>

        <div class="flex justify-end gap-2">
          <button class="button gray button-sm" @click="showAdminModal = false">Cancel</button>
          <button
            class="button button-sm bg-blue-600 text-white hover:bg-blue-700"
            :disabled="loading"
            @click="setAdminLevel"
          >
            Save
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
