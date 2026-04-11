<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useEmoji } from '~/composables/useEmoji'
import type { Board } from '~/types/generated'

const props = defineProps<{
  boardId?: string
}>()

const emit = defineEmits<{
  select: [emoji: string]
  selectCustom: [shortcode: string, imageUrl: string]
}>()

const search = ref('')
const activeCategory = ref('custom')
const { emojis: customEmojis, fetchAllAvailableEmojis } = useEmoji()
const currentBoard = useState<Board | null>('current-board', () => null)

onMounted(() => {
  const boardId = props.boardId ?? currentBoard.value?.id
  fetchAllAvailableEmojis(boardId)
})

// Common Unicode emoji organized by category
const unicodeCategories = [
  {
    id: 'smileys',
    label: 'Smileys',
    icon: '😀',
    type: 'unicode' as const,
    emojis: [
      '😀', '😃', '😄', '😁', '😆', '😅', '🤣', '😂', '🙂', '😊',
      '😇', '🥰', '😍', '🤩', '😘', '😗', '😚', '😙', '🥲', '😋',
      '😛', '😜', '🤪', '😝', '🤑', '🤗', '🤭', '🤫', '🤔', '🫡',
      '🤐', '🤨', '😐', '😑', '😶', '🫥', '😏', '😒', '🙄', '😬',
      '🤥', '😌', '😔', '😪', '🤤', '😴', '😷', '🤒', '🤕', '🤢',
      '🤮', '🥵', '🥶', '🥴', '😵', '🤯', '🤠', '🥳', '🥸', '😎',
      '🤓', '🧐', '😕', '🫤', '😟', '🙁', '😮', '😯', '😲', '😳',
      '🥺', '🥹', '😦', '😧', '😨', '😰', '😥', '😢', '😭', '😱',
      '😖', '😣', '😞', '😓', '😩', '😫', '🥱', '😤', '😡', '😠',
      '🤬', '😈', '👿', '💀', '☠️', '💩', '🤡', '👹', '👺', '👻',
      '👽', '👾', '🤖', '😺', '😸', '😹', '😻', '😼', '😽', '🙀',
    ],
  },
  {
    id: 'gestures',
    label: 'Gestures',
    icon: '👍',
    type: 'unicode' as const,
    emojis: [
      '👋', '🤚', '🖐️', '✋', '🖖', '🫱', '🫲', '🫳', '🫴', '👌',
      '🤌', '🤏', '✌️', '🤞', '🫰', '🤟', '🤘', '🤙', '👈', '👉',
      '👆', '🖕', '👇', '☝️', '🫵', '👍', '👎', '✊', '👊', '🤛',
      '🤜', '👏', '🙌', '🫶', '👐', '🤲', '🤝', '🙏', '✍️', '💪',
    ],
  },
  {
    id: 'hearts',
    label: 'Hearts',
    icon: '❤️',
    type: 'unicode' as const,
    emojis: [
      '❤️', '🧡', '💛', '💚', '💙', '💜', '🖤', '🤍', '🤎', '💔',
      '❤️‍🔥', '❤️‍🩹', '❣️', '💕', '💞', '💓', '💗', '💖', '💘', '💝',
      '💟', '♥️', '🫀', '💋', '💌', '💐', '🌹', '🥀', '🌷', '🌸',
    ],
  },
  {
    id: 'objects',
    label: 'Objects',
    icon: '🔧',
    type: 'unicode' as const,
    emojis: [
      '⭐', '🌟', '✨', '⚡', '🔥', '💥', '🎉', '🎊', '🎈', '🎁',
      '🏆', '🥇', '🥈', '🥉', '🎯', '🎮', '🎲', '🧩', '🎭', '🎨',
      '🔔', '🔕', '📢', '📣', '💬', '💭', '🗯️', '💡', '🔑', '🗝️',
      '🔒', '🔓', '📌', '📎', '🖊️', '✏️', '📝', '📁', '📂', '📊',
      '⚙️', '🔧', '🔨', '🛠️', '⚠️', '🚫', '❌', '✅', '☑️', '✔️',
    ],
  },
  {
    id: 'symbols',
    label: 'Symbols',
    icon: '➡️',
    type: 'unicode' as const,
    emojis: [
      '➡️', '⬅️', '⬆️', '⬇️', '↗️', '↘️', '↙️', '↖️', '↕️', '↔️',
      '🔄', '🔃', '🔀', '🔁', '🔂', '▶️', '⏩', '⏭️', '◀️', '⏪',
      '⏮️', '⏸️', '⏹️', '⏺️', '⏯️', '🔼', '🔽', '➕', '➖', '➗',
      '✖️', '♾️', '💲', '💱', '©️', '®️', '™️', '🔴', '🟠', '🟡',
      '🟢', '🔵', '🟣', '⚫', '⚪', '🟤', '🔶', '🔷', '🔸', '🔹',
    ],
  },
]

