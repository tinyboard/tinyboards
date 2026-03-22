<script setup lang="ts">
import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import type { Post } from '~/types/generated'

const props = defineProps<{
  post: Post
  layout?: 'vertical' | 'horizontal'
}>()

const authStore = useAuthStore()
const siteStore = useSiteStore()

// Local reactive state so vote updates are immediately visible
const localScore = ref(props.post.score)
const localMyVote = ref(props.post.myVote ?? 0)
const scorePop = ref(false)

// Keep in sync if parent re-renders with new post data
watch(() => props.post.score, (v) => { localScore.value = v })
watch(() => props.post.myVote, (v) => { localMyVote.value = v ?? 0 })

const VOTE_MUTATION = `
  mutation VoteOnPost($postId: ID!, $direction: Int!) {
    voteOnPost(postId: $postId, direction: $direction) {
      id
      score
      upvotes
      downvotes
      myVote
    }
  }
`

interface VoteResponse {
  voteOnPost: { id: string; score: number; upvotes: number; downvotes: number; myVote: number }
}

async function vote (score: number): Promise<void> {
  if (!authStore.isLoggedIn) {
    navigateTo('/login')
    return
  }

  const { execute } = useGraphQL<VoteResponse>()
  // Toggle vote if already at that score
  const newScore = localMyVote.value === score ? 0 : score

  // Optimistic update
  localMyVote.value = newScore
  localScore.value = props.post.score + newScore - (props.post.myVote ?? 0)

  // Trigger score pop animation
  scorePop.value = false
  nextTick(() => { scorePop.value = true })

  const result = await execute(VOTE_MUTATION, {
    variables: { postId: props.post.id, direction: newScore },
  })

  // Reconcile with server response
  if (result?.voteOnPost) {
    localScore.value = result.voteOnPost.score
    localMyVote.value = result.voteOnPost.myVote
  }
}
</script>

<template>
  <div
    class="flex items-center gap-0.5 text-sm"
    :class="layout === 'vertical' ? 'flex-col' : 'flex-row'"
  >
    <button
      class="upvote p-1.5 rounded-md hover:bg-primary/10 flex items-center justify-center transition-colors"
      :class="localMyVote === 1 ? 'upvoted text-primary' : 'text-gray-400 hover:text-primary'"
      aria-label="Upvote"
      @click="vote(1)"
    >
      <svg class="w-5 h-5 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7" />
      </svg>
    </button>

    <span
      class="font-semibold text-sm min-w-[2ch] text-center tabular-nums select-none transition-colors"
      :class="[
        localMyVote === 1 ? 'text-primary' : localMyVote === -1 ? 'text-secondary' : 'text-gray-700',
        scorePop ? 'score-pop' : ''
      ]"
      @animationend="scorePop = false"
    >
      {{ localScore }}
    </span>

    <button
      v-if="siteStore.enableDownvotes"
      class="downvote p-1.5 rounded-md hover:bg-secondary/10 flex items-center justify-center transition-colors"
      :class="localMyVote === -1 ? 'downvoted text-secondary' : 'text-gray-400 hover:text-secondary'"
      aria-label="Downvote"
      @click="vote(-1)"
    >
      <svg class="w-5 h-5 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>
  </div>
</template>
