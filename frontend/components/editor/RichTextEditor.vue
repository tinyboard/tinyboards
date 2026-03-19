<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
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

const editorRef = ref<HTMLDivElement | null>(null)
const isSourceMode = ref(false)
const sourceContent = ref('')

const autocomplete = useEditorAutocomplete()

// Track formatting state for toolbar button highlighting
const activeFormats = ref<Set<string>>(new Set())

function updateActiveFormats (): void {
  const formats = new Set<string>()
  if (document.queryCommandState('bold')) formats.add('bold')
  if (document.queryCommandState('italic')) formats.add('italic')
  if (document.queryCommandState('underline')) formats.add('underline')
  if (document.queryCommandState('strikeThrough')) formats.add('strikethrough')
  if (document.queryCommandState('insertOrderedList')) formats.add('ordered-list')
  if (document.queryCommandState('insertUnorderedList')) formats.add('list')

  const block = document.queryCommandValue('formatBlock')
  if (block === 'h1' || block === 'h2' || block === 'h3') formats.add('heading')
  if (block === 'blockquote') formats.add('quote')

  activeFormats.value = formats
}

function exec (command: string, value?: string): void {
  document.execCommand(command, false, value)
  editorRef.value?.focus()
  emitUpdate()
  updateActiveFormats()
}

function emitUpdate (): void {
  if (editorRef.value) {
    emit('update:modelValue', editorRef.value.innerHTML)
  }
}

function handleInput (): void {
  emitUpdate()
  if (editorRef.value) {
    autocomplete.handleContentEditableInput(editorRef.value)
  }
}

function handleKeyDown (e: KeyboardEvent): void {
  if (autocomplete.isActive.value) {
    const consumed = autocomplete.handleKeyDown(e)
    if (consumed) {
      // If Enter or Tab was pressed, apply the selection
      if (e.key === 'Enter' || e.key === 'Tab') {
        const selected = autocomplete.getSelected()
        if (selected && editorRef.value) {
          autocomplete.applyToContentEditable(selected, editorRef.value, emitUpdate)
        }
      }
      return
    }
  }
}

function handleAutocompleteSelect (suggestion: AutocompleteSuggestion): void {
  if (editorRef.value) {
    autocomplete.applyToContentEditable(suggestion, editorRef.value, emitUpdate)
  }
}

function handleKeyUp (): void {
  updateActiveFormats()
}

function handleMouseUp (): void {
  updateActiveFormats()
  // Re-check autocomplete on mouse repositioning
  if (editorRef.value) {
    autocomplete.handleContentEditableInput(editorRef.value)
  }
}

function handlePaste (e: ClipboardEvent): void {
  e.preventDefault()
  // Paste as clean HTML, stripping dangerous attributes
  const html = e.clipboardData?.getData('text/html')
  const text = e.clipboardData?.getData('text/plain') ?? ''

  if (html) {
    // Clean the HTML - allow basic formatting tags only
    const temp = document.createElement('div')
    temp.innerHTML = html
    // Remove scripts, styles, event handlers
    temp.querySelectorAll('script, style, link, meta').forEach(el => el.remove())
    temp.querySelectorAll('*').forEach(el => {
      for (const attr of Array.from(el.attributes)) {
        if (attr.name.startsWith('on') || attr.name === 'style') {
          el.removeAttribute(attr.name)
        }
      }
    })
    document.execCommand('insertHTML', false, temp.innerHTML)
  } else {
    document.execCommand('insertText', false, text)
  }
  emitUpdate()
}

// Toolbar actions
function formatBold (): void { exec('bold') }
function formatItalic (): void { exec('italic') }
function formatUnderline (): void { exec('underline') }
function formatStrikethrough (): void { exec('strikeThrough') }