// Group custom emoji by category
const customCategories = computed(() => {
  if (customEmojis.value.length === 0) return []

  const grouped = new Map<string, typeof customEmojis.value>()
  for (const emoji of customEmojis.value) {
    const cat = emoji.category || 'Custom'
    if (!grouped.has(cat)) grouped.set(cat, [])
    grouped.get(cat)!.push(emoji)
  }

  return Array.from(grouped.entries()).map(([name, items]) => ({
    id: `custom-${name.toLowerCase().replace(/\s+/g, '-')}`,
    label: name,
    icon: null as string | null,
    iconUrl: items[0]?.imageUrl ?? null,
    type: 'custom' as const,
    items,
  }))
})

const hasCustom = computed(() => customEmojis.value.length > 0)

const allCategoryTabs = computed(() => {
  const tabs: Array<{ id: string; label: string; icon: string | null; iconUrl?: string | null }> = []
  if (hasCustom.value) {
    tabs.push({ id: 'custom', label: 'Custom', icon: null, iconUrl: customEmojis.value[0]?.imageUrl })
  }
  for (const cat of unicodeCategories) {
    tabs.push({ id: cat.id, label: cat.label, icon: cat.icon })
  }
  return tabs
})

// Compute the actual default category
const effectiveCategory = computed(() => {
  if (activeCategory.value === 'custom' && !hasCustom.value) return 'smileys'
  return activeCategory.value
})

const filteredContent = computed(() => {
  const q = search.value.toLowerCase()

  if (q) {
    // Search across all categories
    const results: Array<{ type: 'unicode'; emoji: string } | { type: 'custom'; shortcode: string; imageUrl: string }> = []

    // Search custom emoji by shortcode
    for (const emoji of customEmojis.value) {
      if (emoji.shortcode.toLowerCase().includes(q)) {
        results.push({ type: 'custom', shortcode: emoji.shortcode, imageUrl: emoji.imageUrl })
      }
    }

    return { searching: true, results }
  }

  return { searching: false, results: [] }
})

function handleUnicodeSelect (emoji: string): void {
  emit('select', emoji)
}

function handleCustomSelect (shortcode: string, imageUrl: string): void {
  emit('selectCustom', shortcode, imageUrl)
}
</script>

