<script setup lang="ts">
import type { Comment } from '~/types/generated'
import { timeAgo } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import { useModeration } from '~/composables/useModeration'

type CommentNode = Comment & { children?: CommentNode[] }

const props = defineProps<{
  comment: CommentNode
  postId: string
  postNumber: number
  isModerator?: boolean
}>()

const emit = defineEmits<{
  quote: [author: string, body: string, postNumber: number]
  updated: []
}>()

const authStore = useAuthStore()
const { removeComment, restoreComment } = useModeration()

const showRemoveDialog = ref(false)
const showReportDialog = ref(false)
const removeReason = ref('')
const reportReason = ref('')
const acting = ref(false)

const REPORT_MUTATION = `
  mutation ReportComment($commentId: ID!, $reason: String!) {
    reportComment(commentId: $commentId, reason: $reason) { success }
  }
`

const PIN_MUTATION = `
  mutation PinComment($commentId: ID!) {
    pinComment(commentId: $commentId) { id isPinned }
  }
`

const canModerate = computed(() => props.isModerator || authStore.isAdmin)
const isOwnComment = computed(() => authStore.user?.id === props.comment.creator?.id)

const displayName = computed(() => {
  if (!props.comment.creator) return '[deleted]'
  return props.comment.creator.displayName ?? props.comment.creator.name
})

const avatarInitial = computed(() => {
  return displayName.value.charAt(0).toUpperCase()
})

function handleQuote (): void {
  const body = props.comment.body ?? ''
  emit('quote', displayName.value, body, props.postNumber)
}

async function handleRemove (): Promise<void> {
  acting.value = true
  await removeComment(props.comment.id, removeReason.value || undefined)
  showRemoveDialog.value = false
  removeReason.value = ''
  acting.value = false
  emit('updated')
}

async function handleRestore (): Promise<void> {
  acting.value = true
  await restoreComment(props.comment.id)
  acting.value = false
  emit('updated')
}

async function submitReport (): Promise<void> {
  if (!reportReason.value.trim()) return
  const { execute } = useGraphQL()
  await execute(REPORT_MUTATION, { variables: { commentId: props.comment.id, reason: reportReason.value } })
  showReportDialog.value = false
  reportReason.value = ''
}

async function togglePin (): Promise<void> {
  acting.value = true
  const { execute } = useGraphQL()
  await execute(PIN_MUTATION, { variables: { commentId: props.comment.id } })
  acting.value = false
  emit('updated')
}
</script>

<template>
  <article :id="`post-${postNumber}`" class="bg-white border border-gray-200 rounded-lg overflow-hidden">
    <!-- Post header bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-gray-50 border-b border-gray-200">
      <div class="flex items-center gap-3">
        <!-- Avatar -->
        <div v-if="comment.creator?.avatar" class="w-8 h-8 rounded-full overflow-hidden flex-shrink-0">
          <img :src="comment.creator.avatar" :alt="displayName" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-8 h-8 rounded-full bg-primary/10 text-primary flex items-center justify-center text-sm font-semibold flex-shrink-0">
          {{ avatarInitial }}
        </div>

        <div class="flex flex-col">
          <NuxtLink
            v-if="comment.creator"
            :to="`/@${comment.creator.name}`"
            class="text-sm font-semibold text-gray-900 no-underline hover:text-primary transition-colors"
          >
            {{ displayName }}
          </NuxtLink>
          <span v-else class="text-sm text-gray-400 italic">[deleted]</span>
          <time :datetime="comment.createdAt" class="text-xs text-gray-400">
            {{ timeAgo(comment.createdAt) }}
          </time>
        </div>

        <span v-if="comment.isPinned" class="inline-flex items-center gap-1 rounded-full bg-amber-100 px-2 py-0.5 text-xs font-medium text-amber-800">
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path d="M5.5 16.5a.75.75 0 01-.75-.75v-5.5a.75.75 0 011.5 0v5.5a.75.75 0 01-.75.75z" />
          </svg>
          Pinned
        </span>
      </div>

      <!-- Post number -->
      <a :href="`#post-${postNumber}`" class="text-xs text-gray-400 hover:text-primary no-underline font-mono">
        #{{ postNumber }}
      </a>
    </div>

    <!-- Post body -->
    <div class="px-4 py-3">
      <div v-if="comment.isDeleted" class="text-sm text-gray-400 italic py-4">
        This post has been deleted.
      </div>
      <div v-else-if="comment.isRemoved" class="text-sm text-red-400 italic py-4">
        This post has been removed by a moderator.
      </div>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-else-if="comment.bodyHTML" class="prose prose-sm max-w-none" v-html="sanitizeHtml(comment.bodyHTML)" />
      <div v-else class="prose prose-sm max-w-none whitespace-pre-wrap">
        {{ comment.body }}
      </div>
    </div>

    <!-- Post footer -->
    <div class="px-4 py-2 bg-gray-50 border-t border-gray-100 flex items-center justify-between flex-wrap gap-2">
      <!-- Left: reactions -->
      <CommonReactionBar target-type="comment" :target-id="comment.id" />

      <!-- Right: actions -->
      <div class="flex items-center gap-1 text-xs text-gray-500">
        <!-- Quote -->
        <button
          v-if="authStore.isLoggedIn"
          class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-200 hover:text-gray-700 transition-colors"
          @click="handleQuote"
          title="Quote this post"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
          </svg>
          Quote
        </button>

        <!-- Report -->
        <button
          v-if="authStore.isLoggedIn && !isOwnComment"
          class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
          @click="showReportDialog = true"
        >
          Report
        </button>

        <!-- Mod actions -->
        <template v-if="canModerate">
          <div class="w-px h-4 bg-gray-300 mx-1" />
          <button
            class="px-2 py-1 rounded hover:bg-amber-50 hover:text-amber-600 transition-colors"
            :class="{ 'text-amber-500': comment.isPinned }"
            :disabled="acting"
            @click="togglePin"
          >
            {{ comment.isPinned ? 'Unpin' : 'Pin' }}
          </button>
          <button
            v-if="!comment.isRemoved"
            class="px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
            :disabled="acting"
            @click="showRemoveDialog = true"
          >
            Remove
          </button>
          <button
            v-else
            class="px-2 py-1 rounded text-red-400 hover:bg-green-50 hover:text-green-600 transition-colors"
            :disabled="acting"
            @click="handleRestore"
          >
            Restore
          </button>
        </template>
      </div>
    </div>

    <!-- Remove dialog -->
    <CommonModal v-if="showRemoveDialog" @close="showRemoveDialog = false">
      <template #title>Remove Post</template>
      <template #default>
        <div class="space-y-3">
          <p class="text-sm text-gray-600">Remove this post?</p>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason (optional)</label>
            <input v-model="removeReason" type="text" class="form-input" placeholder="Reason..." />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showRemoveDialog = false">Cancel</button>
            <button class="button button-sm bg-red-600 text-white hover:bg-red-700" :disabled="acting" @click="handleRemove">Remove</button>
          </div>
        </div>
      </template>
    </CommonModal>

    <!-- Report dialog -->
    <CommonModal v-if="showReportDialog" @close="showReportDialog = false">
      <template #title>Report Post</template>
      <template #default>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
            <textarea v-model="reportReason" class="form-input" rows="3" placeholder="Why are you reporting this post?" />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showReportDialog = false">Cancel</button>
            <button class="button primary button-sm" @click="submitReport">Submit Report</button>
          </div>
        </div>
      </template>
    </CommonModal>
  </article>
</template>
