<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import { useToast } from '~/composables/useToast'
import { useAuth } from '~/composables/useAuth'
import { validateCss, CSS_SNIPPET_CATEGORIES } from '~/utils/css-validator'
import type { CssSnippet } from '~/utils/css-validator'

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
  customCss: string | null
}

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id icon banner primaryColor secondaryColor hoverColor customCss }
  }
`

const UPDATE_BOARD_SETTINGS = `
  mutation UpdateBoardSettings($input: UpdateBoardSettingsInput!) {
    updateBoardSettings(input: $input) {
      board { id icon banner primaryColor secondaryColor hoverColor customCss }
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

const cssCode = ref('')
const showCssEditor = ref(false)
const showCssWizard = ref(false)
const expandedCategory = ref<string | null>(null)

const cssValidation = computed(() => validateCss(cssCode.value, 25 * 1024))
const cssCharCount = computed(() => new Blob([cssCode.value]).size)

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
  cssCode.value = board.customCss ?? ''
  if (board.customCss) {
    showCssEditor.value = true
  }
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

function insertSnippet (snippet: CssSnippet) {
  if (cssCode.value && !cssCode.value.endsWith('\n')) {
    cssCode.value += '\n'
  }
  cssCode.value += `\n/* ${snippet.name} */\n${snippet.css}\n`
  showCssWizard.value = false
  showCssEditor.value = true
  toast.success(`Inserted: ${snippet.name}`)
}

function toggleCategory (name: string) {
  expandedCategory.value = expandedCategory.value === name ? null : name
}

async function saveSettings () {
  if (!boardId.value) return

  if (cssCode.value && !cssValidation.value.valid) {
    toast.error('Please fix CSS errors before saving')
    return
  }

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
        customCss: cssCode.value || '',
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

      <!-- Custom CSS section -->
      <div class="border-t border-gray-200 pt-6">
        <div class="flex items-center justify-between mb-3">
          <div>
            <h3 class="text-sm font-medium text-gray-900">Custom CSS</h3>
            <p class="text-xs text-gray-500 mt-0.5">Add custom styles that apply only to this board. Overrides site-level CSS.</p>
          </div>
          <div class="flex gap-2">
            <button
              v-if="!showCssWizard"
              type="button"
              class="button white button-sm"
              @click="showCssWizard = true; showCssEditor = true"
            >
              Style Wizard
            </button>
            <button
              type="button"
              class="button white button-sm"
              @click="showCssEditor = !showCssEditor"
            >
              {{ showCssEditor ? 'Hide Editor' : 'Show Editor' }}
            </button>
          </div>
        </div>

        <!-- CSS Wizard (compact version) -->
        <div v-if="showCssWizard" class="mb-4 border border-gray-200 rounded-lg overflow-hidden">
          <div class="px-3 py-2 bg-gray-50 border-b border-gray-200 flex items-center justify-between">
            <span class="text-xs font-medium text-gray-700">Quick Snippets</span>
            <button type="button" class="text-xs text-gray-400 hover:text-gray-600" @click="showCssWizard = false">Close</button>
          </div>
          <div class="max-h-64 overflow-y-auto divide-y divide-gray-100">
            <div v-for="category in CSS_SNIPPET_CATEGORIES" :key="category.name">
              <button
                type="button"
                class="w-full flex items-center gap-2 px-3 py-2 text-left hover:bg-gray-50 text-xs"
                @click="toggleCategory(category.name)"
              >
                <span class="font-medium text-gray-700 flex-1">{{ category.name }}</span>
                <span class="text-gray-400" :class="expandedCategory === category.name ? 'rotate-90' : ''">&#9656;</span>
              </button>
              <div v-if="expandedCategory === category.name" class="bg-gray-50 divide-y divide-gray-100">
                <div v-for="snippet in category.snippets" :key="snippet.name" class="px-3 py-2 flex items-center justify-between gap-2">
                  <div class="min-w-0">
                    <div class="text-xs font-medium text-gray-800 truncate">{{ snippet.name }}</div>
                    <div class="text-xs text-gray-500 truncate">{{ snippet.description }}</div>
                  </div>
                  <button type="button" class="shrink-0 text-xs text-primary hover:underline" @click="insertSnippet(snippet)">Insert</button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- CSS Editor -->
        <div v-if="showCssEditor" class="space-y-3">
          <textarea
            v-model="cssCode"
            class="w-full h-48 font-mono text-xs bg-gray-900 text-green-400 rounded-lg p-3 border border-gray-700 focus:border-primary focus:ring-1 focus:ring-primary resize-y leading-relaxed"
            placeholder="/* Board-specific CSS — overrides site CSS */
.post-card { border-radius: 12px; }"
            spellcheck="false"
          />
          <div class="flex items-center justify-between text-xs">
            <div>
              <span v-if="cssCode && cssValidation.valid" class="text-green-600">&#10003; Valid CSS</span>
              <span v-else-if="cssCode && !cssValidation.valid" class="text-red-600">
                &#10007; {{ cssValidation.errors[0] }}
              </span>
            </div>
            <span class="text-gray-400">{{ (cssCharCount / 1024).toFixed(1) }} KB / 25 KB</span>
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
