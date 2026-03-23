<script setup lang="ts">
import { ref, watch, onBeforeUnmount, nextTick } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Underline from '@tiptap/extension-underline'
import TextStyle from '@tiptap/extension-text-style'
import Color from '@tiptap/extension-color'
import Highlight from '@tiptap/extension-highlight'
import Link from '@tiptap/extension-link'
import Table from '@tiptap/extension-table'
import TableRow from '@tiptap/extension-table-row'
import TableCell from '@tiptap/extension-table-cell'
import TableHeader from '@tiptap/extension-table-header'
import Placeholder from '@tiptap/extension-placeholder'
import TextAlign from '@tiptap/extension-text-align'
import Subscript from '@tiptap/extension-subscript'
import Superscript from '@tiptap/extension-superscript'
import Youtube from '@tiptap/extension-youtube'
import CodeBlockLowlight from '@tiptap/extension-code-block-lowlight'
import { common, createLowlight } from 'lowlight'
import { VueNodeViewRenderer } from '@tiptap/vue-3'
import ForumQuote from './extensions/ForumQuote'
import ImageResize from './extensions/ImageResize'
import ImageUpload from './extensions/ImageUpload'
import CodeBlockComponent from './CodeBlockComponent.vue'
import { useFileUpload } from '~/composables/useFileUpload'
import { useEditorAutocomplete } from '~/composables/useEditorAutocomplete'
import type { AutocompleteSuggestion } from '~/composables/useEditorAutocomplete'
import { sanitizeHtml } from '~/utils/sanitize'

