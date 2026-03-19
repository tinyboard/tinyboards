<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Edit Flair' })

const route = useRoute()
const router = useRouter()
const flairId = route.params.id as string

interface FlairTemplate {
  id: string
  boardId: string
  flairType: string
  templateName: string
  textDisplay: string
  textColor: string
  backgroundColor: string
  isModOnly: boolean
  isActive: boolean
}

const { execute, loading, error } = useGraphQL<{ flairTemplate: FlairTemplate | null }>()
const { execute: executeUpdate, loading: saving, error: saveError } = useGraphQLMutation<{ updateFlairTemplate: FlairTemplate }>()

const form = reactive({
  templateName: '',
  textDisplay: '',
  textColor: '#000000',
  backgroundColor: '#e0e0e0',
  isModOnly: false,
  isActive: true,
})

const FLAIR_QUERY = `
  query GetFlairTemplate($id: ID!) {
    flairTemplate(id: $id) {
      id boardId flairType templateName textDisplay textColor backgroundColor isModOnly isActive
    }
  }
`

const UPDATE_FLAIR = `
  mutation UpdateFlairTemplate($templateId: ID!, $input: UpdateFlairTemplateInput!) {
    updateFlairTemplate(templateId: $templateId, input: $input) {
      id templateName
    }
  }
`

async function saveFlair () {
  const result = await executeUpdate(UPDATE_FLAIR, {
    variables: {
      templateId: flairId,
      input: {
        templateName: form.templateName,
        textDisplay: form.textDisplay,
        textColor: form.textColor,
        backgroundColor: form.backgroundColor,
        isModOnly: form.isModOnly,
        isActive: form.isActive,
      },
    },
  })

  if (result?.updateFlairTemplate) {
    await router.push('/admin/flairs')
  }
}

onMounted(async () => {
  const result = await execute(FLAIR_QUERY, { variables: { id: flairId } })
  if (result?.flairTemplate) {
    const f = result.flairTemplate
    form.templateName = f.templateName
    form.textDisplay = f.textDisplay
    form.textColor = f.textColor
    form.backgroundColor = f.backgroundColor
    form.isModOnly = f.isModOnly
    form.isActive = f.isActive
  }
})
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Edit Flair
    </h2>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <form v-else class="space-y-4 max-w-lg" @submit.prevent="saveFlair">
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Name</label>
        <input v-model="form.templateName" type="text" class="form-input w-full" required />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Display Text</label>
        <input v-model="form.textDisplay" type="text" class="form-input w-full" />
      </div>

      <div class="flex gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
          <input v-model="form.textColor" type="color" class="h-10 w-16 p-1 border border-gray-300 rounded" />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Background</label>
          <input v-model="form.backgroundColor" type="color" class="h-10 w-16 p-1 border border-gray-300 rounded" />
        </div>
        <div class="flex items-end">
          <span
            class="inline-flex items-center px-2.5 py-1 rounded text-xs font-medium"
            :style="{ color: form.textColor, backgroundColor: form.backgroundColor }"
          >
            {{ form.textDisplay || form.templateName || 'Preview' }}
          </span>
        </div>
      </div>

      <div class="space-y-2">
        <label class="flex items-center gap-2">
          <input v-model="form.isModOnly" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Mod-only flair</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.isActive" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Active</span>
        </label>
      </div>

      <CommonErrorDisplay v-if="saveError" :message="saveError.message" />

      <div class="flex gap-3">
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Changes' }}
        </button>
        <NuxtLink to="/admin/flairs" class="button white">
          Cancel
        </NuxtLink>
      </div>
    </form>
  </div>
</template>
