<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Emojis' })

interface EmojiObject {
  id: string
  shortcode: string
  imageUrl: string
  altText: string
  category: string
  scope: string
  isActive: boolean
  createdAt: string
}

interface EmojisResponse {
  getAllEmojisAdmin: EmojiObject[]
}

interface CreateEmojiResponse {
  createEmoji: EmojiObject
}

interface DeleteEmojiResponse {
  deleteEmoji: boolean
}

const { execute, loading, error, data } = useGraphQL<EmojisResponse>()
const { execute: executeCreate, loading: creating, error: createError } = useGraphQLMutation<CreateEmojiResponse>()
const { executeWithFile, uploading, error: uploadError } = useFileUpload()
const { execute: executeDelete, loading: deleting } = useGraphQLMutation<DeleteEmojiResponse>()

const shortcode = ref('')
const imageUrl = ref('')
const inputMode = ref<'upload' | 'url'>('upload')
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const filePreview = ref<string | null>(null)

const EMOJIS_QUERY = `
  query {
    getAllEmojisAdmin {
      id shortcode imageUrl altText category scope isActive createdAt
    }
  }
`

const CREATE_EMOJI = `
  mutation CreateEmoji($input: CreateEmojiInput!) {
    createEmoji(input: $input) {
      id shortcode imageUrl altText category scope isActive createdAt
    }
  }
`

const UPLOAD_EMOJI = `
  mutation UploadEmoji($shortcode: String!, $file: Upload!) {
    uploadEmoji(shortcode: $shortcode, file: $file) {
      id shortcode imageUrl altText category scope isActive createdAt
    }
  }
`

const DELETE_EMOJI = `
  mutation DeleteEmoji($emojiId: ID!) {
    deleteEmoji(emojiId: $emojiId)
  }
`

async function loadEmojis () {
  await execute(EMOJIS_QUERY)
}

function onFileSelected (event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return

  selectedFile.value = file
  // Generate preview
  const reader = new FileReader()
  reader.onload = (e) => {
    filePreview.value = e.target?.result as string
  }
  reader.readAsDataURL(file)
}

function clearFile () {
  selectedFile.value = null
  filePreview.value = null
  if (fileInput.value) {
    fileInput.value.value = ''
  }
}

async function createEmoji () {
  if (!shortcode.value.trim()) return

  if (inputMode.value === 'upload') {
    if (!selectedFile.value) return

    const result = await executeWithFile(
      UPLOAD_EMOJI,
      { shortcode: shortcode.value.trim() },
      'file',
      selectedFile.value,
    )

    if (result?.uploadEmoji) {
      shortcode.value = ''
      clearFile()
      await loadEmojis()
    }
  } else {
    if (!imageUrl.value.trim()) return

    const result = await executeCreate(CREATE_EMOJI, {
      variables: {
        input: {
          shortcode: shortcode.value.trim(),
          imageUrl: imageUrl.value.trim(),
          altText: shortcode.value.trim(),
          category: 'custom',
        },
      },
    })

    if (result?.createEmoji) {
      shortcode.value = ''
      imageUrl.value = ''
      await loadEmojis()
    }
  }
}

async function deleteEmoji (id: string) {
  await executeDelete(DELETE_EMOJI, { variables: { emojiId: id } })
  await loadEmojis()
}

onMounted(() => {
  loadEmojis()
})

const emojis = computed(() => data.value?.getAllEmojisAdmin ?? [])
const formValid = computed(() => {
  if (!shortcode.value.trim()) return false
  if (inputMode.value === 'upload') return !!selectedFile.value
  return !!imageUrl.value.trim()
})
const formBusy = computed(() => creating.value || uploading.value)

const formError = computed(() => createError.value || uploadError.value)
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold text-gray-900 mb-6">
      Custom Emojis
    </h2>

    <!-- Add emoji form -->
    <div class="bg-white rounded-lg border border-gray-200 p-4 mb-6 max-w-lg">
      <h3 class="text-sm font-medium text-gray-900 mb-3">
        Add New Emoji
      </h3>
      <form class="space-y-3" @submit.prevent="createEmoji">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Shortcode
          </label>
          <input
            v-model="shortcode"
            type="text"
            class="form-input w-full"
            placeholder="e.g. party_parrot"
            pattern="[a-z0-9_]+"
          />
          <p class="mt-1 text-xs text-gray-500">
            Lowercase letters, numbers, and underscores only.
          </p>
        </div>

        <!-- Input mode toggle -->
        <div class="flex gap-2">
          <button
            type="button"
            class="px-3 py-1 text-xs rounded-full border transition-colors"
            :class="inputMode === 'upload'
              ? 'bg-primary text-white border-primary'
              : 'bg-white text-gray-600 border-gray-300 hover:border-gray-400'"
            @click="inputMode = 'upload'"
          >
            Upload File
          </button>
          <button
            type="button"
            class="px-3 py-1 text-xs rounded-full border transition-colors"
            :class="inputMode === 'url'
              ? 'bg-primary text-white border-primary'
              : 'bg-white text-gray-600 border-gray-300 hover:border-gray-400'"
            @click="inputMode = 'url'"
          >
            Image URL
          </button>
        </div>

        <!-- File upload -->
        <div v-if="inputMode === 'upload'">
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Emoji Image
          </label>
          <div class="flex items-center gap-3">
            <label
              class="cursor-pointer inline-flex items-center gap-2 px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
            >
              <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              Choose file
              <input
                ref="fileInput"
                type="file"
                accept="image/png,image/gif,image/webp,image/jpeg"
                class="hidden"
                @change="onFileSelected"
              />
            </label>
            <div v-if="selectedFile" class="flex items-center gap-2">
              <img
                v-if="filePreview"
                :src="filePreview"
                alt="Preview"
                class="w-8 h-8 object-contain rounded border border-gray-200"
              />
              <span class="text-sm text-gray-600 truncate max-w-[150px]">{{ selectedFile.name }}</span>
              <button
                type="button"
                class="text-gray-400 hover:text-red-500 transition-colors"
                @click="clearFile"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          </div>
          <p class="mt-1 text-xs text-gray-500">
            PNG, GIF, WebP, or JPEG. Max 512x512 pixels.
          </p>
        </div>

        <!-- URL input -->
        <div v-else>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            Image URL
          </label>
          <input
            v-model="imageUrl"
            type="url"
            class="form-input w-full"
            placeholder="https://example.com/emoji.png"
          />
          <p class="mt-1 text-xs text-gray-500">
            URL to the emoji image (PNG, GIF, or WebP).
          </p>
        </div>

        <CommonErrorDisplay v-if="formError" :message="formError.message" />

        <button
          type="submit"
          class="button primary"
          :disabled="formBusy || !formValid"
        >
          {{ formBusy ? 'Adding...' : 'Add Emoji' }}
        </button>
      </form>
    </div>

    <!-- Emoji list -->
    <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-3">
      Current Emojis
    </h3>

    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" />

    <div v-else-if="emojis.length === 0" class="text-sm text-gray-500">
      No custom emojis uploaded yet.
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      <div
        v-for="emoji in emojis"
        :key="emoji.id"
        class="bg-white rounded-lg border border-gray-200 p-3 flex items-center gap-3"
      >
        <img
          :src="emoji.imageUrl"
          :alt="emoji.shortcode"
          class="w-8 h-8 object-contain"
        />
        <span class="text-sm font-mono text-gray-700 flex-1 truncate">
          :{{ emoji.shortcode }}:
        </span>
        <button
          class="button button-sm red shrink-0"
          :disabled="deleting"
          @click="deleteEmoji(emoji.id)"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
</template>
