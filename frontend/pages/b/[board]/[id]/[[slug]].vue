<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useComments } from '~/composables/useComments'
import { useAuthStore } from '~/stores/auth'
import type { Post, Comment } from '~/types/generated'

const route = useRoute()
const postId = route.params.id as string
const boardName = route.params.board as string
const authStore = useAuthStore()

const isModerator = ref(false)
const commentFormRef = ref<InstanceType<typeof import('~/components/thread/ThreadCommentForm.vue').default> | null>(null)

const MOD_CHECK_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) { moderatorPermissions }
  }
`

const POST_QUERY = `
  query GetPost($id: ID!) {
    post(id: $id) {
      id
      title
      body
      bodyHTML
      postType
      isThread
      url
      image
      altText
      embedTitle
      embedDescription
      embedVideoUrl
      createdAt
      updatedAt
      isDeleted
      isRemoved
      isLocked
      isFeaturedBoard
      isFeaturedLocal
      isNSFW
      slug
      score
      upvotes
      downvotes
      commentCount
      myVote
      isSaved
      thumbnailUrl
      distinguishedAs
      board { id name title icon }
      creator { id name displayName avatar isAdmin }
    }
  }
`

interface PostResponse {
  post: Post
}

const { execute, loading: postLoading, error: postError } = useGraphQL<PostResponse>()
const { execute: execModCheck } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
const post = ref<Post | null>(null)

async function fetchPost (): Promise<void> {
  const result = await execute(POST_QUERY, { variables: { id: postId } })
  if (result?.post) {
    post.value = result.post
  }
}

const isThread = computed(() => post.value?.isThread ?? false)

useHead({ title: computed(() => post.value?.title ?? 'Post') })
useSeoMeta({
  title: computed(() => post.value?.title ?? 'Post'),
  ogTitle: computed(() => post.value?.title ?? 'Post'),
  description: computed(() => {
    if (!post.value?.body) return `Post in +${boardName}`
    return post.value.body.substring(0, 160).replace(/\n/g, ' ')
  }),
  ogType: 'article',
  ogImage: computed(() => post.value?.image || post.value?.thumbnailUrl || undefined),
})

await fetchPost()

// JSON-LD structured data
if (post.value) {
  const p = post.value
  useHead({
    script: [{
      type: 'application/ld+json',
      innerHTML: JSON.stringify({
        '@context': 'https://schema.org',
        '@type': 'DiscussionForumPosting',
        headline: p.title,
        text: p.body?.substring(0, 500),
        datePublished: p.createdAt,
        dateModified: p.updatedAt,
        author: p.creator ? {
          '@type': 'Person',
          name: p.creator.displayName || p.creator.name,
        } : undefined,
        interactionStatistic: [
          { '@type': 'InteractionCounter', interactionType: 'https://schema.org/CommentAction', userInteractionCount: p.commentCount },
          { '@type': 'InteractionCounter', interactionType: 'https://schema.org/LikeAction', userInteractionCount: p.upvotes },
        ],
      }),
    }],
  })
}

// Check mod permissions
if (authStore.isLoggedIn && post.value?.board?.id) {
  const modResult = await execModCheck(MOD_CHECK_QUERY, { variables: { boardId: post.value.board.id } })
  if (modResult?.getBoardSettings?.moderatorPermissions != null) {
    isModerator.value = true
  }
}

// Comments
const { comments, loading: commentsLoading, error: commentsError, fetchComments, submitComment } = useComments()
await fetchComments(postId)

// --- Feed post handlers ---
async function handleReply (parentId: string, body: string): Promise<void> {
  const newComment = await submitComment(postId, body, parentId)
  if (newComment) {
    await fetchComments(postId)
  }
}

async function handleTopLevelComment (body: string): Promise<void> {
  const newComment = await submitComment(postId, body)
  if (newComment) {
    await fetchComments(postId)
  }
}

// --- Thread/forum post handlers ---
const linearComments = computed(() => {
  return [...comments.value].sort((a, b) => {
    return new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
  })
})

function handleQuoteFromComment (author: string, body: string, postNumber: number): void {
  commentFormRef.value?.addQuote(author, body, postNumber)
  nextTick(() => {
    const formEl = document.getElementById('thread-reply-form')
    if (formEl) {
      formEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  })
}

function handleQuoteOP (author: string, body: string): void {
  commentFormRef.value?.addQuote(author, body, 1)
  nextTick(() => {
    const formEl = document.getElementById('thread-reply-form')
    if (formEl) {
      formEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
    }
  })
}
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="postLoading && !post" size="lg" />
    <CommonErrorDisplay v-else-if="postError" :message="postError.message" @retry="fetchPost" />

    <template v-else-if="post">
      <!-- Thread/forum-style view -->
      <template v-if="isThread">
        <div class="mb-3 flex items-center gap-2 text-xs text-gray-500">
          <NuxtLink
            v-if="post.board"
            :to="`/b/${post.board.name}`"
            class="no-underline hover:text-primary text-gray-500"
          >
            b/{{ post.board.name }}
          </NuxtLink>
          <span class="text-gray-300">/</span>
          <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-primary/10 text-primary text-xs font-medium">
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
            </svg>
            Thread
          </span>
          <span class="text-gray-300">/</span>
          <span class="text-gray-700 font-medium truncate">{{ post.title }}</span>
        </div>

        <ThreadDetailHeader
          :post="post"
          :is-moderator="isModerator"
          @post-updated="fetchPost"
          @quote="handleQuoteOP"
        />

        <div class="mt-6 mb-3 flex items-center justify-between">
          <h2 class="text-sm font-semibold text-gray-700 flex items-center gap-2">
            <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            {{ post.commentCount }} Repl{{ post.commentCount === 1 ? 'y' : 'ies' }}
          </h2>
        </div>

        <div class="space-y-3">
          <CommonLoadingSpinner v-if="commentsLoading && comments.length === 0" />
          <CommonErrorDisplay v-if="commentsError" :message="commentsError.message" @retry="fetchComments(postId)" />

          <div v-if="!commentsLoading && comments.length === 0" class="bg-white border border-gray-200 rounded-lg py-8 text-center text-sm text-gray-400">
            No replies yet. Be the first to reply.
          </div>

          <ThreadPost
            v-for="(comment, index) in linearComments"
            :key="comment.id"
            :comment="comment"
            :post-id="postId"
            :post-number="index + 2"
            :is-moderator="isModerator"
            :board-id="post.board?.id"
            @quote="handleQuoteFromComment"
          />
        </div>

        <div id="thread-reply-form" class="mt-6">
          <ThreadCommentForm
            ref="commentFormRef"
            @submit="handleTopLevelComment"
          />
        </div>
      </template>

      <!-- Feed-style view -->
      <template v-else>
        <PostDetail :post="post" :is-moderator="isModerator" @post-updated="fetchPost" />

        <div class="mt-4">
          <CommentForm @submit="handleTopLevelComment" />
        </div>

        <div class="mt-6">
          <CommonErrorDisplay v-if="commentsError" :message="commentsError.message" @retry="fetchComments(postId)" />
          <CommentTree
            :comments="comments"
            :post-id="postId"
            :loading="commentsLoading"
            :is-moderator="isModerator"
            @reply="handleReply"
          />
        </div>
      </template>
    </template>
  </div>
</template>
