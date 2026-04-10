<script setup lang="ts">
import { useFlairs } from '~/composables/useFlairs'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const flairId = route.params.id as string
const toast = useToast()

useHead({ title: `Edit Flair - b/${boardName}` })

const { updateFlair } = useFlairs()
const boardId = ref<string | null>(null)
const isMod = ref(false)
const loadingFlair = ref(true)

const form = reactive({
  templateName: '',
  textDisplay: '',
  textColor: '#000000',
  backgroundColor: '#e0e0e0',
  isModOnly: false,
  isEditable: false,
  isActive: true,
  displayOrder: 0,
})
const saving = ref(false)

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

const FLAIR_QUERY = `
  query GetFlairTemplate($id: ID!) {
    flairTemplate(id: $id) {
      id boardId flairType templateName textDisplay textColor backgroundColor
      isModOnly isEditable isActive displayOrder
    }
  }
`

interface FlairData {
  id: string
  boardId: string
  flairType: string
  templateName: string
  textDisplay: string | null
  textColor: string
  backgroundColor: string
  isModOnly: boolean
  isEditable: boolean
  isActive: boolean
  displayOrder: number
}

const flair = ref<FlairData | null>(null)

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  } else {
    await navigateTo(`/b/${boardName}/flairs`)
    return
  }

  const { execute: execFlair } = useGraphQL<{ flairTemplate: FlairData }>()
  const flairResult = await execFlair(FLAIR_QUERY, { variables: { id: flairId } })
  if (flairResult?.flairTemplate) {
    flair.value = flairResult.flairTemplate
    form.templateName = flairResult.flairTemplate.templateName
    form.textDisplay = flairResult.flairTemplate.textDisplay ?? ''
    form.textColor = flairResult.flairTemplate.textColor
    form.backgroundColor = flairResult.flairTemplate.backgroundColor
    form.isModOnly = flairResult.flairTemplate.isModOnly
    form.isEditable = flairResult.flairTemplate.isEditable
    form.isActive = flairResult.flairTemplate.isActive
    form.displayOrder = flairResult.flairTemplate.displayOrder
  }

  loadingFlair.value = false
})

async function handleSubmit () {
  if (!form.templateName.trim()) return

  saving.value = true
  const result = await updateFlair(flairId, {
    templateName: form.templateName.trim(),
    textDisplay: form.textDisplay.trim() || undefined,
    textColor: form.textColor,
    backgroundColor: form.backgroundColor,
    isModOnly: form.isModOnly,
    isEditable: form.isEditable,
    isActive: form.isActive,
    displayOrder: form.displayOrder,
  })
  saving.value = false

  if (result) {
    toast.success('Flair updated')
    await navigateTo(`/b/${boardName}/flairs`)
  } else {
    toast.error('Failed to update flair')
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <nav class="text-sm text-gray-500 mb-4">
      <NuxtLink :to="`/b/${boardName}`" class="hover:text-gray-700">b/{{ boardName }}</NuxtLink>
      <span class="mx-1">/</span>
      <NuxtLink :to="`/b/${boardName}/flairs`" class="hover:text-gray-700">Flairs</NuxtLink>
      <span class="mx-1">/</span>
      <span>Edit</span>
    </nav>

    <h1 class="text-lg font-semibold text-gray-900 mb-6">Edit Flair</h1>

    <CommonLoadingSpinner v-if="loadingFlair" />

    <div v-else-if="!flair" class="text-center py-12">
      <p class="text-sm text-gray-500 mb-4">Flair not found.</p>
      <NuxtLink :to="`/b/${boardName}/flairs`" class="text-sm text-primary hover:underline">
        Back to flairs
      </NuxtLink>
    </div>

    <form v-else class="space-y-5 max-w-2xl" @submit.prevent="handleSubmit">
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Template Name</label>
          <input v-model="form.templateName" type="text" class="form-input w-full" required />
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Display Text</label>
          <input v-model="form.textDisplay" type="text" class="form-input w-full" placeholder="Optional" />
        </div>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Type</label>
        <input :value="flair.flairType" type="text" class="form-input w-full" disabled />
        <p class="text-xs text-gray-500 mt-1">Flair type cannot be changed after creation.</p>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
          <div class="flex items-center gap-2">
            <input v-model="form.textColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
            <input v-model="form.textColor" type="text" class="form-input w-full font-mono text-sm" />
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Background Color</label>
          <div class="flex items-center gap-2">
            <input v-model="form.backgroundColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
            <input v-model="form.backgroundColor" type="text" class="form-input w-full font-mono text-sm" />
          </div>
        </div>
      </div>

      <div class="space-y-2">
        <label class="flex items-center gap-2">
          <input v-model="form.isModOnly" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Mod-only</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.isEditable" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Editable by users</span>
        </label>
        <label class="flex items-center gap-2">
          <input v-model="form.isActive" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Active (visible to users)</span>
        </label>
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Display Order</label>
        <input v-model.number="form.displayOrder" type="number" class="form-input w-24" min="0" />
      </div>

      <div>
        <label class="block text-sm font-medium text-gray-700 mb-1">Preview</label>
        <span
          class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
          :style="{ color: form.textColor, backgroundColor: form.backgroundColor }"
        >
          {{ form.textDisplay || form.templateName || 'Preview' }}
        </span>
      </div>

      <div class="flex gap-3">
        <button type="submit" class="button primary" :disabled="saving || !form.templateName.trim()">
          {{ saving ? 'Saving...' : 'Save Changes' }}
        </button>
        <NuxtLink :to="`/b/${boardName}/flairs`" class="button white">Cancel</NuxtLink>
      </div>
    </form>
  </div>
</template>
