<script setup lang="ts">
import { useReactions } from '~/composables/useReactions'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'

interface ReactionEmojiEntry {
  type: 'unicode' | 'custom'
  value?: string
  shortcode?: string
  imageUrl?: string
}

const props = defineProps<{
  targetType: 'post' | 'comment'
  targetId: string
  boardId?: string
}>()

const authStore = useAuthStore()
const { reactions, acting, toggleReaction, addReaction } = useReactions(props.targetType, props.targetId)

const showPicker = ref(false)
const settingsLoaded = ref(false)

const defaultEmojis: ReactionEmojiEntry[] = [
  { type: 'unicode', value: '👍' },
  { type: 'unicode', value: '❤️' },
  { type: 'unicode', value: '😂' },
  { type: 'unicode', value: '😮' },
  { type: 'unicode', value: '😢' },
  { type: 'unicode', value: '🔥' },
]

const pickerEmojis = ref<ReactionEmojiEntry[]>([...defaultEmojis])

// Custom emoji lookup for rendering reactions that use :shortcode: format
const customEmojiMap = ref<Map<string, string>>(new Map())

const REACTION_SETTINGS_QUERY = `
  query GetBoardReactionSettings($boardId: ID!) {
    getBoardReactionSettings(boardId: $boardId) {
      reactionEmojis
      reactionsEnabled
    }
  }
`

const reactionsEnabled = ref(true)

onMounted(async () => {
  if (props.boardId) {
    const { execute } = useGraphQL<{
      getBoardReactionSettings: {
        reactionEmojis: ReactionEmojiEntry[]
        reactionsEnabled: boolean
      } | null
    }>()
    const result = await execute(REACTION_SETTINGS_QUERY, {
      variables: { boardId: props.boardId },
    })
    if (result?.getBoardReactionSettings) {
      const settings = result.getBoardReactionSettings
      reactionsEnabled.value = settings.reactionsEnabled
      if (Array.isArray(settings.reactionEmojis) && settings.reactionEmojis.length > 0) {
        pickerEmojis.value = settings.reactionEmojis
        // Build custom emoji lookup
        for (const entry of settings.reactionEmojis) {
          if (entry.type === 'custom' && entry.shortcode && entry.imageUrl) {
            customEmojiMap.value.set(`:${entry.shortcode}:`, entry.imageUrl)
          }
        }
      }
    }
  }
  settingsLoaded.value = true
})

function emojiKey (entry: ReactionEmojiEntry): string {
  if (entry.type === 'custom' && entry.shortcode) {
    return `:${entry.shortcode}:`
  }
  return entry.value ?? ''
}

function isCustomEmoji (emoji: string): boolean {
  return emoji.startsWith(':') && emoji.endsWith(':') && emoji.length > 2
}

function getCustomEmojiUrl (emoji: string): string | undefined {
  return customEmojiMap.value.get(emoji)
}

async function handleToggle (emoji: string): Promise<void> {
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }
  await toggleReaction(emoji)
}

async function handlePickerSelect (entry: ReactionEmojiEntry): Promise<void> {
  showPicker.value = false
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }
  const key = emojiKey(entry)
  const existing = reactions.value.find(r => r.emoji === key)
  if (existing?.reacted) return
  await addReaction(key)
}
</script>

<template>
  <div v-if="reactionsEnabled" class="flex items-center gap-1.5 flex-wrap">
    <!-- Existing reactions -->
    <button
      v-for="r in reactions"
      :key="r.emoji"
      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs border transition-colors"
      :class="r.reacted
        ? 'border-primary/30 bg-primary/5 text-primary'
        : 'border-gray-200 bg-white text-gray-600 hover:border-gray-300'"
      :disabled="acting"
      @click="handleToggle(r.emoji)"
    >
      <img
        v-if="isCustomEmoji(r.emoji) && getCustomEmojiUrl(r.emoji)"
        :src="getCustomEmojiUrl(r.emoji)"
        :alt="r.emoji"
        class="w-4 h-4 object-contain"
      />
      <span v-else>{{ r.emoji }}</span>
      <span class="font-medium">{{ r.count }}</span>
    </button>

    <!-- Add reaction button -->
    <div v-if="authStore.isLoggedIn" class="relative">
      <button
        class="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs border border-dashed border-gray-300 text-gray-400 hover:text-gray-600 hover:border-gray-400 transition-colors"
        :disabled="acting"
        @click="showPicker = !showPicker"
      >
        +
      </button>

      <!-- Emoji picker dropdown -->
      <div
        v-if="showPicker"
        class="absolute bottom-full left-0 mb-1 bg-white border border-gray-200 rounded-lg shadow-lg p-2 z-20 flex gap-1 dropdown-enter"
      >
        <button
          v-for="entry in pickerEmojis"
          :key="emojiKey(entry)"
          class="w-10 h-10 sm:w-8 sm:h-8 flex items-center justify-center rounded hover:bg-gray-100 text-base"
          :title="entry.type === 'custom' ? `:${entry.shortcode}:` : entry.value"
          @click="handlePickerSelect(entry)"
        >
          <img
            v-if="entry.type === 'custom' && entry.imageUrl"
            :src="entry.imageUrl"
            :alt="entry.shortcode ?? ''"
            class="w-6 h-6 object-contain"
          />
          <span v-else>{{ entry.value }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
