<script setup lang="ts">
import { useFlairs } from '~/composables/useFlairs'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Flairs - b/${boardName}` })

const { flairs, loading, fetchFlairs, createFlair, deleteFlair } = useFlairs()
const boardId = ref<string | null>(null)
const isMod = ref(false)

const showCreateForm = ref(false)
const newFlair = reactive({
  templateName: '',
  textDisplay: '',
  textColor: '#000000',
  backgroundColor: '#e0e0e0',
  flairType: 'Post' as 'Post' | 'User',
  isModOnly: false,
  isEditable: false,
})
const creating = ref(false)
const confirmDeleteId = ref<string | null>(null)
const filterType = ref<'all' | 'Post' | 'User'>('all')

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
    await navigateTo(`/b/${boardName}`)
    return
  }

  await fetchFlairs(boardId.value)
})

const filteredFlairs = computed(() => {
  if (filterType.value === 'all') return flairs.value
  return flairs.value.filter(f => f.flairType === filterType.value)
})

async function handleCreate () {
  if (!boardId.value || !newFlair.templateName.trim()) return

  creating.value = true
  const result = await createFlair({
    boardId: boardId.value,
    flairType: newFlair.flairType,
    templateName: newFlair.templateName.trim(),
    textDisplay: newFlair.textDisplay.trim() || undefined,
    textColor: newFlair.textColor,
    backgroundColor: newFlair.backgroundColor,
    isModOnly: newFlair.isModOnly,
    isEditable: newFlair.isEditable,
  })
  creating.value = false

  if (result) {
    toast.success('Flair created')
    newFlair.templateName = ''
    newFlair.textDisplay = ''
    newFlair.textColor = '#000000'
    newFlair.backgroundColor = '#e0e0e0'
    newFlair.flairType = 'Post'
    newFlair.isModOnly = false
    newFlair.isEditable = false
    showCreateForm.value = false
  } else {
    toast.error('Failed to create flair')
  }
}

async function handleDelete (templateId: string) {
  await deleteFlair(templateId)
  confirmDeleteId.value = null
  toast.success('Flair deleted')
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <div class="flex items-center justify-between mb-6">
      <div>
        <nav class="text-sm text-gray-500 mb-1">
          <NuxtLink :to="`/b/${boardName}`" class="hover:text-gray-700">b/{{ boardName }}</NuxtLink>
          <span class="mx-1">/</span>
          <span>Flairs</span>
        </nav>
        <h1 class="text-lg font-semibold text-gray-900">Flair Management</h1>
      </div>
      <button
        v-if="!showCreateForm"
        class="button primary button-sm"
        @click="showCreateForm = true"
      >
        Create flair
      </button>
    </div>

    <!-- Create form -->
    <div v-if="showCreateForm" class="bg-white rounded-lg border border-gray-200 p-4 mb-6">
      <h3 class="text-sm font-medium text-gray-900 mb-4">New Flair</h3>
      <form class="space-y-4" @submit.prevent="handleCreate">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Template Name</label>
            <input v-model="newFlair.templateName" type="text" class="form-input w-full" placeholder="e.g. discussion" required />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Display Text</label>
            <input v-model="newFlair.textDisplay" type="text" class="form-input w-full" placeholder="e.g. Discussion (optional)" />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Flair Type</label>
          <select v-model="newFlair.flairType" class="form-input w-full">
            <option value="Post">Post Flair</option>
            <option value="User">User Flair</option>
          </select>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Text Color</label>
            <div class="flex items-center gap-2">
              <input v-model="newFlair.textColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
              <input v-model="newFlair.textColor" type="text" class="form-input w-full font-mono text-sm" placeholder="#000000" />
            </div>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Background Color</label>
            <div class="flex items-center gap-2">
              <input v-model="newFlair.backgroundColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
              <input v-model="newFlair.backgroundColor" type="text" class="form-input w-full font-mono text-sm" placeholder="#e0e0e0" />
            </div>
          </div>
        </div>

        <div class="space-y-2">
          <label class="flex items-center gap-2">
            <input v-model="newFlair.isModOnly" type="checkbox" class="form-checkbox" />
            <span class="text-sm text-gray-700">Mod-only (only moderators can assign)</span>
          </label>
          <label class="flex items-center gap-2">
            <input v-model="newFlair.isEditable" type="checkbox" class="form-checkbox" />
            <span class="text-sm text-gray-700">Editable (users can customize text)</span>
          </label>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">Preview</label>
          <span
            class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
            :style="{ color: newFlair.textColor, backgroundColor: newFlair.backgroundColor }"
          >
            {{ newFlair.textDisplay || newFlair.templateName || 'Preview' }}
          </span>
        </div>

        <div class="flex gap-3">
          <button type="submit" class="button primary button-sm" :disabled="creating">
            {{ creating ? 'Creating...' : 'Create' }}
          </button>
          <button type="button" class="button white button-sm" @click="showCreateForm = false">Cancel</button>
        </div>
      </form>
    </div>

    <!-- Type filter -->
    <div class="flex gap-2 mb-4">
      <button
        v-for="ft in [{ label: 'All', value: 'all' }, { label: 'Post', value: 'Post' }, { label: 'User', value: 'User' }]"
        :key="ft.value"
        class="button button-sm"
        :class="filterType === ft.value ? 'primary' : 'white'"
        @click="filterType = ft.value as typeof filterType"
      >
        {{ ft.label }}
      </button>
    </div>

    <CommonLoadingSpinner v-if="loading" />

    <div v-else-if="filteredFlairs.length === 0" class="text-center py-12">
      <p class="text-sm text-gray-500">No flairs configured for this board.</p>
    </div>

    <div v-else class="space-y-2">
      <div
        v-for="flair in filteredFlairs"
        :key="flair.id"
        class="bg-white rounded-lg border border-gray-200 p-4 flex items-center justify-between"
      >
        <div class="flex items-center gap-3">
          <span
            class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium"
            :style="{ color: flair.textColor, backgroundColor: flair.backgroundColor }"
          >
            {{ flair.textDisplay || flair.templateName }}
          </span>
          <span class="text-xs text-gray-500">{{ flair.templateName }}</span>
          <span class="text-xs text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded">{{ flair.flairType }}</span>
        </div>
        <div class="flex items-center gap-2">
          <NuxtLink
            :to="`/b/${boardName}/flairs/${flair.id}/edit`"
            class="button white button-sm"
          >
            Edit
          </NuxtLink>
          <button
            v-if="confirmDeleteId === flair.id"
            class="button button-sm text-red-600 hover:bg-red-50"
            @click="handleDelete(flair.id)"
          >
            Confirm delete
          </button>
          <button
            v-else
            class="button white button-sm text-red-600"
            @click="confirmDeleteId = flair.id"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