const props = defineProps<{
  modelValue: string
  placeholder?: string
  minHeight?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const { uploadFile: doUpload, uploading } = useFileUpload()
const autocomplete = useEditorAutocomplete()

// Dropdowns
const showTextColor = ref(false)
const showHighlight = ref(false)
const showEmojiPicker = ref(false)
const isSourceMode = ref(false)
const sourceContent = ref('')
const fileInput = ref<HTMLInputElement | null>(null)

// Create lowlight instance with common languages
const lowlight = createLowlight(common)

const editor = useEditor({
  content: props.modelValue || '',
  extensions: [
    StarterKit.configure({
      codeBlock: false, // replaced by CodeBlockLowlight
      history: { depth: 100 },
    }),
    Underline,
    TextStyle,
    Color,
    Highlight.configure({ multicolor: true }),
    Link.configure({
      openOnClick: false,
      HTMLAttributes: { rel: 'noopener noreferrer nofollow', target: '_blank' },
    }),
    ImageResize.configure({ inline: false, allowBase64: false }),
    Table.configure({ resizable: true }),
    TableRow,
    TableCell,
    TableHeader,
    Placeholder.configure({ placeholder: props.placeholder ?? 'Start writing...' }),
    TextAlign.configure({ types: ['heading', 'paragraph'] }),
    Subscript,
    Superscript,
    Youtube.configure({ inline: false, ccLanguage: 'en' }),
    CodeBlockLowlight.extend({
      addNodeView () {
        return VueNodeViewRenderer(CodeBlockComponent)
      },
    }).configure({ lowlight }),
    ForumQuote,
    ImageUpload.configure({
      uploadFn: doUpload,
      onUploadStart: () => {},
      onUploadEnd: () => {},
    }),
  ],
  onUpdate: ({ editor: e }) => {
    if (!isSourceMode.value) {
      emit('update:modelValue', e.getHTML())
    }
    // Trigger autocomplete detection on content changes
    const dom = e.view.dom as HTMLElement
    autocomplete.handleContentEditableInput(dom)
  },
  editorProps: {
    handleKeyDown: (_view, event) => {
      if (autocomplete.isActive.value) {
        const consumed = autocomplete.handleKeyDown(event)
        if (consumed) {
          if (event.key === 'Enter' || event.key === 'Tab') {
            const selected = autocomplete.getSelected()
            if (selected && editor.value) {
              autocomplete.applyToTipTap(selected, editor.value)
            }
          }
          return true
        }
      }
      return false
    },
  },
})

// Sync external changes
watch(() => props.modelValue, (val) => {
  if (editor.value && val !== editor.value.getHTML()) {
    editor.value.commands.setContent(val, false)
  }
  if (isSourceMode.value) {
    sourceContent.value = val
  }
})

onBeforeUnmount(() => {
  editor.value?.destroy()
})

// Toolbar actions
function toggleBold () { editor.value?.chain().focus().toggleBold().run() }
function toggleItalic () { editor.value?.chain().focus().toggleItalic().run() }
function toggleUnderline () { editor.value?.chain().focus().toggleUnderline().run() }
function toggleStrike () { editor.value?.chain().focus().toggleStrike().run() }

function toggleHeading () {
  if (editor.value?.isActive('heading', { level: 2 })) {
    editor.value.chain().focus().setParagraph().run()
  } else {
    editor.value?.chain().focus().toggleHeading({ level: 2 }).run()
  }
}

function toggleBlockquote () { editor.value?.chain().focus().toggleBlockquote().run() }
function toggleBulletList () { editor.value?.chain().focus().toggleBulletList().run() }
function toggleOrderedList () { editor.value?.chain().focus().toggleOrderedList().run() }
function toggleCodeBlock () { editor.value?.chain().focus().toggleCodeBlock().run() }
function toggleCode () { editor.value?.chain().focus().toggleCode().run() }
function insertHr () { editor.value?.chain().focus().setHorizontalRule().run() }
function undo () { editor.value?.chain().focus().undo().run() }
function redo () { editor.value?.chain().focus().redo().run() }

function setTextColor (color: string) {
  editor.value?.chain().focus().setColor(color).run()
  showTextColor.value = false
}
function clearTextColor () {
  editor.value?.chain().focus().unsetColor().run()
  showTextColor.value = false
}
function setHighlight (color: string) {
  editor.value?.chain().focus().setHighlight({ color }).run()
  showHighlight.value = false
}
function clearHighlight () {
  editor.value?.chain().focus().unsetHighlight().run()
  showHighlight.value = false
}

function setTextAlign (align: string) {
  editor.value?.chain().focus().setTextAlign(align).run()
}

function insertLink () {
  const previousUrl = editor.value?.getAttributes('link')?.href ?? ''
  const url = prompt('Enter URL:', previousUrl)
  if (url === null) return
  if (url === '') {
    editor.value?.chain().focus().extendMarkRange('link').unsetLink().run()
    return
  }
  if (/^(https?:\/\/|mailto:|\/|#)/i.test(url)) {
    editor.value?.chain().focus().extendMarkRange('link').setLink({ href: url }).run()
  }
}

function triggerImageUpload () {
  fileInput.value?.click()
}

async function handleFileSelect (event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const url = await doUpload(file)
  if (url) {
    editor.value?.chain().focus().setImage({ src: url }).run()
  }
  input.value = ''
}

function insertImageUrl () {
  const url = prompt('Enter image URL:')
  if (url && /^(https?:\/\/|\/)/i.test(url)) {
    editor.value?.chain().focus().setImage({ src: url }).run()
  }
}

function insertYoutube () {
  const url = prompt('Enter YouTube video URL:')
  if (url) {
    editor.value?.chain().focus().setYoutubeVideo({ src: url }).run()
  }
}

function insertTable () {
  editor.value?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
}

function insertSpoiler () {
  editor.value?.chain().focus().insertContent('<details><summary>Spoiler</summary><p>Hidden content here</p></details><p></p>').run()
}

function handleEmojiSelect (emoji: string) {
  editor.value?.chain().focus().insertContent(emoji).run()
  showEmojiPicker.value = false
}

function handleAutocompleteSelect (suggestion: AutocompleteSuggestion) {
  if (editor.value) {
    autocomplete.applyToTipTap(suggestion, editor.value)
  }
}

function handleCustomEmojiSelect (shortcode: string, _imageUrl: string) {
  editor.value?.chain().focus().insertContent(`:${shortcode}:`).run()
  showEmojiPicker.value = false
}

function toggleSourceMode () {
  if (isSourceMode.value) {
    editor.value?.commands.setContent(sourceContent.value, false)
    emit('update:modelValue', sourceContent.value)
    isSourceMode.value = false
  } else {
    sourceContent.value = editor.value?.getHTML() ?? ''
    isSourceMode.value = true
  }
}

function handleSourceInput () {
  emit('update:modelValue', sourceContent.value)
}

// Quote insertion for thread replies
function insertQuoteBlock (author: string, htmlContent: string, postNumber: number): void {
  const clean = sanitizeHtml(htmlContent)
  editor.value?.chain().focus().insertContent(
    `<blockquote class="forum-quote" data-author="${author}" data-post-number="${postNumber}"><div class="forum-quote-header"><strong>@${author}</strong> said (<a href="#post-${postNumber}">#${postNumber}</a>):</div><div class="forum-quote-body">${clean}</div></blockquote><p></p>`,
  ).run()
}

function clearContent (): void {
  editor.value?.commands.clearContent()
}

// Close dropdowns when clicking outside
function closeDropdowns (e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.dropdown-trigger-text-color')) showTextColor.value = false
  if (!target.closest('.dropdown-trigger-highlight')) showHighlight.value = false
  if (!target.closest('.dropdown-trigger-emoji')) showEmojiPicker.value = false
}

defineExpose({ insertQuoteBlock, clearContent })
</script>

<template>
  <div class="rich-editor-container" @click="closeDropdowns">
    <!-- Hidden file input for image uploads -->
    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      class="hidden"
      @change="handleFileSelect"
    />

    <!-- Toolbar -->
    <div class="toolbar">
      <!-- History -->
      <button type="button" class="tb-btn" title="Undo" :disabled="!editor?.can().undo()" @click="undo">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="1 4 1 10 7 10" /><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Redo" :disabled="!editor?.can().redo()" @click="redo">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10" /><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" /></svg>
      </button>

      <div class="tb-sep" />

      <!-- Text formatting -->
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('bold') }" title="Bold" @click="toggleBold">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4h8a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" /><path d="M6 12h9a4 4 0 0 1 4 4 4 4 0 0 1-4 4H6z" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('italic') }" title="Italic" @click="toggleItalic">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="4" x2="10" y2="4" /><line x1="14" y1="20" x2="5" y2="20" /><line x1="15" y1="4" x2="9" y2="20" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('underline') }" title="Underline" @click="toggleUnderline">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3v7a6 6 0 0 0 6 6 6 6 0 0 0 6-6V3" /><line x1="4" y1="21" x2="20" y2="21" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('strike') }" title="Strikethrough" @click="toggleStrike">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 4H9a3 3 0 0 0-2.83 4" /><path d="M14 12a4 4 0 0 1 0 8H6" /><line x1="4" y1="12" x2="20" y2="12" /></svg>
      </button>

      <div class="tb-sep" />

      <!-- Text Color -->
      <div class="dropdown-trigger-text-color relative">
        <button
          type="button"
          class="tb-btn"
          title="Text color"
          @click.stop="showTextColor = !showTextColor; showHighlight = false; showEmojiPicker = false"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 20h16" /><path d="m8.5 4 3.5 10 3.5-10" /><path d="M7 14h10" /></svg>
          <span
            class="color-indicator"
            :style="{ backgroundColor: editor?.getAttributes('textStyle')?.color ?? '#000' }"
          />
        </button>
        <div v-if="showTextColor" class="dropdown-panel">
          <EditorColorPalette type="text" :current-color="editor?.getAttributes('textStyle')?.color" @select="setTextColor" @clear="clearTextColor" />
        </div>
      </div>

      <!-- Highlight Color -->
      <div class="dropdown-trigger-highlight relative">
        <button
          type="button"
          class="tb-btn"
          title="Highlight color"
          @click.stop="showHighlight = !showHighlight; showTextColor = false; showEmojiPicker = false"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 11-6 6v3h9l3-3" /><path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4" /></svg>
          <span
            class="color-indicator highlight-indicator"
            :style="{ backgroundColor: editor?.getAttributes('highlight')?.color ?? '#fef08a' }"
          />
        </button>
        <div v-if="showHighlight" class="dropdown-panel">
          <EditorColorPalette type="highlight" :current-color="editor?.getAttributes('highlight')?.color" @select="setHighlight" @clear="clearHighlight" />
        </div>
      </div>

      <div class="tb-sep" />

      <!-- Block formatting -->
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('heading') }" title="Heading" @click="toggleHeading">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M4 12h8" /><path d="M4 18V6" /><path d="M12 18V6" /><path d="m17 12 3-2v8" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('blockquote') }" title="Quote" @click="toggleBlockquote">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 21c3 0 7-1 7-8V5c0-1.25-.756-2.017-2-2H4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2 1 0 1 0 1 1v1c0 1-1 2-2 2s-1 .008-1 1.031V21z" /><path d="M15 21c3 0 7-1 7-8V5c0-1.25-.757-2.017-2-2h-4c-1.25 0-2 .75-2 1.972V11c0 1.25.75 2 2 2h.75c0 2.25.25 4-2.75 4v3z" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('bulletList') }" title="Bullet list" @click="toggleBulletList">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" /><line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('orderedList') }" title="Numbered list" @click="toggleOrderedList">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="10" y1="6" x2="21" y2="6" /><line x1="10" y1="12" x2="21" y2="12" /><line x1="10" y1="18" x2="21" y2="18" /><path d="M4 6h1v4" /><path d="M4 10h2" /><path d="M6 18H4c0-1 2-2 2-3s-1-1.5-2-1" /></svg>
      </button>

      <!-- Alignment -->
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive({ textAlign: 'left' }) }" title="Align left" @click="setTextAlign('left')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="17" y1="10" x2="3" y2="10" /><line x1="21" y1="6" x2="3" y2="6" /><line x1="21" y1="14" x2="3" y2="14" /><line x1="17" y1="18" x2="3" y2="18" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive({ textAlign: 'center' }) }" title="Align center" @click="setTextAlign('center')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="10" x2="6" y2="10" /><line x1="21" y1="6" x2="3" y2="6" /><line x1="21" y1="14" x2="3" y2="14" /><line x1="18" y1="18" x2="6" y2="18" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive({ textAlign: 'right' }) }" title="Align right" @click="setTextAlign('right')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="21" y1="10" x2="7" y2="10" /><line x1="21" y1="6" x2="3" y2="6" /><line x1="21" y1="14" x2="3" y2="14" /><line x1="21" y1="18" x2="7" y2="18" /></svg>
      </button>

      <div class="tb-sep" />

      <!-- Links & media -->
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('link') }" title="Insert link" @click="insertLink">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Upload image" @click="triggerImageUpload">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2" /><circle cx="8.5" cy="8.5" r="1.5" /><polyline points="21 15 16 10 5 21" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Image from URL" @click="insertImageUrl">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" /></svg>
      </button>
      <button type="button" class="tb-btn" title="YouTube video" @click="insertYoutube">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22.54 6.42a2.78 2.78 0 0 0-1.94-2C18.88 4 12 4 12 4s-6.88 0-8.6.46a2.78 2.78 0 0 0-1.94 2A29 29 0 0 0 1 11.75a29 29 0 0 0 .46 5.33A2.78 2.78 0 0 0 3.4 19.13C5.12 19.56 12 19.56 12 19.56s6.88 0 8.6-.46a2.78 2.78 0 0 0 1.94-2 29 29 0 0 0 .46-5.25 29 29 0 0 0-.46-5.33z" /><polygon points="9.75 15.02 15.5 11.75 9.75 8.48 9.75 15.02" /></svg>
      </button>

      <div class="tb-sep" />

      <!-- Code & extras -->
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('code') }" title="Inline code" @click="toggleCode">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6" /><polyline points="8 6 2 12 8 18" /></svg>
      </button>
      <button type="button" class="tb-btn" :class="{ active: editor?.isActive('codeBlock') }" title="Code block" @click="toggleCodeBlock">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 16 4-4-4-4" /><path d="m6 8-4 4 4 4" /><path d="m14.5 4-5 16" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Table" @click="insertTable">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v18" /><rect width="18" height="18" x="3" y="3" rx="2" /><path d="M3 9h18" /><path d="M3 15h18" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Horizontal rule" @click="insertHr">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="2" y1="12" x2="22" y2="12" /></svg>
      </button>
      <button type="button" class="tb-btn" title="Spoiler" @click="insertSpoiler">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" /><line x1="1" y1="1" x2="23" y2="23" /></svg>
      </button>

      <!-- Emoji picker -->
      <div class="dropdown-trigger-emoji relative">
        <button
          type="button"
          class="tb-btn"
          title="Insert emoji"
          @click.stop="showEmojiPicker = !showEmojiPicker; showTextColor = false; showHighlight = false"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><path d="M8 14s1.5 2 4 2 4-2 4-2" /><line x1="9" y1="9" x2="9.01" y2="9" /><line x1="15" y1="9" x2="15.01" y2="9" /></svg>
        </button>
        <div v-if="showEmojiPicker" class="dropdown-panel dropdown-panel-emoji">
          <EditorEmojiPicker @select="handleEmojiSelect" @select-custom="handleCustomEmojiSelect" />
        </div>
      </div>

      <!-- Source mode toggle -->
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

    <!-- Autocomplete dropdown -->
    <EditorAutocompleteDropdown
      v-if="autocomplete.isActive.value"
      :suggestions="autocomplete.suggestions.value"
      :selected-index="autocomplete.selectedIndex.value"
      :position="autocomplete.dropdownPosition.value"
      @select="handleAutocompleteSelect"
    />

    <!-- Upload indicator -->
    <div v-if="uploading" class="upload-indicator">
      <svg class="animate-spin w-4 h-4 text-primary" viewBox="0 0 24 24" fill="none">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
      </svg>
      <span class="text-xs text-gray-500">Uploading image...</span>
    </div>

    <!-- TipTap editor -->
    <EditorContent
      v-show="!isSourceMode"
      :editor="editor"
      class="editor-content"
      :style="{ minHeight: minHeight ?? '150px' }"
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
.rich-editor-container {
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  overflow: hidden;
  background: white;
}

