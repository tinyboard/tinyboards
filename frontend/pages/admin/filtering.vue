<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ layout: 'admin', middleware: 'guards' })
useHead({ title: 'Content Filtering' })

const toast = useToast()

const SITE_QUERY = `
  query GetSite {
    site {
      wordFilterEnabled
      filteredWords
    }
  }
`

const UPDATE_CONFIG = `
  mutation UpdateSiteConfig($input: UpdateSiteConfigInput!) {
    updateSiteConfig(input: $input) {
      wordFilterEnabled
      filteredWords
    }
  }
`

const loading = ref(true)
const saving = ref(false)

const form = reactive({
  wordFilterEnabled: false,
  filteredWords: '',
  linkFilterEnabled: false,
  bannedDomains: '',
})

onMounted(async () => {
  const { execute } = useGraphQL<{ site: { wordFilterEnabled: boolean; filteredWords: string | null } }>()
  const result = await execute(SITE_QUERY)
  if (result?.site) {
    form.wordFilterEnabled = result.site.wordFilterEnabled
    form.filteredWords = result.site.filteredWords ?? ''
  }
  loading.value = false
})

async function saveFilters (): Promise<void> {
  saving.value = true

  const { execute } = useGraphQLMutation()
  const result = await execute(UPDATE_CONFIG, {
    variables: {
      input: {
        wordFilterEnabled: form.wordFilterEnabled,
        filteredWords: form.filteredWords || null,
        linkFilterEnabled: form.linkFilterEnabled,
        bannedDomains: form.bannedDomains || null,
      },
    },
  })

  if (result) {
    toast.success('Content filtering settings saved')
  } else {
    toast.error('Failed to save settings')
  }
  saving.value = false
}

const wordCount = computed(() => {
  if (!form.filteredWords.trim()) return 0
  return form.filteredWords.split(',').filter(w => w.trim()).length
})

const domainCount = computed(() => {
  if (!form.bannedDomains.trim()) return 0
  return form.bannedDomains.split(',').filter(d => d.trim()).length
})
</script>

<template>
  <div>
    <h2 class="text-base font-semibold text-gray-900 mb-4">Content Filtering</h2>

    <CommonLoadingSpinner v-if="loading" />

    <form v-else class="space-y-6 max-w-2xl" @submit.prevent="saveFilters">
      <!-- Word Filter -->
      <div class="bg-white border border-gray-200 rounded-lg p-5">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-medium text-gray-900">Word Filter</h3>
          <label class="flex items-center gap-2">
            <input v-model="form.wordFilterEnabled" type="checkbox" class="form-checkbox" />
            <span class="text-sm text-gray-700">Enabled</span>
          </label>
        </div>
        <p class="text-xs text-gray-500 mb-3">
          Posts and comments containing these words will be automatically filtered. Separate words with commas.
        </p>
        <textarea
          v-model="form.filteredWords"
          :disabled="!form.wordFilterEnabled"
          class="form-input w-full font-mono text-sm"
          rows="4"
          placeholder="word1, word2, phrase to filter, ..."
        />
        <p class="text-xs text-gray-400 mt-1">{{ wordCount }} word{{ wordCount === 1 ? '' : 's' }} configured</p>
      </div>

      <!-- Link / Domain Filter -->
      <div class="bg-white border border-gray-200 rounded-lg p-5">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm font-medium text-gray-900">Domain Filter</h3>
          <label class="flex items-center gap-2">
            <input v-model="form.linkFilterEnabled" type="checkbox" class="form-checkbox" />
            <span class="text-sm text-gray-700">Enabled</span>
          </label>
        </div>
        <p class="text-xs text-gray-500 mb-3">
          Links to these domains will be blocked in posts and comments. Separate domains with commas.
        </p>
        <textarea
          v-model="form.bannedDomains"
          :disabled="!form.linkFilterEnabled"
          class="form-input w-full font-mono text-sm"
          rows="4"
          placeholder="spam-site.com, bad-domain.org, ..."
        />
        <p class="text-xs text-gray-400 mt-1">{{ domainCount }} domain{{ domainCount === 1 ? '' : 's' }} configured</p>
      </div>

      <div>
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Filtering Settings' }}
        </button>
      </div>
    </form>
  </div>
</template>
