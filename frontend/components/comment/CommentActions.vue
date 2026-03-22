<script setup lang="ts">
import { ref } from 'vue'
import type { Comment } from '~/types/generated'
import { useGraphQL } from '~/composables/useGraphQL'
import { useModeration } from '~/composables/useModeration'
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import { useToast } from '~/composables/useToast'

const props = defineProps<{
  comment: Comment
  isModerator?: boolean
}>()

const emit = defineEmits<{
  'toggle-reply': []
  updated: []
}>()

const authStore = useAuthStore()
const siteStore = useSiteStore()
const toast = useToast()
const { removeComment, restoreComment } = useModeration()

// Local reactive state so vote updates are immediately visible
const localScore = ref(props.comment.score)
const localMyVote = ref(props.comment.myVote ?? 0)

watch(() => props.comment.score, (v) => { localScore.value = v })
watch(() => props.comment.myVote, (v) => { localMyVote.value = v ?? 0 })

const showRemoveDialog = ref(false)
const showReportDialog = ref(false)
const showModMenu = ref(false)
const removeReason = ref('')
const reportReason = ref('')
const acting = ref(false)

const VOTE_MUTATION = `
  mutation VoteOnComment($commentId: ID!, $direction: Int!) {
    voteOnComment(commentId: $commentId, direction: $direction) {
      id score upvotes downvotes myVote
    }
  }
`

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

const SAVE_COMMENT_MUTATION = `
  mutation SaveComment($commentId: ID!) { saveComment(commentId: $commentId) { id isSaved } }
`
const UNSAVE_COMMENT_MUTATION = `
  mutation UnsaveComment($commentId: ID!) { unsaveComment(commentId: $commentId) { id isSaved } }
`

interface VoteResponse {
  voteOnComment: { id: string; score: number; upvotes: number; downvotes: number; myVote: number }
}

const canModerate = computed(() => props.isModerator || authStore.isAdmin)
const isOwnComment = computed(() => authStore.user?.id === props.comment.creator?.id)
const commentSaved = ref(props.comment.isSaved ?? false)

async function vote (score: number): Promise<void> {
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }

  const { execute } = useGraphQL<VoteResponse>()
  const newScore = localMyVote.value === score ? 0 : score

  // Optimistic update
  localMyVote.value = newScore
  localScore.value = props.comment.score + newScore - (props.comment.myVote ?? 0)

  const result = await execute(VOTE_MUTATION, {
    variables: { commentId: props.comment.id, direction: newScore },
  })

  if (result?.voteOnComment) {
    localScore.value = result.voteOnComment.score
    localMyVote.value = result.voteOnComment.myVote
  }
}

async function toggleSaveComment (): Promise<void> {
  const { execute } = useGraphQL()
  const mutation = commentSaved.value ? UNSAVE_COMMENT_MUTATION : SAVE_COMMENT_MUTATION
  const result = await execute(mutation, { variables: { commentId: props.comment.id } })
  if (result) {
    commentSaved.value = !commentSaved.value
    toast.success(commentSaved.value ? 'Comment saved' : 'Comment unsaved')
  }
}

async function handleRemove (): Promise<void> {
  acting.value = true
  const success = await removeComment(props.comment.id, removeReason.value || undefined)
  showRemoveDialog.value = false
  removeReason.value = ''
  acting.value = false
  if (success) emit('updated')
}

async function handleRestore (): Promise<void> {
  acting.value = true
  showModMenu.value = false
  const success = await restoreComment(props.comment.id)
  acting.value = false
  if (success) emit('updated')
}

async function submitReport (): Promise<void> {
  if (!reportReason.value.trim()) return
  const { execute } = useGraphQL()
  const result = await execute(REPORT_MUTATION, { variables: { commentId: props.comment.id, reason: reportReason.value } })
  showReportDialog.value = false
  reportReason.value = ''
  if (result) toast.success('Report submitted')
}

async function togglePin (): Promise<void> {
  acting.value = true
  showModMenu.value = false
  const { execute } = useGraphQL()
  const result = await execute(PIN_MUTATION, { variables: { commentId: props.comment.id } })
  acting.value = false
  if (result) {
    toast.success(props.comment.isPinned ? 'Comment unpinned' : 'Comment pinned')
    emit('updated')
  }
}

function openRemoveDialog (): void {
  showModMenu.value = false
  showRemoveDialog.value = true
}
</script>