/* Toolbar */
.toolbar {
  display: flex;
  align-items: center;
  gap: 2px;
  padding: 4px 8px;
  background: #f9fafb;
  border-bottom: 1px solid #e5e7eb;
  flex-wrap: wrap;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  padding: 5px;
  border-radius: 4px;
  color: #4b5563;
  cursor: pointer;
  transition: background-color 0.15s, color 0.15s;
  position: relative;
  flex-shrink: 0;
}

.tb-btn svg {
  width: 16px;
  height: 16px;
}

.tb-btn:hover {
  background-color: #e5e7eb;
  color: #111827;
}

.tb-btn.active {
  background-color: #dbeafe;
  color: #2563eb;
}

.tb-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.tb-sep {
  width: 1px;
  height: 20px;
  background: #d1d5db;
  margin: 0 4px;
  flex-shrink: 0;
}

/* Color indicator under text/highlight buttons */
.color-indicator {
  position: absolute;
  bottom: 3px;
  left: 50%;
  transform: translateX(-50%);
  width: 14px;
  height: 3px;
  border-radius: 1px;
}

/* Dropdown panels */
.dropdown-panel {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.12);
  z-index: 50;
}

.dropdown-panel-emoji {
  left: auto;
  right: 0;
}

/* Upload indicator */
.upload-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: #eff6ff;
  border-bottom: 1px solid #dbeafe;
}