function insertLink (): void {
  const url = prompt('Enter URL:')
  if (url) {
    // Validate URL
    if (/^(https?:\/\/|mailto:|\/|#)/i.test(url)) {
      exec('createLink', url)
    }
  }
}

function insertImage (): void {
  const url = prompt('Enter image URL:')
  if (url && /^(https?:\/\/|\/)/i.test(url)) {
    exec('insertImage', url)
  }
}

function formatHeading (): void {
  const block = document.queryCommandValue('formatBlock')
  if (block === 'h2') {
    exec('formatBlock', 'p')
  } else {
    exec('formatBlock', 'h2')
  }
}

function formatQuote (): void {
  const block = document.queryCommandValue('formatBlock')
  if (block === 'blockquote') {
    exec('formatBlock', 'p')
  } else {
    exec('formatBlock', 'blockquote')
  }
}

function formatCode (): void {
  const selection = window.getSelection()
  if (selection && selection.rangeCount > 0) {
    const range = selection.getRangeAt(0)
    const selectedText = range.toString()
    if (selectedText) {
      const code = document.createElement('code')
      code.textContent = selectedText
      range.deleteContents()
      range.insertNode(code)
      emitUpdate()
    }
  }
}

function insertCodeBlock (): void {
  exec('insertHTML', '<pre><code>code here</code></pre><p><br></p>')
}

function formatBulletList (): void { exec('insertUnorderedList') }
function formatNumberedList (): void { exec('insertOrderedList') }
function insertHorizontalRule (): void { exec('insertHorizontalRule') }

function insertTable (): void {
  const html = `<table style="border-collapse: collapse; width: 100%;"><thead><tr><th style="border: 1px solid #d1d5db; padding: 8px; text-align: left;">Column 1</th><th style="border: 1px solid #d1d5db; padding: 8px; text-align: left;">Column 2</th><th style="border: 1px solid #d1d5db; padding: 8px; text-align: left;">Column 3</th></tr></thead><tbody><tr><td style="border: 1px solid #d1d5db; padding: 8px;">cell</td><td style="border: 1px solid #d1d5db; padding: 8px;">cell</td><td style="border: 1px solid #d1d5db; padding: 8px;">cell</td></tr></tbody></table><p><br></p>`
  exec('insertHTML', html)
}

function insertSpoiler (): void {
  exec('insertHTML', '<details><summary>Spoiler</summary><p>Hidden content here</p></details><p><br></p>')
}

function toggleSourceMode (): void {
  if (isSourceMode.value) {
    // Switching back to WYSIWYG
    if (editorRef.value) {
      editorRef.value.innerHTML = sourceContent.value
      emitUpdate()
    }
    isSourceMode.value = false
  } else {
    // Switching to source
    sourceContent.value = editorRef.value?.innerHTML ?? ''
    isSourceMode.value = true
  }
}

function handleSourceInput (): void {
  emit('update:modelValue', sourceContent.value)
}

// Set initial content
onMounted(() => {
  if (editorRef.value && props.modelValue) {
    editorRef.value.innerHTML = props.modelValue
  }
})

// Watch for external changes
watch(() => props.modelValue, (val) => {
  if (editorRef.value && val !== editorRef.value.innerHTML) {
    editorRef.value.innerHTML = val
  }
  if (isSourceMode.value) {
    sourceContent.value = val
  }
})

// Insert a quote block with attribution
function insertQuoteBlock (author: string, content: string, postNumber: number): void {
  const quoteHtml = `<blockquote><p><strong>@${author}</strong> said (<a href="#post-${postNumber}">#${postNumber}</a>):</p><p>${content.replace(/\n/g, '<br>')}</p></blockquote><p><br></p>`

  if (isSourceMode.value) {
    sourceContent.value = quoteHtml + sourceContent.value
    handleSourceInput()
    isSourceMode.value = false
    nextTick(() => {
      if (editorRef.value) {
        editorRef.value.innerHTML = sourceContent.value
      }
    })
  } else if (editorRef.value) {
    // Insert at the beginning of the editor
    editorRef.value.innerHTML = quoteHtml + editorRef.value.innerHTML
    emitUpdate()
  }
}

function clearContent (): void {
  if (editorRef.value) {
    editorRef.value.innerHTML = ''
  }
  sourceContent.value = ''
  emit('update:modelValue', '')
}

defineExpose({ insertQuoteBlock, clearContent })
</script>

<template>
  <div class="border border-gray-200 rounded-lg overflow-hidden">
    <!-- Toolbar -->
    <div class="px-2 py-1 bg-gray-50 border-b border-gray-200 flex items-center gap-0.5 flex-wrap">
      <!-- Text formatting -->
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('bold') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Bold"
        @click="formatBold"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 4h8a4 4 0 014 4 4 4 0 01-4 4H6z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M6 12h9a4 4 0 014 4 4 4 0 01-4 4H6z" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('italic') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Italic"
        @click="formatItalic"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 4h4m-2 0l-4 16m0 0h4" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('underline') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Underline"
        @click="formatUnderline"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 4v7a5 5 0 0010 0V4M5 20h14" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('strikethrough') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Strikethrough"
        @click="formatStrikethrough"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 4H9.5a3.5 3.5 0 000 7h5a3.5 3.5 0 010 7H6M4 12h16" />
        </svg>
      </button>

      <div class="w-px h-4 bg-gray-300 mx-0.5" />

      <!-- Links & media -->
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Insert link" @click="insertLink">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
        </svg>
      </button>
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Insert image" @click="insertImage">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      </button>

      <div class="w-px h-4 bg-gray-300 mx-0.5" />

      <!-- Block formatting -->
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('heading') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Heading"
        @click="formatHeading"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h8" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('quote') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Quote"
        @click="formatQuote"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('list') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Bullet list"
        @click="formatBulletList"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
        </svg>
      </button>
      <button
        type="button"
        class="p-1.5 rounded transition-colors"
        :class="activeFormats.has('ordered-list') ? 'bg-gray-300 text-gray-900' : 'hover:bg-gray-200 text-gray-600'"
        title="Numbered list"
        @click="formatNumberedList"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6h11M10 12h11M10 18h11M3 5l2 1V4M3 11h2l-2 2M3 17l2 2-2-0" />
        </svg>
      </button>

      <div class="w-px h-4 bg-gray-300 mx-0.5" />

      <!-- Code & extras -->
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Inline code" @click="formatCode">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
        </svg>
      </button>
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Code block" @click="insertCodeBlock">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h10" />
        </svg>
      </button>
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Table" @click="insertTable">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h18M3 14h18M8 4v16M16 4v16M4 4h16a1 1 0 011 1v14a1 1 0 01-1 1H4a1 1 0 01-1-1V5a1 1 0 011-1z" />
        </svg>
      </button>
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Horizontal rule" @click="insertHorizontalRule">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 12h16" />
        </svg>
      </button>
      <button type="button" class="p-1.5 rounded hover:bg-gray-200 text-gray-600" title="Spoiler" @click="insertSpoiler">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
        </svg>
      </button>

      <div class="ml-auto">
        <button
          type="button"
          class="px-2 py-1 text-xs rounded font-medium transition-colors"
          :class="isSourceMode ? 'bg-primary text-white' : 'text-gray-500 hover:text-gray-700 hover:bg-gray-200'"
          @click="toggleSourceMode"
        >
          {{ isSourceMode ? 'Visual' : 'Source' }}
        </button>
      </div>
    </div>

    <!-- WYSIWYG editor -->
    <div
      v-show="!isSourceMode"
      ref="editorRef"
      contenteditable="true"
      class="w-full p-4 text-sm focus:outline-none prose prose-sm max-w-none rich-editor"
      :style="{ minHeight: minHeight ?? '150px' }"
      :data-placeholder="placeholder ?? 'Start writing...'"
      @input="handleInput"
      @keydown="handleKeyDown"
      @keyup="handleKeyUp"
      @mouseup="handleMouseUp"
      @paste="handlePaste"
    />

    <!-- Autocomplete dropdown -->
    <EditorAutocompleteDropdown
      v-if="autocomplete.isActive.value"
      :suggestions="autocomplete.suggestions.value"
      :selected-index="autocomplete.selectedIndex.value"
      :position="autocomplete.dropdownPosition.value"
      @select="handleAutocompleteSelect"
    />

    <!-- Source mode -->
    <textarea
      v-show="isSourceMode"
      v-model="sourceContent"
      class="w-full p-4 text-sm border-0 focus:ring-0 resize-y font-mono"
      :style="{ minHeight: minHeight ?? '150px' }"
      placeholder="HTML source..."
      @input="handleSourceInput"
    />
  </div>
</template>

<style scoped>
.rich-editor:empty::before {
  content: attr(data-placeholder);
  color: #9ca3af;
  pointer-events: none;
}

.rich-editor :deep(blockquote) {
  border-left: 4px solid rgb(var(--color-primary) / 0.3);
  background: #f9fafb;
  border-radius: 0 4px 4px 0;
  padding: 8px 12px;
  margin: 8px 0;
}

.rich-editor :deep(code) {
  background: #f3f4f6;
  padding: 2px 4px;
  border-radius: 3px;
  font-size: 0.875em;
}

.rich-editor :deep(pre) {
  background: #1f2937;
  color: #e5e7eb;
  padding: 12px;
  border-radius: 6px;
  overflow-x: auto;
}

.rich-editor :deep(pre code) {
  background: none;
  padding: 0;
  color: inherit;
}

.rich-editor :deep(table) {
  border-collapse: collapse;
  width: 100%;
}

.rich-editor :deep(th),
.rich-editor :deep(td) {
  border: 1px solid #d1d5db;
  padding: 8px;
  text-align: left;
}

.rich-editor :deep(th) {
  background: #f9fafb;
  font-weight: 600;
}

.rich-editor :deep(img) {
  max-width: 100%;
  border-radius: 6px;
}

.rich-editor :deep(details) {
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 8px 12px;
  margin: 8px 0;
}

.rich-editor :deep(summary) {
  cursor: pointer;
  font-weight: 600;
  color: #374151;
}

.rich-editor :deep(hr) {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 16px 0;
}
</style>
