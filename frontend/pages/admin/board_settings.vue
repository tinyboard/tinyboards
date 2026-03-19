<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Board Settings' })

interface BoardSettings {
  boardCreationMode: string
  boardsEnabled: boolean
  boardCreationAdminOnly: boolean
}

interface BoardSettingsResponse {
  site: BoardSettings
}

interface SaveBoardSettingsResponse {
  updateSiteConfig: BoardSettings
}

const { execute, loading, error, data } = useGraphQL<BoardSettingsResponse>()
const { execute: executeSave, loading: saving, error: saveError } = useGraphQLMutation<SaveBoardSettingsResponse>()

const boardCreationMode = ref('open')
const boardsEnabled = ref(true)
const boardCreationAdminOnly = ref(false)
const saved = ref(false)

const SETTINGS_QUERY = `
  query {
    site {
      boardCreationMode
      boardsEnabled
      boardCreationAdminOnly
    }
  }
`

const SAVE_SETTINGS = `
  mutation UpdateSiteConfig($input: UpdateSiteConfigInput!) {
    updateSiteConfig(input: $input) {
      boardCreationMode
      boardsEnabled
      boardCreationAdminOnly
    }
  }
`

async function loadSettings () {
  const result = await execute(SETTINGS_QUERY)
  if (result?.site) {
    boardCreationMode.value = result.site.boardCreationMode
    boardsEnabled.value = result.site.boardsEnabled
    boardCreationAdminOnly.value = result.site.boardCreationAdminOnly
  }
}

async function saveSettings () {
  saved.value = false
  await executeSave(SAVE_SETTINGS, {
    variables: {
      input: {
        boardCreationMode: boardCreationMode.value,
        boardsEnabled: boardsEnabled.value,
        boardCreationAdminOnly: boardCreationAdminOnly.value,
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
      Board Defaults
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
          <option value="open">Open (anyone can create)</option>
          <option value="restricted">Restricted (approval required)</option>
          <option value="closed">Closed (admin only)</option>
        </select>
        <p class="mt-1 text-xs text-gray-500">
          Controls who can create new boards on the site.
        </p>
      </div>

      <!-- Admin-only board creation -->
      <div>
        <label class="flex items-center gap-2">
          <input v-model="boardCreationAdminOnly" type="checkbox" class="form-checkbox" />
          <span class="text-sm font-medium text-gray-700">Admin-Only Board Creation</span>
        </label>
        <p class="mt-1 text-xs text-gray-500">
          When enabled, only admins can create new boards regardless of the creation mode above.
        </p>
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
