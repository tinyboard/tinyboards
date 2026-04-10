<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Board Settings' })

interface BoardSettings {
  boardCreationMode: string
  boardsEnabled: boolean
  boardCreationAdminOnly: boolean
  trustedUserMinReputation: number
  trustedUserMinAccountAgeDays: number
  trustedUserManualApproval: boolean
  trustedUserMinPosts: number
}

interface BoardSettingsResponse {
  site: BoardSettings
}

interface SaveBoardSettingsResponse {
  updateSiteConfig: BoardSettings
}

const { execute, loading, error, data } = useGraphQL<BoardSettingsResponse>()
const { execute: executeSave, loading: saving, error: saveError } = useGraphQLMutation<SaveBoardSettingsResponse>()

const boardCreationMode = ref('AdminOnly')
const boardsEnabled = ref(true)
const trustedUserMinReputation = ref(0)
const trustedUserMinAccountAgeDays = ref(0)
const trustedUserManualApproval = ref(false)
const trustedUserMinPosts = ref(0)
const saved = ref(false)

const SETTINGS_QUERY = `
  query {
    site {
      boardCreationMode
      boardsEnabled
      boardCreationAdminOnly
      trustedUserMinReputation
      trustedUserMinAccountAgeDays
      trustedUserManualApproval
      trustedUserMinPosts
    }
  }
`

const SAVE_SETTINGS = `
  mutation UpdateSiteConfig($input: UpdateSiteConfigInput!) {
    updateSiteConfig(input: $input) {
      boardCreationMode
      boardsEnabled
      boardCreationAdminOnly
      trustedUserMinReputation
      trustedUserMinAccountAgeDays
      trustedUserManualApproval
      trustedUserMinPosts
    }
  }
`

async function loadSettings () {
  const result = await execute(SETTINGS_QUERY)
  if (result?.site) {
    boardCreationMode.value = result.site.boardCreationMode
    boardsEnabled.value = result.site.boardsEnabled
    trustedUserMinReputation.value = result.site.trustedUserMinReputation
    trustedUserMinAccountAgeDays.value = result.site.trustedUserMinAccountAgeDays
    trustedUserManualApproval.value = result.site.trustedUserManualApproval
    trustedUserMinPosts.value = result.site.trustedUserMinPosts
  }
}

async function saveSettings () {
  saved.value = false
  await executeSave(SAVE_SETTINGS, {
    variables: {
      input: {
        boardCreationMode: boardCreationMode.value,
        boardsEnabled: boardsEnabled.value,
        ...(boardCreationMode.value === 'TrustedUsers' ? {
          trustedUserMinReputation: trustedUserMinReputation.value,
          trustedUserMinAccountAgeDays: trustedUserMinAccountAgeDays.value,
          trustedUserManualApproval: trustedUserManualApproval.value,
          trustedUserMinPosts: trustedUserMinPosts.value,
        } : {}),
      },
    },
  })
  if (!saveError.value) {
    saved.value = true
    setTimeout(() => { saved.value = false }, 3000)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Board Settings
    </h2>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <form v-else class="space-y-6 max-w-lg" @submit.prevent="saveSettings">
      <!-- Boards enabled -->
      <div>
        <label class="flex items-center gap-2">
          <input v-model="boardsEnabled" type="checkbox" class="form-checkbox" />
          <span class="text-sm font-medium text-gray-700">Boards Enabled</span>
        </label>
        <p class="mt-1 text-xs text-gray-500">
          When disabled, the site operates as a single community without sub-boards.
        </p>
      </div>

      <!-- Board creation mode -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">
          Board Creation Mode
        </label>
        <select v-model="boardCreationMode" class="form-input w-full">
          <option value="Open">Open (anyone can create)</option>
          <option value="TrustedUsers">Trusted Users (configurable requirements)</option>
          <option value="AdminOnly">Admin Only</option>
          <option value="Disabled">Disabled (no one can create)</option>
        </select>
        <p class="mt-1 text-xs text-gray-500">
          Controls who can create new boards on the site.
        </p>
      </div>

      <!-- Trusted Users settings (shown only when TrustedUsers mode is selected) -->
      <template v-if="boardCreationMode === 'TrustedUsers'">
        <div class="rounded-lg border border-gray-200 bg-gray-50 p-4 space-y-4">
          <h3 class="text-sm font-semibold text-gray-800">
            Trusted User Requirements
          </h3>
          <p class="text-xs text-gray-500">
            Users must meet all of these requirements to create boards. Admins always bypass these checks.
          </p>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Minimum Reputation (post + comment score)
            </label>
            <input
              v-model.number="trustedUserMinReputation"
              type="number"
              min="0"
              class="form-input w-full"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Minimum Account Age (days)
            </label>
            <input
              v-model.number="trustedUserMinAccountAgeDays"
              type="number"
              min="0"
              class="form-input w-full"
            />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Minimum Posts
            </label>
            <input
              v-model.number="trustedUserMinPosts"
              type="number"
              min="0"
              class="form-input w-full"
            />
          </div>

          <div>
            <label class="flex items-center gap-2">
              <input v-model="trustedUserManualApproval" type="checkbox" class="form-checkbox" />
              <span class="text-sm font-medium text-gray-700">Require Manual Approval</span>
            </label>
            <p class="mt-1 text-xs text-gray-500">
              When enabled, users must also be manually approved by an admin before they can create boards,
              even if they meet the automatic requirements above.
            </p>
          </div>
        </div>
      </template>

      <!-- Default board mode -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Default Board Mode</label>
        <p class="text-xs text-gray-500 mb-3">
          Pre-selected mode when users create a new board. This is a default, not a restriction.
        </p>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            class="text-left rounded-lg border-2 p-4 transition-all"
            :class="defaultBoardMode === 'feed'
              ? 'border-blue-600 bg-blue-50 ring-1 ring-blue-600'
              : 'border-gray-200 bg-white hover:border-gray-300'"
            @click="defaultBoardMode = 'feed'"
          >
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-lg">📰</span>
              <span class="font-semibold text-sm text-gray-900">Feed Board</span>
            </div>
            <p class="text-xs text-gray-500 leading-relaxed">
              Share links, images, and text posts. Members vote on content.
            </p>
          </button>
          <button
            type="button"
            class="text-left rounded-lg border-2 p-4 transition-all"
            :class="defaultBoardMode === 'forum'
              ? 'border-blue-600 bg-blue-50 ring-1 ring-blue-600'
              : 'border-gray-200 bg-white hover:border-gray-300'"
            @click="defaultBoardMode = 'forum'"
          >
            <div class="flex items-center gap-2 mb-1.5">
              <span class="text-lg">💬</span>
              <span class="font-semibold text-sm text-gray-900">Forum Board</span>
            </div>
            <p class="text-xs text-gray-500 leading-relaxed">
              Threaded discussions. Great for Q&amp;A, support, or structured topics.
            </p>
          </button>
        </div>
      </div>

      <CommonErrorDisplay v-if="saveError" :message="saveError.message" />

      <div class="flex items-center gap-3">
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Settings' }}
        </button>
        <span v-if="saved" class="text-sm text-green-600">
          Settings saved successfully.
        </span>
      </div>
    </form>
  </div>
</template>
