<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useEditorAutocomplete } from '~/composables/useEditorAutocomplete'
import type { AutocompleteSuggestion } from '~/composables/useEditorAutocomplete'

const props = defineProps<{
  modelValue: string
  placeholder?: string
  minHeight?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const content = ref(props.modelValue)
const textareaRef = ref<HTMLTextAreaElement | null>(null)
const showPreview = ref(false)

const autocomplete = useEditorAutocomplete()

watch(content, (val) => {
  emit('update:modelValue', val)
})

watch(() => props.modelValue, (val) => {
  if (val !== content.value) {
    content.value = val
  }
})

function wrapSelection (before: string, after: string) {
  const textarea = textareaRef.value
  if (!textarea) return

  const start = textarea.selectionStart
  const end = textarea.selectionEnd
  const text = content.value
  const selected = text.substring(start, end)

  if (selected) {
    content.value = text.substring(0, start) + before + selected + after + text.substring(end)
    nextTick(() => {
      textarea.focus()
      textarea.setSelectionRange(start + before.length, end + before.length)
    })
  } else {
    content.value = text.substring(0, start) + before + after + text.substring(end)
    nextTick(() => {
      textarea.focus()
      textarea.setSelectionRange(start + before.length, start + before.length)
    })
  }
}

function prependLine (prefix: string) {
  const textarea = textareaRef.value
  if (!textarea) return

  const start = textarea.selectionStart
  const text = content.value
  const lineStart = text.lastIndexOf('\n', start - 1) + 1
  content.value = text.substring(0, lineStart) + prefix + text.substring(lineStart)
  nextTick(() => {
    textarea.focus()
    textarea.setSelectionRange(start + prefix.length, start + prefix.length)
  })
}

function handleToolbarAction (action: string) {
  switch (action) {
    case 'bold': wrapSelection('**', '**'); break
    case 'italic': wrapSelection('*', '*'); break
    case 'strikethrough': wrapSelection('~~', '~~'); break
    case 'code': wrapSelection('`', '`'); break
    case 'link': wrapSelection('[', '](url)'); break
    case 'heading': prependLine('## '); break
    case 'quote': prependLine('> '); break
    case 'list': prependLine('- '); break
  }
}

function handleTextareaInput (): void {
  if (textareaRef.value) {
    autocomplete.handleTextareaInput(textareaRef.value)
  }
}

function handleTextareaKeyDown (e: KeyboardEvent): void {
  if (autocomplete.isActive.value) {
    const consumed = autocomplete.handleKeyDown(e)
    if (consumed) {
      if (e.key === 'Enter' || e.key === 'Tab') {
        const selected = autocomplete.getSelected()
        if (selected && textareaRef.value) {
          autocomplete.applyToTextarea(selected, textareaRef.value, content)
        }
      }
      return
    }
  }
}

function handleAutocompleteSelect (suggestion: AutocompleteSuggestion): void {
  if (textareaRef.value) {
    autocomplete.applyToTextarea(suggestion, textareaRef.value, content)
  }
}

function renderMarkdown (text: string): string {
  if (!text) return ''
  let html = text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/```(\w*)\n([\s\S]*?)```/g, '<pre><code>$2</code></pre>')
    .replace(/`([^`]+)`/g, '<code class="bg-gray-100 px-1 rounded text-sm">$1</code>')
    .replace(/^### (.+)$/gm, '<h3 class="text-lg font-semibold mt-3 mb-1">$1</h3>')
    .replace(/^## (.+)$/gm, '<h2 class="text-xl font-semibold mt-3 mb-1">$1</h2>')
    .replace(/^# (.+)$/gm, '<h1 class="text-2xl font-bold mt-3 mb-1">$1</h1>')
    .replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    .replace(/~~(.+?)~~/g, '<del>$1</del>')
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_match: string, text: string, href: string) => {
      const safeHref = /^(https?:\/\/|mailto:|\/|#)/i.test(href) ? href : '#'
      return `<a href="${safeHref}" class="text-blue-600 underline" target="_blank" rel="noopener">${text}</a>`
    })
    .replace(/^&gt; (.+)$/gm, '<blockquote class="border-l-4 border-gray-300 pl-3 text-gray-600 italic">$1</blockquote>')
    .replace(/^- (.+)$/gm, '<li class="ml-4">$1</li>')
    .replace(/\n\n/g, '</p><p class="mt-2">')
    .replace(/\n/g, '<br>')
  return '<p>' + html + '</p>'
}
</script>

<template>
  <div class="border border-gray-200 rounded overflow-hidden">
    <div class="flex items-center justify-between bg-gray-50 border-b border-gray-200">
      <EditorToolbar @action="handleToolbarAction" />
      <button
        type="button"
        class="px-2 py-1 text-xs text-gray-500 hover:text-gray-700 mr-1"
        @click="showPreview = !showPreview"
      >
        {{ showPreview ? 'Edit' : 'Preview' }}
      </button>
    </div>
    <!-- eslint-disable-next-line vue/no-v-html -->
    <div v-if="showPreview" class="p-3 min-h-[200px] prose prose-sm max-w-none" v-html="renderMarkdown(content)" />
    <textarea
      v-else
      ref="textareaRef"
      v-model="content"
      class="w-full p-3 text-sm border-0 focus:ring-0 resize-y font-mono"
      :style="{ minHeight: minHeight ?? '200px' }"
      :placeholder="placeholder ?? 'Write in markdown...'"
      @input="handleTextareaInput"
      @keydown="handleTextareaKeyDown"
    />

    <!-- Autocomplete dropdown -->
    <EditorAutocompleteDropdown
      v-if="autocomplete.isActive.value"
      :suggestions="autocomplete.suggestions.value"
      :selected-index="autocomplete.selectedIndex.value"
      :position="autocomplete.dropdownPosition.value"
      @select="handleAutocompleteSelect"
    />
  </div>
</template>