/* Editor content area */
.editor-content {
  padding: 0;
}

.editor-content :deep(.tiptap) {
  padding: 16px;
  outline: none;
  min-height: inherit;
}

.editor-content :deep(.tiptap p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: #9ca3af;
  pointer-events: none;
  height: 0;
}

/* Prose styling for editor content */
.editor-content :deep(.tiptap) {
  font-size: 14px;
  line-height: 1.6;
  color: #1f2937;
}

.editor-content :deep(.tiptap p) {
  margin: 0 0 0.5em;
}

.editor-content :deep(.tiptap h1),
.editor-content :deep(.tiptap h2),
.editor-content :deep(.tiptap h3) {
  font-weight: 700;
  margin: 1em 0 0.5em;
  line-height: 1.3;
}

.editor-content :deep(.tiptap h1) { font-size: 1.5em; }
.editor-content :deep(.tiptap h2) { font-size: 1.25em; }
.editor-content :deep(.tiptap h3) { font-size: 1.1em; }

.editor-content :deep(.tiptap ul),
.editor-content :deep(.tiptap ol) {
  padding-left: 1.5em;
  margin: 0.5em 0;
}

.editor-content :deep(.tiptap li) {
  margin: 0.25em 0;
}

.editor-content :deep(.tiptap blockquote) {
  border-left: 4px solid rgb(var(--color-primary, 99 102 241) / 0.3);
  background: #f9fafb;
  border-radius: 0 4px 4px 0;
  padding: 8px 12px;
  margin: 8px 0;
}

