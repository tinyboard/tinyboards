<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useFileUpload } from '~/composables/useFileUpload'
import { useToast } from '~/composables/useToast'
import { unicodeEmojis } from '~/data/emojis'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const boardName = route.params.board as string
const toast = useToast()

useHead({ title: `Emojis - b/${boardName}` })

// ============================================================
// Types
// ============================================================

interface ReactionEmojiEntry {
  type: 'unicode' | 'custom'
  value?: string
  shortcode?: string
  imageUrl?: string
}

interface EmojiObject {
  id: string
  shortcode: string
  imageUrl: string
  altText: string
  category: string
  scope: string
  boardId: string | null
  isActive: boolean
  createdAt: string
}

// ============================================================
// GraphQL queries/mutations
// ============================================================

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const REACTION_SETTINGS_QUERY = `
  query GetBoardReactionSettings($boardId: ID!) {
    getBoardReactionSettings(boardId: $boardId) {
      id boardId reactionEmojis reactionsEnabled
    }
  }
`

const UPDATE_REACTION_SETTINGS = `
  mutation UpdateBoardReactionSettings($input: UpdateBoardReactionSettingsInput!) {
    updateBoardReactionSettings(input: $input) {
      settings { id reactionEmojis reactionsEnabled }
    }
  }
`

const LIST_BOARD_EMOJIS = `
  query ListBoardEmojis($input: ListEmojisInput) {
    listEmojis(input: $input) {
      id shortcode imageUrl altText category scope boardId isActive createdAt
    }
  }
`

const LIST_ALL_EMOJIS = `
  query ListAllEmojis($input: ListEmojisInput) {
    listEmojis(input: $input) {
      id shortcode imageUrl category scope boardId
    }
  }
`

const CREATE_EMOJI = `
  mutation CreateEmoji($input: CreateEmojiInput!) {
    createEmoji(input: $input) {
      id shortcode imageUrl altText category scope boardId isActive createdAt
    }
  }
`

const UPLOAD_EMOJI = `
  mutation UploadEmoji($shortcode: String!, $file: Upload!, $category: String, $boardId: ID, $scope: EmojiScope) {
    uploadEmoji(shortcode: $shortcode, file: $file, category: $category, boardId: $boardId, scope: $scope) {
      id shortcode imageUrl altText category scope boardId isActive createdAt
    }
  }
`

const DELETE_EMOJI = `
  mutation DeleteEmoji($emojiId: ID!) {
    deleteEmoji(emojiId: $emojiId)
  }
`

// ============================================================
// State
// ============================================================

const { execute, loading } = useGraphQL()
const { execute: executeMutation, loading: saving } = useGraphQLMutation()
const { executeWithFile, uploading } = useFileUpload()
const { execute: executeDelete, loading: deleting } = useGraphQLMutation()

const boardId = ref<string | null>(null)

// --- Reaction emoji config ---
const reactionEmojis = ref<ReactionEmojiEntry[]>([])
const reactionsEnabled = ref(true)
const showEmojiSelector = ref(false)
const emojiSearch = ref('')

// Available custom emojis (site-wide + board-specific)
const availableCustomEmojis = ref<EmojiObject[]>([])

// --- Board emoji management ---
const boardEmojis = ref<EmojiObject[]>([])
const newShortcode = ref('')
const newImageUrl = ref('')
const inputMode = ref<'upload' | 'url'>('upload')
const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const filePreview = ref<string | null>(null)
const createError = ref<string | null>(null)

// ============================================================
// Lifecycle
// ============================================================

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return
  boardId.value = boardResult.board.id
  await Promise.all([loadReactionSettings(), loadBoardEmojis(), loadAvailableEmojis()])
})

// ============================================================
// Reaction settings
// ============================================================

