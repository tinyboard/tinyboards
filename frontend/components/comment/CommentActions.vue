<script setup lang="ts">
import { ref } from 'vue'
import type { Comment } from '~/types/generated'
import { useGraphQL } from '~/composables/useGraphQL'
import { useModeration } from '~/composables/useModeration'
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'

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
const { removeComment, restoreComment } = useModeration()

// Local reactive state so vote updates are immediately visible
const localScore = ref(props.comment.score)
const localMyVote = ref(props.comment.myVote ?? 0)

watch(() => props.comment.score, (v) => { localScore.value = v })
watch(() => props.comment.myVote, (v) => { localMyVote.value = v ?? 0 })

const showRemoveDialog = ref(false)
const showReportDialog = ref(false)
const removeReason = ref('')
const reportReason = ref('')
const acting = ref(false)

const VOTE_MUTATION = `
  mutation VoteOnComment($commentId: ID!, $direction: Int!) {
    voteOnComment(commentId: $commentId, direction: $direction) {
      id
      score
      upvotes
      downvotes
      myVote
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

interface VoteResponse {
  voteOnComment: { id: string; score: number; upvotes: number; downvotes: number; myVote: number }
}

const canModerate = computed(() => props.isModerator || authStore.isAdmin)
const isOwnComment = computed(() => authStore.user?.id === props.comment.creator?.id)

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
  const success = await restoreComment(props.comment.id)
  acting.value = false
  if (success) emit('updated')
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
  <div class="flex items-center gap-2 mt-1 text-xs text-gray-400 flex-wrap">
    <button
      class="hover:text-gray-600"
      :class="{ 'text-primary': localMyVote === 1 }"
      @click="vote(1)"
    >
      upvote
    </button>
    <span class="font-medium" :class="localMyVote === 1 ? 'text-primary' : localMyVote === -1 ? 'text-secondary' : 'text-gray-500'">
      {{ localScore }}
    </span>
    <button
      v-if="siteStore.enableDownvotes"
      class="hover:text-gray-600"
      :class="{ 'text-secondary': localMyVote === -1 }"
      @click="vote(-1)"
    >
      downvote
    </button>
    <button
      v-if="authStore.isLoggedIn"
      class="hover:text-gray-600"
      @click="emit('toggle-reply')"
    >
      reply
    </button>

    <!-- Report (non-own comments) -->
    <button
      v-if="authStore.isLoggedIn && !isOwnComment"
      class="hover:text-red-500"
      @click="showReportDialog = true"
    >
      report
    </button>

    <!-- Mod actions -->
    <template v-if="canModerate">
      <span class="text-gray-200">|</span>
      <button
        class="hover:text-amber-500"
        :class="{ 'text-amber-500': comment.isPinned }"
        :disabled="acting"
        @click="togglePin"
      >
        {{ comment.isPinned ? 'unpin' : 'pin' }}
      </button>
      <button
        v-if="!comment.isRemoved"
        class="hover:text-red-500"
        :disabled="acting"
        @click="showRemoveDialog = true"
      >
        remove
      </button>
      <button
        v-else
        class="text-red-400 hover:text-green-500"
        :disabled="acting"
        @click="handleRestore"
      >
        restore
      </button>
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
