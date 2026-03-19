<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ layout: 'settings', middleware: 'guards' })
useHead({ title: 'Privacy Settings' })

const UPDATE_SETTINGS_MUTATION = `
  mutation UpdateSettings($input: UpdateSettingsInput!) {
    updateSettings(input: $input) {
      id
      showNSFW
      showBots
    }
  }
`

const GET_SETTINGS_QUERY = `
  query GetUserSettings {
    getUserSettings {
      showNSFW
      showBots
    }
  }
`

interface SettingsResponse {
  getUserSettings: { showNSFW: boolean; showBots: boolean }
}

const { execute, loading, error } = useGraphQL<SettingsResponse>()
const showNSFW = ref(false)
const showBots = ref(false)
const saving = ref(false)
const success = ref(false)

async function fetchSettings (): Promise<void> {
  const result = await execute(GET_SETTINGS_QUERY)
  if (result?.getUserSettings) {
    showNSFW.value = result.getUserSettings.showNSFW
    showBots.value = result.getUserSettings.showBots
  }
}

async function saveSettings (): Promise<void> {
  saving.value = true
  success.value = false

  const { execute: exec, error: saveError } = useGraphQL()
  await exec(UPDATE_SETTINGS_MUTATION, {
    variables: {
      input: {
        showNsfw: showNSFW.value,
        showBots: showBots.value,
      },
    },
  })

  if (!saveError.value) {
    success.value = true
    setTimeout(() => { success.value = false }, 3000)
  }
  saving.value = false
}

await fetchSettings()
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-4">
      Privacy
    </h2>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchSettings" />
    <CommonLoadingSpinner v-else-if="loading" size="lg" />

    <form v-else @submit.prevent="saveSettings" class="space-y-4 max-w-md">
      <div class="space-y-3">
        <label class="flex items-center gap-2">
          <input v-model="showNSFW" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Show NSFW content</span>
        </label>

        <label class="flex items-center gap-2">
          <input v-model="showBots" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Show bot accounts in feeds</span>
        </label>
      </div>

      <div class="flex items-center gap-3">
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save' }}
        </button>
        <span v-if="success" class="text-sm text-green-600">Saved successfully.</span>
      </div>
    </form>
  </div>
</template>
