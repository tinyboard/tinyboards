<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import { useToast } from '~/composables/useToast'
import { useAuth } from '~/composables/useAuth'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()
const { isAdmin } = useAuth()

useHead({ title: `Appearance - b/${boardName}` })

interface BoardData {
  id: string
  icon: string | null
  banner: string | null
  primaryColor: string
  secondaryColor: string
  hoverColor: string
}

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id icon banner primaryColor secondaryColor hoverColor }
  }
`

const UPDATE_BOARD_SETTINGS = `
  mutation UpdateBoardSettings($input: UpdateBoardSettingsInput!) {
    updateBoardSettings(input: $input) {
      board { id icon banner primaryColor secondaryColor hoverColor }
    }
  }
`

const { execute, loading, error } = useGraphQL<{ board: BoardData }>()
const { execute: executeMutation, loading: saving } = useGraphQLMutation()
const { uploadFile: doUploadFile } = useFileUpload()
const boardId = ref<string | null>(null)

const form = reactive({
  primaryColor: '#3b82f6',
  secondaryColor: '#1e40af',
  hoverColor: '#2563eb',
  icon: null as string | null,
  banner: null as string | null,
})

const iconPreview = ref<string | null>(null)
const bannerPreview = ref<string | null>(null)
const pendingIconFile = ref<File | null>(null)
const pendingBannerFile = ref<File | null>(null)
const uploading = ref(false)

onMounted(async () => {
  const result = await execute(BOARD_QUERY, { variables: { name: boardName } })
  if (!result?.board) return

  const board = result.board
  boardId.value = board.id
  form.primaryColor = board.primaryColor
  form.secondaryColor = board.secondaryColor
  form.hoverColor = board.hoverColor
  form.icon = board.icon
  form.banner = board.banner
  iconPreview.value = board.icon
  bannerPreview.value = board.banner
})

function handleIconSelect (event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  pendingIconFile.value = file
  const reader = new FileReader()
  reader.onload = () => {
    iconPreview.value = reader.result as string
  }
  reader.readAsDataURL(file)
}

function handleBannerSelect (event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  pendingBannerFile.value = file
  const reader = new FileReader()
  reader.onload = () => {
    bannerPreview.value = reader.result as string
  }
  reader.readAsDataURL(file)
}

function removeIcon () {
  form.icon = null
  iconPreview.value = null
  pendingIconFile.value = null
}

function removeBanner () {
  form.banner = null
  bannerPreview.value = null
  pendingBannerFile.value = null
}

async function saveSettings () {
  if (!boardId.value) return

  uploading.value = true

  // Upload pending files first
  if (pendingIconFile.value) {
    const url = await doUploadFile(pendingIconFile.value)
    if (url) {
      form.icon = url
      pendingIconFile.value = null
    }
  }

  if (pendingBannerFile.value) {
    const url = await doUploadFile(pendingBannerFile.value)
    if (url) {
      form.banner = url
      pendingBannerFile.value = null
    }
  }

  uploading.value = false

  const result = await executeMutation(UPDATE_BOARD_SETTINGS, {
    variables: {
      input: {
        boardId: boardId.value,
        primaryColor: form.primaryColor,
        secondaryColor: form.secondaryColor,
        hoverColor: form.hoverColor,
        icon: form.icon,
        banner: form.banner,
      },
    },
  })

  if (result) {
    toast.success('Appearance settings saved')
  }
}
</script>

<template>
  <div>
    <!-- Settings sub-navigation -->
    <div class="flex gap-1 border-b border-gray-200 mb-4">
      <NuxtLink
        :to="`/b/${boardName}/settings`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        General
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/appearance`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-blue-600 text-blue-600"
      >
        Appearance
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/moderation`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Moderation
      </NuxtLink>
    </div>

    <h2 class="text-base font-semibold text-gray-900 mb-4">
      Board Appearance
    </h2>

    <CommonLoadingSpinner v-if="loading" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <form v-else class="space-y-6 max-w-2xl" @submit.prevent="saveSettings">
      <!-- Board icon -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Board Icon</label>
        <div class="flex items-center gap-4">
          <div class="h-16 w-16 rounded-full bg-gray-100 flex items-center justify-center overflow-hidden border">
            <img v-if="iconPreview" :src="iconPreview" alt="Board icon" class="h-full w-full object-cover" />
            <span v-else class="text-gray-400 text-xs">No icon</span>
          </div>
          <div class="flex gap-2">
            <label class="button white button-sm cursor-pointer">
              Upload icon
              <input type="file" accept="image/*" class="hidden" @change="handleIconSelect" />
            </label>
            <button v-if="iconPreview" type="button" class="button white button-sm text-red-600" @click="removeIcon">
              Remove
            </button>
          </div>
        </div>
      </div>

      <!-- Board banner -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Board Banner</label>
        <div
          class="w-full h-32 rounded-lg bg-gray-100 flex items-center justify-center overflow-hidden border"
          :style="bannerPreview ? { backgroundImage: `url(${bannerPreview})`, backgroundSize: 'cover', backgroundPosition: 'center' } : {}"
        >
          <span v-if="!bannerPreview" class="text-gray-400 text-xs">No banner (recommended 3:1 aspect ratio)</span>
        </div>
        <div class="flex gap-2 mt-2">
          <label class="button white button-sm cursor-pointer">
            Upload banner
            <input type="file" accept="image/*" class="hidden" @change="handleBannerSelect" />
          </label>
          <button v-if="bannerPreview" type="button" class="button white button-sm text-red-600" @click="removeBanner">
            Remove
          </button>
        </div>
      </div>

      <!-- Colors -->
      <div>
        <h3 class="text-sm font-medium text-gray-700 mb-3">Board Colors</h3>
        <div class="grid grid-cols-3 gap-4">
          <div>
            <label class="block text-xs text-gray-600 mb-1">Primary</label>
            <div class="flex items-center gap-2">
              <input v-model="form.primaryColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
              <input v-model="form.primaryColor" type="text" class="form-input w-full font-mono text-sm" />
            </div>
          </div>
          <div>
            <label class="block text-xs text-gray-600 mb-1">Secondary</label>
            <div class="flex items-center gap-2">
              <input v-model="form.secondaryColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
              <input v-model="form.secondaryColor" type="text" class="form-input w-full font-mono text-sm" />
            </div>
          </div>
          <div>
            <label class="block text-xs text-gray-600 mb-1">Hover</label>
            <div class="flex items-center gap-2">
              <input v-model="form.hoverColor" type="color" class="h-8 w-8 cursor-pointer rounded border" />
              <input v-model="form.hoverColor" type="text" class="form-input w-full font-mono text-sm" />
            </div>
          </div>
        </div>
      </div>

      <!-- Color preview -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">Preview</label>
        <div class="rounded-lg border p-4">
          <div class="flex gap-2">
            <span
              class="inline-flex items-center rounded px-3 py-1.5 text-sm font-medium text-white"
              :style="{ backgroundColor: form.primaryColor }"
            >
              Primary
            </span>
            <span
              class="inline-flex items-center rounded px-3 py-1.5 text-sm font-medium text-white"
              :style="{ backgroundColor: form.secondaryColor }"
            >
              Secondary
            </span>
            <span
              class="inline-flex items-center rounded px-3 py-1.5 text-sm font-medium text-white"
              :style="{ backgroundColor: form.hoverColor }"
            >
              Hover
            </span>
          </div>
        </div>
      </div>

      <div>
        <button type="submit" class="button primary" :disabled="saving">
          {{ saving ? 'Saving...' : 'Save Appearance' }}
        </button>
      </div>
    </form>
  </div>
</template>