/* Forum quote styling within editor */
.editor-content :deep(.tiptap .forum-quote) {
  border-left: 4px solid rgb(var(--color-primary, 99 102 241) / 0.3);
  background: #f9fafb;
  border-radius: 0 6px 6px 0;
  padding: 10px 14px;
  margin: 8px 0;
}

.editor-content :deep(.tiptap .forum-quote-header) {
  font-size: 0.85em;
  color: #6b7280;
  margin-bottom: 6px;
}

.editor-content :deep(.tiptap .forum-quote-header strong) {
  color: rgb(var(--color-primary, 99 102 241));
}

.editor-content :deep(.tiptap .forum-quote-header a) {
  color: rgb(var(--color-primary, 99 102 241));
  text-decoration: none;
}

.editor-content :deep(.tiptap .forum-quote .forum-quote) {
  border-left-color: rgb(var(--color-primary, 99 102 241) / 0.15);
  background: #f3f4f6;
}

.editor-content :deep(.tiptap code) {
  background: #f3f4f6;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.875em;
  font-family: 'Fira Code', 'JetBrains Mono', Menlo, monospace;
  color: #e11d48;
}

.editor-content :deep(.tiptap pre) {
  background: #1f2937;
  color: #e5e7eb;
  padding: 16px;
  border-radius: 8px;
  overflow-x: auto;
  margin: 8px 0;
  font-size: 13px;
  line-height: 1.5;
}