async function loadReactionSettings () {
  if (!boardId.value) return
  const { execute: exec } = useGraphQL<{
    getBoardReactionSettings: {
      reactionEmojis: ReactionEmojiEntry[]
      reactionsEnabled: boolean
    } | null
  }>()
  const result = await exec(REACTION_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (result?.getBoardReactionSettings) {
    const s = result.getBoardReactionSettings
    reactionsEnabled.value = s.reactionsEnabled
    if (Array.isArray(s.reactionEmojis) && s.reactionEmojis.length > 0) {
      reactionEmojis.value = s.reactionEmojis
    }
  }
}

async function saveReactionSettings () {
  if (!boardId.value) return
  const result = await executeMutation(UPDATE_REACTION_SETTINGS, {
    variables: {
      input: {
        boardId: boardId.value,
        isReactionsEnabled: reactionsEnabled.value,
        reactionEmojis: reactionEmojis.value,
      },
    },
  })
  if (result) {
    toast.success('Reaction settings saved')
  }
}

function removeReactionEmoji (index: number) {
  reactionEmojis.value.splice(index, 1)
}

function addUnicodeReaction (emoji: string) {
  if (reactionEmojis.value.length >= 10) {
    toast.error('Maximum 10 reaction emojis')
    return
  }
  // Check for duplicate
  if (reactionEmojis.value.some(e => e.type === 'unicode' && e.value === emoji)) return
  reactionEmojis.value.push({ type: 'unicode', value: emoji })
  showEmojiSelector.value = false
  emojiSearch.value = ''
}

function addCustomReaction (emoji: EmojiObject) {
  if (reactionEmojis.value.length >= 10) {
    toast.error('Maximum 10 reaction emojis')
    return
  }
  if (reactionEmojis.value.some(e => e.type === 'custom' && e.shortcode === emoji.shortcode)) return
  reactionEmojis.value.push({
    type: 'custom',
    shortcode: emoji.shortcode,
    imageUrl: emoji.imageUrl,
  })
  showEmojiSelector.value = false
  emojiSearch.value = ''
}

function resetToDefaults () {
  reactionEmojis.value = []
}

const filteredUnicodeEmojis = computed(() => {
  if (!emojiSearch.value) return unicodeEmojis.slice(0, 50)
  const q = emojiSearch.value.toLowerCase()
  return unicodeEmojis.filter(e =>
    e.shortcode.includes(q) || e.keywords.some(k => k.includes(q)),
  ).slice(0, 50)
})

const filteredCustomEmojis = computed(() => {
  if (!emojiSearch.value) return availableCustomEmojis.value
  const q = emojiSearch.value.toLowerCase()
  return availableCustomEmojis.value.filter(e => e.shortcode.includes(q))
})

// ============================================================
// Board emoji management
// ============================================================

async function loadBoardEmojis () {
  if (!boardId.value) return
  const { execute: exec } = useGraphQL<{ listEmojis: EmojiObject[] }>()
  const result = await exec(LIST_BOARD_EMOJIS, {
    variables: { input: { boardId: boardId.value, scope: 'Board' } },
  })
  if (result?.listEmojis) {
    boardEmojis.value = result.listEmojis
  }
}

async function loadAvailableEmojis () {
  if (!boardId.value) return
  const { execute: exec } = useGraphQL<{ listEmojis: EmojiObject[] }>()
  const result = await exec(LIST_ALL_EMOJIS, {
    variables: { input: { boardId: boardId.value, limit: 200 } },
  })
  if (result?.listEmojis) {
    availableCustomEmojis.value = result.listEmojis
  }
}

function onFileSelected (event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  selectedFile.value = file
  const reader = new FileReader()
  reader.onload = (e) => { filePreview.value = e.target?.result as string }
  reader.readAsDataURL(file)
}

function clearFile () {
  selectedFile.value = null
  filePreview.value = null
  if (fileInput.value) fileInput.value.value = ''
}

async function createBoardEmoji () {
  if (!boardId.value || !newShortcode.value.trim()) return
  createError.value = null

  if (inputMode.value === 'upload') {
    if (!selectedFile.value) return
    const result = await executeWithFile(
      UPLOAD_EMOJI,
      {
        shortcode: newShortcode.value.trim(),
        boardId: boardId.value,
        scope: 'Board',
        category: 'custom',
      },
      'file',
      selectedFile.value,
    )
    if (result?.uploadEmoji) {
      newShortcode.value = ''
      clearFile()
      await Promise.all([loadBoardEmojis(), loadAvailableEmojis()])
      toast.success('Board emoji created')
    }
  } else {
    if (!newImageUrl.value.trim()) return
    const { execute: exec } = useGraphQLMutation<{ createEmoji: EmojiObject }>()
    const result = await exec(CREATE_EMOJI, {
      variables: {
        input: {
          shortcode: newShortcode.value.trim(),
          imageUrl: newImageUrl.value.trim(),
          altText: newShortcode.value.trim(),
          category: 'custom',
          boardId: boardId.value,
          scope: 'Board',
        },
      },
    })
    if (result?.createEmoji) {
      newShortcode.value = ''
      newImageUrl.value = ''
      await Promise.all([loadBoardEmojis(), loadAvailableEmojis()])
      toast.success('Board emoji created')
    }
  }
}

async function deleteBoardEmoji (id: string) {
  await executeDelete(DELETE_EMOJI, { variables: { emojiId: id } })
  await Promise.all([loadBoardEmojis(), loadAvailableEmojis()])
  toast.success('Emoji deleted')
}

const formValid = computed(() => {
  if (!newShortcode.value.trim()) return false
  if (inputMode.value === 'upload') return !!selectedFile.value
  return !!newImageUrl.value.trim()
})
const formBusy = computed(() => uploading.value)
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
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Appearance
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/moderation`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-transparent text-gray-500 hover:text-gray-700"
      >
        Moderation
      </NuxtLink>
      <NuxtLink
        :to="`/b/${boardName}/settings/emojis`"
        class="px-3 py-1.5 text-sm font-medium border-b-2 no-underline transition-colors border-blue-600 text-blue-600"
      >
        Emojis
      </NuxtLink>
    </div>

    <CommonLoadingSpinner v-if="loading && !boardId" size="lg" />

    <div v-else class="space-y-8 max-w-3xl">
      <!-- ============================================================ -->
      <!-- Section A: Reaction Emojis Configuration -->
      <!-- ============================================================ -->
      <section>
        <h2 class="text-base font-semibold text-gray-900 mb-1">
          Reaction Emojis
        </h2>
        <p class="text-xs text-gray-500 mb-4">
          Choose which emojis appear as quick-reaction buttons on posts and comments. Leave empty to use the site defaults.
        </p>

        <!-- Reactions enabled toggle -->
        <label class="flex items-center gap-2 mb-4">
          <input v-model="reactionsEnabled" type="checkbox" class="form-checkbox" />
          <span class="text-sm text-gray-700">Enable reactions on this board</span>
        </label>

        <!-- Current reaction emojis -->
        <div class="bg-white border border-gray-200 rounded-lg p-4 mb-4">
          <div class="flex items-center gap-2 flex-wrap min-h-[2.5rem]">
            <div
              v-for="(entry, idx) in reactionEmojis"
              :key="idx"
              class="group relative inline-flex items-center gap-1 px-2.5 py-1 rounded-full border border-gray-200 bg-gray-50 text-sm"
            >
              <img
                v-if="entry.type === 'custom' && entry.imageUrl"
                :src="entry.imageUrl"
                :alt="entry.shortcode"
                class="w-5 h-5 object-contain"
              />
              <span v-else>{{ entry.value }}</span>
              <span v-if="entry.type === 'custom'" class="text-xs text-gray-400">
                :{{ entry.shortcode }}:
              </span>
              <button
                class="ml-0.5 text-gray-300 hover:text-red-500 transition-colors"
                title="Remove"
                @click="removeReactionEmoji(idx)"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <span v-if="reactionEmojis.length === 0" class="text-xs text-gray-400 italic">
              Using site defaults (👍 ❤️ 😂 😮 😢 🔥)
            </span>

            <!-- Add button -->
            <button
              v-if="reactionEmojis.length < 10"
              class="inline-flex items-center px-2 py-1 rounded-full text-xs border border-dashed border-gray-300 text-gray-400 hover:text-gray-600 hover:border-gray-400 transition-colors"
              @click="showEmojiSelector = !showEmojiSelector"
            >
              + Add
            </button>
          </div>
        </div>

        <!-- Emoji selector popup -->
        <div v-if="showEmojiSelector" class="bg-white border border-gray-200 rounded-lg shadow-lg p-4 mb-4">
          <div class="flex items-center justify-between mb-3">
            <span class="text-sm font-medium text-gray-700">Select an Emoji</span>
            <button class="text-xs text-gray-400 hover:text-gray-600" @click="showEmojiSelector = false">Close</button>
          </div>
          <input
            v-model="emojiSearch"
            type="text"
            class="form-input w-full mb-3 text-sm"
            placeholder="Search emojis..."
          />

          <!-- Custom emojis section -->
          <div v-if="filteredCustomEmojis.length > 0" class="mb-3">
            <h4 class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">Custom Emojis</h4>
            <div class="flex flex-wrap gap-1.5 max-h-32 overflow-y-auto">
              <button
                v-for="emoji in filteredCustomEmojis"
                :key="emoji.id"
                class="w-9 h-9 flex items-center justify-center rounded hover:bg-gray-100 border border-transparent hover:border-gray-200 transition-colors"
                :title="`:${emoji.shortcode}:`"
                @click="addCustomReaction(emoji)"
              >
                <img :src="emoji.imageUrl" :alt="emoji.shortcode" class="w-6 h-6 object-contain" />
              </button>
            </div>
          </div>

          <!-- Unicode emojis section -->
          <div>
            <h4 class="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">Unicode Emojis</h4>
            <div class="flex flex-wrap gap-1 max-h-40 overflow-y-auto">
              <button
                v-for="ue in filteredUnicodeEmojis"
                :key="ue.shortcode"
                class="w-9 h-9 flex items-center justify-center rounded hover:bg-gray-100 text-lg transition-colors"
                :title="`:${ue.shortcode}:`"
                @click="addUnicodeReaction(ue.emoji)"
              >
                {{ ue.emoji }}
              </button>
            </div>
          </div>
        </div>

        <!-- Action buttons -->
        <div class="flex gap-2">
          <button
            class="button primary"
            :disabled="saving"
            @click="saveReactionSettings"
          >
            {{ saving ? 'Saving...' : 'Save Reaction Settings' }}
          </button>
          <button
            v-if="reactionEmojis.length > 0"
            class="button white"
            @click="resetToDefaults"
          >
            Reset to Defaults
          </button>
        </div>
      </section>

      <!-- ============================================================ -->
      <!-- Section B: Board Custom Emojis -->
      <!-- ============================================================ -->
      <section>
        <h2 class="text-base font-semibold text-gray-900 mb-1">
          Board Emojis
        </h2>
        <p class="text-xs text-gray-500 mb-4">
          Custom emojis uploaded here are only available within this board. Site-wide emojis are managed by site administrators and are available everywhere.
        </p>

        <!-- Add emoji form -->
        <div class="bg-white rounded-lg border border-gray-200 p-4 mb-6 max-w-lg">
          <h3 class="text-sm font-medium text-gray-900 mb-3">
            Add Board Emoji
          </h3>
          <form class="space-y-3" @submit.prevent="createBoardEmoji">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-1">Shortcode</label>
              <input
                v-model="newShortcode"
                type="text"
                class="form-input w-full"
                placeholder="e.g. board_mascot"
                pattern="[a-z0-9_]+"
              />
              <p class="mt-1 text-xs text-gray-500">Lowercase letters, numbers, and underscores only.</p>
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
              <label class="block text-sm font-medium text-gray-700 mb-1">Emoji Image</label>
              <div class="flex items-center gap-3">
                <label class="cursor-pointer inline-flex items-center gap-2 px-3 py-1.5 text-sm border border-gray-300 rounded-md hover:bg-gray-50 transition-colors">
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
                  <img v-if="filePreview" :src="filePreview" alt="Preview" class="w-8 h-8 object-contain rounded border border-gray-200" />
                  <span class="text-sm text-gray-600 truncate max-w-[150px]">{{ selectedFile.name }}</span>
                  <button type="button" class="text-gray-400 hover:text-red-500 transition-colors" @click="clearFile">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
              <p class="mt-1 text-xs text-gray-500">PNG, GIF, WebP, or JPEG. Max 512x512 pixels.</p>
            </div>

            <!-- URL input -->
            <div v-else>
              <label class="block text-sm font-medium text-gray-700 mb-1">Image URL</label>
              <input
                v-model="newImageUrl"
                type="url"
                class="form-input w-full"
                placeholder="https://example.com/emoji.png"
              />
              <p class="mt-1 text-xs text-gray-500">URL to the emoji image (PNG, GIF, or WebP).</p>
            </div>

            <div v-if="createError" class="text-sm text-red-600">{{ createError }}</div>

            <button
              type="submit"
              class="button primary"
              :disabled="formBusy || !formValid"
            >
              {{ formBusy ? 'Adding...' : 'Add Emoji' }}
            </button>
          </form>
        </div>

        <!-- Board emoji list -->
        <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wide mb-3">
          Board Emojis ({{ boardEmojis.length }})
        </h3>

        <div v-if="boardEmojis.length === 0" class="text-sm text-gray-500">
          No board-specific emojis yet. Emojis uploaded here will only be available within this board.
        </div>

        <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
          <div
            v-for="emoji in boardEmojis"
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
              @click="deleteBoardEmoji(emoji.id)"
            >
              Delete
            </button>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
