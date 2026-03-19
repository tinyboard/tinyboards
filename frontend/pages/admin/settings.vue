<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useSiteStore } from '~/stores/site'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Settings' })

const siteStore = useSiteStore()

interface SiteConfig {
  name: string
  description: string
  enableDownvotes: boolean
  enableNSFW: boolean
  registrationMode: string
  requireEmailVerification: boolean
  applicationQuestion: string
  isPrivate: boolean
  captchaEnabled: boolean
  captchaDifficulty: string
}

interface SiteResponse {
  site: SiteConfig
}

interface UpdateSiteResponse {
  updateSiteConfig: SiteConfig
}

const { execute: fetchSite, loading, error } = useGraphQL<SiteResponse>()
const { execute: executeMutation, loading: saving, error: saveError } = useGraphQLMutation<UpdateSiteResponse>()

const form = reactive({
  name: '',
  description: '',
  enableDownvotes: true,
  enableNSFW: false,
  registrationMode: 'open',
  requireEmailVerification: false,
  applicationQuestion: '',
  isPrivate: false,
  captchaEnabled: false,
  captchaDifficulty: 'medium',
})

const saveSuccess = ref(false)

const SITE_QUERY = `
  query {
    site {
      name
      description
      enableDownvotes
      enableNSFW
      registrationMode
      requireEmailVerification
      applicationQuestion
      isPrivate
      captchaEnabled
      captchaDifficulty
    }
  }
`

const UPDATE_SITE_MUTATION = `
  mutation UpdateSiteConfig($input: UpdateSiteConfigInput!) {
    updateSiteConfig(input: $input) {
      name
      description
      enableDownvotes
      enableNSFW
      registrationMode
      requireEmailVerification
      applicationQuestion
      isPrivate
      captchaEnabled
      captchaDifficulty
    }
  }
`

const registrationModes = [
  { value: 'open', label: 'Open' },
  { value: 'application_required', label: 'Require Application' },
  { value: 'invite_only', label: 'Invite Only' },
  { value: 'closed', label: 'Closed' },
]

onMounted(async () => {
  const result = await fetchSite(SITE_QUERY)
  if (result?.site) {
    Object.assign(form, result.site)
  }
})

async function saveSettings () {
  saveSuccess.value = false
  const result = await executeMutation(UPDATE_SITE_MUTATION, {
    variables: {
      input: {
        name: form.name,
        description: form.description,
        enableDownvotes: form.enableDownvotes,
        enableNsfw: form.enableNSFW,
        registrationMode: form.registrationMode,
        requireEmailVerification: form.requireEmailVerification,
        applicationQuestion: form.applicationQuestion || undefined,
        isPrivate: form.isPrivate,
        captchaEnabled: form.captchaEnabled,
        captchaDifficulty: form.captchaDifficulty,
      },
    },
  })
  if (result?.updateSiteConfig) {
    Object.assign(form, result.updateSiteConfig)
    saveSuccess.value = true
    if (siteStore.site) {
      siteStore.setSite({
        ...siteStore.site,
        name: form.name,
        registrationMode: form.registrationMode,
        enableDownvotes: form.enableDownvotes,
        enableNSFW: form.enableNSFW,
        requireEmailVerification: form.requireEmailVerification,
        isPrivate: form.isPrivate,
      })
    }
    setTimeout(() => { saveSuccess.value = false }, 3000)
  }
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Site Settings
    </h2>

    <div v-if="loading" class="text-sm text-gray-500">
      Loading settings...
    </div>

    <div v-else-if="error" class="rounded-md bg-red-50 p-4 text-sm text-red-700">
      Failed to load settings: {{ error.message }}
    </div>

    <form v-else class="space-y-8 max-w-2xl" @submit.prevent="saveSettings">
      <!-- General section -->
      <section>
        <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-4">
          General
        </h3>
        <div class="space-y-5">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Site Name</label>
            <input v-model="form.name" type="text" class="form-input w-full" />
          </div>

          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <textarea v-model="form.description" rows="3" class="form-input w-full" />
          </div>

          <div class="space-y-3">
            <label class="flex items-center gap-2">
              <input v-model="form.enableDownvotes" type="checkbox" class="form-checkbox" />
              <span class="text-sm text-gray-700">Enable Downvotes</span>
            </label>

            <label class="flex items-center gap-2">
              <input v-model="form.enableNSFW" type="checkbox" class="form-checkbox" />
              <span class="text-sm text-gray-700">Enable NSFW Content</span>
            </label>

            <label class="flex items-center gap-2">
              <input v-model="form.isPrivate" type="checkbox" class="form-checkbox" />
              <span class="text-sm text-gray-700">Private Instance</span>
            </label>
          </div>
        </div>
      </section>

      <hr class="border-gray-200" />

      <!-- Registration & Security section -->
      <section>
        <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-4">
          Registration &amp; Security
        </h3>
        <div class="space-y-5">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Registration Mode
            </label>
            <select v-model="form.registrationMode" class="form-input w-full">
              <option v-for="mode in registrationModes" :key="mode.value" :value="mode.value">
                {{ mode.label }}
              </option>
            </select>
            <p class="mt-1 text-xs text-gray-500">
              Controls how new users can register on the site.
            </p>
          </div>

          <div v-if="form.registrationMode === 'application_required'">
            <label class="block text-sm font-medium text-gray-700 mb-1">Application Question</label>
            <textarea v-model="form.applicationQuestion" rows="2" class="form-input w-full" />
          </div>

          <div class="space-y-3">
            <div>
              <label class="flex items-center gap-2">
                <input v-model="form.requireEmailVerification" type="checkbox" class="form-checkbox" />
                <span class="text-sm text-gray-700">Require Email Verification</span>
              </label>
              <p class="ml-6 text-xs text-gray-500">
                Users must verify their email address before they can post.
              </p>
            </div>

            <div>
              <label class="flex items-center gap-2">
                <input v-model="form.captchaEnabled" type="checkbox" class="form-checkbox" />
                <span class="text-sm text-gray-700">Enable Captcha</span>
              </label>
              <p class="ml-6 text-xs text-gray-500">
                Require captcha verification during registration.
              </p>
            </div>
          </div>

          <div v-if="form.captchaEnabled">
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Captcha Difficulty
            </label>
            <select v-model="form.captchaDifficulty" class="form-input w-full">
              <option value="easy">Easy</option>
              <option value="medium">Medium</option>
              <option value="hard">Hard</option>
            </select>
          </div>
        </div>
      </section>

      <hr class="border-gray-200" />

      <div v-if="saveError" class="rounded-md bg-red-50 p-4 text-sm text-red-700">
        Failed to save: {{ saveError.message }}
      </div>

      <div v-if="saveSuccess" class="rounded-md bg-green-50 p-4 text-sm text-green-700">
        Settings saved successfully.
      </div>

      <div>
        <button type="submit" class="button button-sm primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Settings' }}
        </button>
      </div>
    </form>
  </div>
</template>
