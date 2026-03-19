<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useComments } from '~/composables/useComments'
import { useAuthStore } from '~/stores/auth'
import type { Post } from '~/types/generated'

const route = useRoute()
const postId = route.params.id as string
const boardName = route.params.board as string
const authStore = useAuthStore()

// Check if current user is a moderator of this board
const isModerator = ref(false)

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
      board { id name title icon }
      creator { id name displayName avatar }
    }
  }
`

interface PostResponse {
  post: Post
}

const { execute, loading: postLoading, error: postError } = useGraphQL<PostResponse>()
const post = ref<Post | null>(null)

async function fetchPost (): Promise<void> {
  const result = await execute(POST_QUERY, { variables: { id: postId } })
  if (result?.post) {
    post.value = result.post
    const p = result.post
    const desc = p.body ? p.body.substring(0, 160).replace(/\n/g, ' ') : `Post in +${boardName}`
    useHead({
      title: p.title,
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
    useSeoMeta({
      title: p.title,
      ogTitle: p.title,
      description: desc,
      ogDescription: desc,
      ogType: 'article',
      ogImage: p.image || p.thumbnailUrl || undefined,
      articlePublishedTime: p.createdAt,
      articleModifiedTime: p.updatedAt,
      articleSection: boardName,
    })
  }
}

await fetchPost()

// Redirect threads to the correct URL
if (post.value?.isThread) {
  const slug = post.value.slug ?? ''
  const slugPart = slug ? `/${slug}` : ''
  await navigateTo(`/b/${boardName}/threads/${postId}${slugPart}`, { replace: true })
}

// Check mod permissions if logged in
if (authStore.isLoggedIn && post.value?.board?.id) {
  const { execute: execModCheck } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const modResult = await execModCheck(MOD_CHECK_QUERY, { variables: { boardId: post.value.board.id } })
  if (modResult?.getBoardSettings?.moderatorPermissions != null) {
    isModerator.value = true
  }
}

const { comments, loading: commentsLoading, error: commentsError, fetchComments, submitComment } = useComments()
await fetchComments(postId)

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
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="postLoading && !post" size="lg" />
    <CommonErrorDisplay v-else-if="postError" :message="postError.message" @retry="fetchPost" />

    <template v-else-if="post">
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
  </div>
</template>