<template>
  <div class="flex items-center gap-1 mt-1.5 text-xs text-gray-400 flex-wrap">
    <!-- Upvote -->
    <button
      class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded hover:bg-primary/10 transition-colors"
      :class="localMyVote === 1 ? 'text-primary' : 'hover:text-primary'"
      @click="vote(1)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
      </svg>
    </button>

    <!-- Score -->
    <span
      class="font-semibold text-xs min-w-[2ch] text-center tabular-nums select-none"
      :class="localMyVote === 1 ? 'text-primary' : localMyVote === -1 ? 'text-secondary' : 'text-gray-500'"
    >
      {{ localScore }}
    </span>

    <!-- Downvote -->
    <button
      v-if="siteStore.enableDownvotes"
      class="inline-flex items-center gap-0.5 px-1.5 py-0.5 rounded hover:bg-secondary/10 transition-colors"
      :class="localMyVote === -1 ? 'text-secondary' : 'hover:text-secondary'"
      @click="vote(-1)"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>

    <span class="text-gray-200 mx-0.5">|</span>

    <!-- Reply -->
    <button
      v-if="authStore.isLoggedIn"
      class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded hover:bg-gray-100 hover:text-gray-600 transition-colors"
      @click="emit('toggle-reply')"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
      </svg>
      Reply
    </button>

    <!-- Save -->
    <button
      v-if="authStore.isLoggedIn"
      class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
      :class="commentSaved ? 'text-primary hover:bg-primary/10' : 'hover:bg-gray-100 hover:text-gray-600'"
      @click="toggleSaveComment"
    >
      <svg class="w-3.5 h-3.5" :fill="commentSaved ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
      </svg>
      {{ commentSaved ? 'Saved' : 'Save' }}
    </button>

    <!-- Report (non-own comments) -->
    <button
      v-if="authStore.isLoggedIn && !isOwnComment"
      class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded hover:bg-gray-100 hover:text-red-500 transition-colors"
      @click="showReportDialog = true"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 21v-4m0 0V5a2 2 0 012-2h6.5l1 1H21l-3 6 3 6h-8.5l-1-1H5a2 2 0 00-2 2zm9-13.5V9" />
      </svg>
      Report
    </button>

    <!-- Mod actions menu -->
    <template v-if="canModerate">
      <span class="text-gray-200 mx-0.5">|</span>
      <div class="relative">
        <button
          class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded transition-colors"
          :class="showModMenu ? 'bg-gray-100 text-gray-700' : 'hover:bg-gray-100 hover:text-gray-600'"
          @click="showModMenu = !showModMenu"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          Mod
        </button>

        <!-- Dropdown -->
        <Teleport to="body">
          <div
            v-if="showModMenu"
            class="fixed inset-0 z-40"
            @click="showModMenu = false"
          />
        </Teleport>
        <div
          v-if="showModMenu"
          class="absolute left-0 top-full mt-1 w-44 bg-white rounded-lg border border-gray-200 shadow-lg z-50 py-1"
        >
          <div class="px-3 py-1.5 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
            Moderation
          </div>

          <!-- Pin/Unpin -->
          <button
            class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
            :class="comment.isPinned ? 'text-amber-700 hover:bg-amber-50' : 'text-gray-700 hover:bg-gray-50'"
            :disabled="acting"
            @click="togglePin"
          >
            <svg class="w-4 h-4" :class="comment.isPinned ? 'text-amber-500' : 'text-gray-400'" fill="currentColor" viewBox="0 0 20 20">
              <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            </svg>
            {{ comment.isPinned ? 'Unpin' : 'Pin' }}
          </button>

          <div class="border-t border-gray-100 my-1" />

          <!-- Remove/Restore -->
          <button
            v-if="!comment.isRemoved"
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
            :disabled="acting"
            @click="openRemoveDialog"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
            </svg>
            Remove
          </button>
          <button
            v-else
            class="w-full flex items-center gap-2 px-3 py-2 text-sm text-green-600 hover:bg-green-50 transition-colors"
            :disabled="acting"
            @click="handleRestore"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            Restore
          </button>
        </div>
      </div>
    </template>

    <!-- Remove dialog -->
    <CommonModal v-if="showRemoveDialog" @close="showRemoveDialog = false">
      <template #title>Remove Comment</template>
      <template #default>
        <div class="space-y-3">
          <p class="text-sm text-gray-600">Remove this comment?</p>
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
      <template #title>Report Comment</template>
      <template #default>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
            <textarea v-model="reportReason" class="form-input" rows="3" placeholder="Why are you reporting this comment?" />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showReportDialog = false">Cancel</button>
            <button class="button primary button-sm" @click="submitReport">Submit Report</button>
          </div>
        </div>
      </template>
    </CommonModal>
  </div>
</template>
