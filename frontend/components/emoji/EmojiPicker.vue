<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { unicodeEmojis, type UnicodeEmoji } from '~/data/emojis'
import { useEmoji } from '~/composables/useEmoji'

const props = withDefaults(defineProps<{
  boardId?: string
}>(), {
  boardId: undefined,
})

const emit = defineEmits<{
  // For unicode emojis the payload is the unicode character.
  // For custom emojis the payload is a :shortcode: string the backend will
  // expand to an <img> tag during message rendering.
  select: [token: string]
}>()

const search = ref('')
const customEmoji = useEmoji()

onMounted(() => {
  customEmoji.fetchAllAvailableEmojis(props.boardId)
})

const filteredUnicode = computed<UnicodeEmoji[]>(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) { return unicodeEmojis.slice(0, 96) }
  return unicodeEmojis
    .filter(e => e.shortcode.includes(q) || e.keywords.some(k => k.includes(q)))
    .slice(0, 96)
})

const filteredCustom = computed(() => {
  const q = search.value.trim().toLowerCase()
  const list = customEmoji.emojis.value
  if (!q) { return list }
  return list.filter(e => e.shortcode.toLowerCase().includes(q))
})

function pickUnicode (e: UnicodeEmoji): void {
  emit('select', e.emoji)
}

function pickCustom (shortcode: string): void {
  emit('select', `:${shortcode}:`)
}
</script>

<template>
  <ClientOnly>
    <div class="bg-white border border-gray-200 rounded-lg shadow-lg w-[280px]">
      <div class="p-2 border-b border-gray-100">
        <input
          v-model="search"
          type="text"
          class="form-input form-input-sm w-full text-sm"
          placeholder="Search emoji..."
        >
      </div>

      <div class="max-h-64 overflow-y-auto p-2">
        <div v-if="filteredCustom.length > 0" class="mb-2">
          <p class="text-[10px] uppercase tracking-wide text-gray-400 px-1 mb-1">
            Custom
          </p>
          <div class="grid grid-cols-7 gap-1">
            <button
              v-for="e in filteredCustom"
              :key="e.id"
              type="button"
              class="w-9 h-9 flex items-center justify-center rounded hover:bg-gray-100"
              :title="`:${e.shortcode}:`"
              @click="pickCustom(e.shortcode)"
            >
              <img :src="e.imageUrl" :alt="e.shortcode" class="w-7 h-7 object-contain">
            </button>
          </div>
        </div>

        <div v-if="filteredUnicode.length > 0">
          <p
            v-if="filteredCustom.length > 0"
            class="text-[10px] uppercase tracking-wide text-gray-400 px-1 mb-1"
          >
            Standard
          </p>
          <div class="grid grid-cols-8 gap-1">
            <button
              v-for="e in filteredUnicode"
              :key="e.shortcode"
              type="button"
              class="w-7 h-7 flex items-center justify-center rounded text-lg leading-none hover:bg-gray-100"
              :title="`:${e.shortcode}:`"
              @click="pickUnicode(e)"
            >
              {{ e.emoji }}
            </button>
          </div>
        </div>

        <p
          v-if="filteredUnicode.length === 0 && filteredCustom.length === 0"
          class="text-xs text-gray-400 text-center py-6"
        >
          No emoji match "{{ search }}"
        </p>
      </div>
    </div>

    <template #fallback>
      <div class="bg-white border border-gray-200 rounded-lg shadow-lg p-4">
        <CommonLoadingSpinner size="sm" />
      </div>
    </template>
  </ClientOnly>
</template>
