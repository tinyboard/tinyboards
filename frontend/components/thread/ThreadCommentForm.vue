<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '~/stores/auth'

interface QuotedPost {
  author: string
  body: string
  postNumber: number
}

defineProps<{
  placeholder?: string
}>()

const emit = defineEmits<{
  submit: [body: string]
}>()

const authStore = useAuthStore()
const body = ref('')
const editorRef = ref<InstanceType<typeof import('~/components/editor/RichTextEditor.vue').default> | null>(null)
const quotedPosts = ref<QuotedPost[]>([])

function addQuote (author: string, text: string, postNumber: number): void {
  // Check if this post is already quoted
  const existing = quotedPosts.value.findIndex(q => q.postNumber === postNumber)
  if (existing >= 0) {
    quotedPosts.value.splice(existing, 1)
    return
  }

  quotedPosts.value.push({ author, body: text, postNumber })
}

function removeQuote (index: number): void {
  quotedPosts.value.splice(index, 1)
}

function insertQuotes (): void {
  if (quotedPosts.value.length === 0) return

  for (const q of quotedPosts.value) {
    const cleanBody = q.body.replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/\n/g, '<br>')
    editorRef.value?.insertQuoteBlock(q.author, cleanBody, q.postNumber)
  }
  quotedPosts.value = []
}

function handleSubmit (): void {
  // Auto-insert any pending quotes before submitting
  if (quotedPosts.value.length > 0) {
    insertQuotes()
  }

  // Check if content is empty (just whitespace or empty tags)
  const stripped = body.value.replace(/<[^>]*>/g, '').trim()
  if (!stripped) return

  emit('submit', body.value)
  body.value = ''
  editorRef.value?.clearContent()
}

// Expose addQuote so parent can call it
defineExpose({ addQuote })
</script>

<template>
  <div v-if="authStore.isLoggedIn" class="bg-white border border-gray-200 rounded-lg overflow-hidden">
    <!-- Header -->
    <div class="px-4 py-2 bg-gray-50 border-b border-gray-200 flex items-center justify-between">
      <h3 class="text-sm font-semibold text-gray-700">Post a Reply</h3>
    </div>

    <!-- Queued quotes indicator -->
    <div v-if="quotedPosts.length > 0" class="px-4 py-2 bg-blue-50 border-b border-blue-100">
      <div class="flex items-center justify-between mb-1">
        <span class="text-xs font-medium text-blue-700">
          {{ quotedPosts.length }} post{{ quotedPosts.length > 1 ? 's' : '' }} quoted
        </span>
        <div class="flex gap-2">
          <button
            class="text-xs text-blue-600 hover:text-blue-800 font-medium"
            @click="insertQuotes"
          >
            Insert All
          </button>
          <button
            class="text-xs text-gray-400 hover:text-gray-600"
            @click="quotedPosts = []"
          >
            Clear
          </button>
        </div>
      </div>
      <div class="flex flex-wrap gap-1">
        <span
          v-for="(q, i) in quotedPosts"
          :key="q.postNumber"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-blue-100 text-blue-700 text-xs"
        >
          @{{ q.author }} #{{ q.postNumber }}
          <button class="hover:text-blue-900 ml-0.5" @click="removeQuote(i)">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </span>
      </div>
    </div>

    <!-- Rich text editor -->
    <EditorRichTextEditor
      ref="editorRef"
      v-model="body"
      :placeholder="placeholder ?? 'Write your reply...'"
      min-height="150px"
    />

    <!-- Submit bar -->
    <div class="px-4 py-2 bg-gray-50 border-t border-gray-200 flex items-center justify-end">
      <button
        class="button primary button-sm"
        @click="handleSubmit"
      >
        Post Reply
      </button>
    </div>
  </div>

  <div v-else class="bg-white border border-gray-200 rounded-lg p-4 text-center">
    <p class="text-sm text-gray-500">
      <NuxtLink to="/login" class="text-primary font-medium">
        Log in
      </NuxtLink> to reply to this thread.
    </p>
  </div>
</template>
