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
  initialReactions?: { emoji: string; count: number }[] | null
  myReactionEmoji?: string | null
}>()

const authStore = useAuthStore()
const { reactions, acting, toggleReaction, addReaction, setInitial } = useReactions(props.targetType, props.targetId)

setInitial(props.initialReactions, props.myReactionEmoji)
watch(
  () => [props.initialReactions, props.myReactionEmoji] as const,
  ([next, my]) => setInitial(next, my),
)

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
      class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-sm border transition-colors leading-none"
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
        class="w-7 h-7 object-contain shrink-0"
      />
      <span v-else class="text-lg leading-none">{{ r.emoji }}</span>
      <span class="font-medium text-xs">{{ r.count }}</span>
    </button>

    <!-- Add reaction button -->
    <div v-if="authStore.isLoggedIn" class="relative">
      <button
        class="inline-flex items-center justify-center w-9 h-9 rounded-full text-sm border border-dashed border-gray-300 text-gray-400 hover:text-gray-600 hover:border-gray-400 transition-colors"
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
          class="w-11 h-11 sm:w-10 sm:h-10 flex items-center justify-center rounded hover:bg-gray-100 text-xl"
          :title="entry.type === 'custom' ? `:${entry.shortcode}:` : entry.value"
          @click="handlePickerSelect(entry)"
        >
          <img
            v-if="entry.type === 'custom' && entry.imageUrl"
            :src="entry.imageUrl"
            :alt="entry.shortcode ?? ''"
            class="w-8 h-8 object-contain"
          />
          <span v-else>{{ entry.value }}</span>
        </button>
      </div>
    </div>
  </div>
</template>