.editor-content :deep(.tiptap pre code) {
  background: none;
  padding: 0;
  color: inherit;
  font-size: inherit;
}

.editor-content :deep(.tiptap img) {
  max-width: 100%;
  border-radius: 6px;
}

.editor-content :deep(.tiptap table) {
  border-collapse: collapse;
  width: 100%;
  margin: 8px 0;
}

.editor-content :deep(.tiptap th),
.editor-content :deep(.tiptap td) {
  border: 1px solid #d1d5db;
  padding: 8px 12px;
  text-align: left;
  min-width: 80px;
}

.editor-content :deep(.tiptap th) {
  background: #f9fafb;
  font-weight: 600;
}

.editor-content :deep(.tiptap hr) {
  border: none;
  border-top: 1px solid #e5e7eb;
  margin: 16px 0;
}

.editor-content :deep(.tiptap a) {
  color: rgb(var(--color-primary, 99 102 241));
  text-decoration: underline;
  cursor: pointer;
}

.editor-content :deep(.tiptap details) {
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  padding: 8px 12px;
  margin: 8px 0;
}

.editor-content :deep(.tiptap summary) {
  cursor: pointer;
  font-weight: 600;
  color: #374151;
}

.editor-content :deep(.tiptap mark) {
  border-radius: 2px;
  padding: 0 2px;
}

/* YouTube iframe */
.editor-content :deep(.tiptap iframe) {
  width: 100%;
  aspect-ratio: 16/9;
  border-radius: 8px;
  margin: 8px 0;
  border: none;
}

/* Highlight.js syntax colors */
.editor-content :deep(.hljs-comment),
.editor-content :deep(.hljs-quote) { color: #6b7280; }
.editor-content :deep(.hljs-keyword),
.editor-content :deep(.hljs-selector-tag) { color: #c084fc; }
.editor-content :deep(.hljs-string),
.editor-content :deep(.hljs-addition) { color: #86efac; }
.editor-content :deep(.hljs-number),
.editor-content :deep(.hljs-literal) { color: #fbbf24; }
.editor-content :deep(.hljs-built_in),
.editor-content :deep(.hljs-type) { color: #67e8f9; }
.editor-content :deep(.hljs-title),
.editor-content :deep(.hljs-section) { color: #93c5fd; }
.editor-content :deep(.hljs-attr),
.editor-content :deep(.hljs-attribute) { color: #fca5a5; }
.editor-content :deep(.hljs-variable),
.editor-content :deep(.hljs-template-variable) { color: #fca5a5; }
.editor-content :deep(.hljs-deletion) { color: #fca5a5; background: rgba(239, 68, 68, 0.15); }
.editor-content :deep(.hljs-meta) { color: #a5b4fc; }
</style>