<template>
  <div class="emoji-picker">
    <!-- Search -->
    <div class="emoji-search">
      <input
        v-model="search"
        type="text"
        placeholder="Search emoji..."
        class="emoji-search-input"
      />
    </div>

    <!-- Category tabs -->
    <div v-if="!search" class="emoji-tabs">
      <button
        v-for="tab in allCategoryTabs"
        :key="tab.id"
        type="button"
        class="emoji-tab"
        :class="{ active: effectiveCategory === tab.id }"
        :title="tab.label"
        @click="activeCategory = tab.id"
      >
        <template v-if="tab.icon">{{ tab.icon }}</template>
        <img v-else-if="tab.iconUrl" :src="tab.iconUrl" class="emoji-tab-img" :alt="tab.label" />
        <span v-else class="text-xs">{{ tab.label.charAt(0) }}</span>
      </button>
    </div>

    <!-- Search results -->
    <div v-if="filteredContent.searching" class="emoji-grid-container">
      <div class="emoji-grid">
        <template v-for="item in filteredContent.results" :key="item.type === 'unicode' ? item.emoji : item.shortcode">
          <button
            v-if="item.type === 'unicode'"
            type="button"
            class="emoji-btn"
            :title="item.emoji"
            @click="handleUnicodeSelect(item.emoji)"
          >
            {{ item.emoji }}
          </button>
          <button
            v-else
            type="button"
            class="emoji-btn custom-emoji-btn"
            :title="`:${item.shortcode}:`"
            @click="handleCustomSelect(item.shortcode, item.imageUrl)"
          >
            <img :src="item.imageUrl" :alt="item.shortcode" class="custom-emoji-img" />
          </button>
        </template>
      </div>
      <p v-if="filteredContent.results.length === 0" class="text-center text-xs text-gray-400 py-4">
        No emoji found
      </p>
    </div>

    <!-- Category content -->
    <div v-else class="emoji-grid-container">
      <!-- Custom emoji categories -->
      <template v-if="effectiveCategory === 'custom' || effectiveCategory.startsWith('custom-')">
        <div v-for="cat in customCategories" :key="cat.id" class="emoji-category">
          <div v-if="customCategories.length > 1" class="emoji-category-label">{{ cat.label }}</div>
          <div class="emoji-grid">
            <button
              v-for="emoji in cat.items"
              :key="emoji.id"
              type="button"
              class="emoji-btn custom-emoji-btn"
              :title="`:${emoji.shortcode}:`"
              @click="handleCustomSelect(emoji.shortcode, emoji.imageUrl)"
            >
              <img :src="emoji.imageUrl" :alt="emoji.shortcode" class="custom-emoji-img" />
            </button>
          </div>
        </div>
        <p v-if="customCategories.length === 0" class="text-center text-xs text-gray-400 py-4">
          No custom emoji available
        </p>
      </template>

      <!-- Unicode emoji categories -->
      <template v-else>
        <div
          v-for="cat in unicodeCategories.filter(c => c.id === effectiveCategory)"
          :key="cat.id"
          class="emoji-category"
        >
          <div class="emoji-grid">
            <button
              v-for="emoji in cat.emojis"
              :key="emoji"
              type="button"
              class="emoji-btn"
              :title="emoji"
              @click="handleUnicodeSelect(emoji)"
            >
              {{ emoji }}
            </button>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.emoji-picker {
  width: 320px;
  max-height: 360px;
  display: flex;
  flex-direction: column;
}

.emoji-search {
  padding: 8px;
  border-bottom: 1px solid #e5e7eb;
}

.emoji-search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
}

.emoji-search-input:focus {
  border-color: rgb(var(--color-primary, 99 102 241));
  box-shadow: 0 0 0 2px rgb(var(--color-primary, 99 102 241) / 0.15);
}

.emoji-tabs {
  display: flex;
  gap: 2px;
  padding: 4px 8px;
  border-bottom: 1px solid #e5e7eb;
  overflow-x: auto;
}

.emoji-tab {
  padding: 4px 6px;
  border-radius: 4px;
  font-size: 18px;
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color 0.15s;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 32px;
  min-height: 32px;
}

.emoji-tab:hover {
  background-color: #f3f4f6;
}

.emoji-tab.active {
  background-color: #e5e7eb;
}

.emoji-tab-img {
  width: 20px;
  height: 20px;
  object-fit: contain;
}

.emoji-grid-container {
  overflow-y: auto;
  padding: 8px;
  flex: 1;
}

.emoji-category-label {
  font-size: 11px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 4px 2px 6px;
}

.emoji-grid {
  display: grid;
  grid-template-columns: repeat(8, 1fr);
  gap: 2px;
}

.emoji-btn {
  width: 34px;
  height: 34px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.1s, transform 0.1s;
  line-height: 1;
}

.emoji-btn:hover {
  background-color: #f3f4f6;
  transform: scale(1.15);
}

.custom-emoji-btn {
  padding: 4px;
}

.custom-emoji-img {
  width: 24px;
  height: 24px;
  object-fit: contain;
}
</style>
