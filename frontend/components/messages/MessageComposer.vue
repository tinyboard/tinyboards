<script setup lang="ts">
import { ref, nextTick, onMounted, onBeforeUnmount } from 'vue'
import { useEditorAutocomplete } from '~/composables/useEditorAutocomplete'
import type { AutocompleteSuggestion } from '~/composables/useEditorAutocomplete'

const props = withDefaults(defineProps<{
  disabled?: boolean
  placeholder?: string
}>(), {
  disabled: false,
  placeholder: 'Type a message...',
})

const emit = defineEmits<{
  send: [body: string]
}>()

const body = ref('')
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const showEmojiPicker = ref(false)
const emojiContainerRef = ref<HTMLElement | null>(null)

const autocomplete = useEditorAutocomplete()

function autoResize (): void {
  const ta = textareaRef.value
  if (!ta) { return }
  ta.style.height = 'auto'
  ta.style.height = Math.min(ta.scrollHeight, 160) + 'px'
}

function handleInput (): void {
  autoResize()
  if (textareaRef.value) {
    autocomplete.handleTextareaInput(textareaRef.value)
  }
}

function handleSend (): void {
  const trimmed = body.value.trim()
  if (!trimmed || props.disabled) { return }
  emit('send', trimmed)
  body.value = ''
  nextTick(autoResize)
}

function handleKeyDown (e: KeyboardEvent): void {
  if (autocomplete.isActive.value) {
    const consumed = autocomplete.handleKeyDown(e)
    if (consumed) {
      if (e.key === 'Enter' || e.key === 'Tab') {
        const selected = autocomplete.getSelected()
        if (selected && textareaRef.value) {
          autocomplete.applyToTextarea(selected, textareaRef.value, body)
          nextTick(autoResize)
        }
      }
      return
    }
  }

  // Send on Enter (without Shift). Shift+Enter inserts a newline.
  if (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey) {
    e.preventDefault()
    handleSend()
  }
}

function handleAutocompleteSelect (suggestion: AutocompleteSuggestion): void {
  if (textareaRef.value) {
    autocomplete.applyToTextarea(suggestion, textareaRef.value, body)
    nextTick(autoResize)
  }
}

function insertAtCursor (token: string): void {
  const ta = textareaRef.value
  if (!ta) {
    body.value += token
    return
  }
  const start = ta.selectionStart
  const end = ta.selectionEnd
  body.value = body.value.substring(0, start) + token + body.value.substring(end)
  const newPos = start + token.length
  nextTick(() => {
    ta.focus()
    ta.setSelectionRange(newPos, newPos)
    autoResize()
  })
}

function handleEmojiSelect (token: string): void {
  insertAtCursor(token)
  showEmojiPicker.value = false
}

function toggleEmojiPicker (): void {
  showEmojiPicker.value = !showEmojiPicker.value
}

function handleClickOutside (e: MouseEvent): void {
  if (!showEmojiPicker.value) { return }
  const target = e.target as Node
  if (emojiContainerRef.value && !emojiContainerRef.value.contains(target)) {
    showEmojiPicker.value = false
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
  autoResize()
})

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside)
})
</script>

<template>
  <div class="flex items-end gap-2">
    <div ref="emojiContainerRef" class="relative">
      <button
        type="button"
        class="w-9 h-9 flex items-center justify-center rounded text-gray-500 hover:text-gray-700 hover:bg-gray-100 transition-colors"
        :class="{ 'text-primary': showEmojiPicker }"
        :aria-label="showEmojiPicker ? 'Close emoji picker' : 'Open emoji picker'"
        @click="toggleEmojiPicker"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="w-5 h-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <circle cx="12" cy="12" r="9" />
          <circle cx="9" cy="10" r="0.9" fill="currentColor" />
          <circle cx="15" cy="10" r="0.9" fill="currentColor" />
          <path d="M8 14.5c1 1.3 2.4 2 4 2s3-.7 4-2" stroke-linecap="round" />
        </svg>
      </button>

      <div
        v-if="showEmojiPicker"
        class="absolute bottom-full left-0 mb-2 z-30"
      >
        <EmojiPicker @select="handleEmojiSelect" />
      </div>
    </div>

    <textarea
      ref="textareaRef"
      v-model="body"
      rows="1"
      class="form-input flex-1 resize-none py-2 leading-snug"
      style="min-height: 36px; max-height: 160px;"
      :placeholder="placeholder"
      :disabled="disabled"
      @input="handleInput"
      @keydown="handleKeyDown"
    />

    <button
      type="button"
      class="button primary"
      :disabled="disabled || !body.trim()"
      @click="handleSend"
    >
      Send
    </button>

    <EditorAutocompleteDropdown
      v-if="autocomplete.isActive.value"
      :suggestions="autocomplete.suggestions.value"
      :selected-index="autocomplete.selectedIndex.value"
      :position="autocomplete.dropdownPosition.value"
      @select="handleAutocompleteSelect"
    />
  </div>
</template>
