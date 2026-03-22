<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post, Comment } from '~/types/generated'

const route = useRoute()
const username = computed(() => route.params.username as string)

const RECENT_POSTS_QUERY = `
  query RecentPosts($userName: String, $limit: Int) {
    listPosts(userName: $userName, limit: $limit) {
      id
      title
      body
      url
      createdAt
      slug
      score
      commentCount
      myVote
      board { id name title icon }
      creator { id name displayName avatar }
    }
  }
`

const RECENT_COMMENTS_QUERY = `
  query RecentComments($userName: String, $limit: Int) {
    comments(userName: $userName, limit: $limit) {
      id
      body
      createdAt
      score
      upvotes
      downvotes
      replyCount
      postId
      boardId
      creator { id name displayName avatar }
    }
  }
`

interface RecentPostsResponse { listPosts: Post[] }
interface RecentCommentsResponse { comments: Comment[] }

const { execute: execPosts, loading: postsLoading } = useGraphQL<RecentPostsResponse>()
const { execute: execComments, loading: commentsLoading } = useGraphQL<RecentCommentsResponse>()
const recentPosts = ref<Post[]>([])
const recentComments = ref<Comment[]>([])

const activeTab = ref<'overview' | 'posts' | 'comments'>('overview')

async function loadContent (name: string): Promise<void> {
  const [postsResult, commentsResult] = await Promise.all([
    execPosts(RECENT_POSTS_QUERY, { variables: { userName: name, limit: 10 } }),
    execComments(RECENT_COMMENTS_QUERY, { variables: { userName: name, limit: 10 } }),
  ])
  recentPosts.value = postsResult?.listPosts ?? []
  recentComments.value = commentsResult?.comments ?? []
}

watch(username, (name) => { loadContent(name) })
await loadContent(username.value)
</script>

<template>
  <div>
    <!-- Content type filter for overview -->
    <div class="flex gap-1 mb-4">
      <button
        class="px-3 py-1.5 text-xs font-medium rounded-full transition-colors"
        :class="activeTab === 'overview' ? 'bg-primary text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'"
        @click="activeTab = 'overview'"
      >
        All
      </button>
      <button
        class="px-3 py-1.5 text-xs font-medium rounded-full transition-colors"
        :class="activeTab === 'posts' ? 'bg-primary text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'"
        @click="activeTab = 'posts'"
      >
        Posts
      </button>
      <button
        class="px-3 py-1.5 text-xs font-medium rounded-full transition-colors"
        :class="activeTab === 'comments' ? 'bg-primary text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'"
        @click="activeTab = 'comments'"
      >
        Comments
      </button>
    </div>

    <!-- Posts section -->
    <template v-if="activeTab === 'overview' || activeTab === 'posts'">
      <div v-if="activeTab === 'overview'" class="bg-white rounded-lg border border-gray-200 px-4 py-2.5 mb-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold text-gray-700">Recent Posts</h3>
          <NuxtLink :to="`/@${username}/posts`" class="text-xs text-primary hover:text-primary-hover no-underline">
            View all
          </NuxtLink>
        </div>
      </div>
      <PostList :posts="recentPosts" :loading="postsLoading" />
      <div v-if="!postsLoading && recentPosts.length === 0" class="bg-white rounded-lg border border-gray-200 py-8 text-center mb-4">
        <p class="text-sm text-gray-500">No posts yet.</p>
      </div>
    </template>

    <!-- Comments section -->
    <template v-if="activeTab === 'overview' || activeTab === 'comments'">
      <div v-if="activeTab === 'overview'" class="bg-white rounded-lg border border-gray-200 px-4 py-2.5 mb-3 mt-4">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold text-gray-700">Recent Comments</h3>
          <NuxtLink :to="`/@${username}/comments`" class="text-xs text-primary hover:text-primary-hover no-underline">
            View all
          </NuxtLink>
        </div>
      </div>
      <CommonLoadingSpinner v-if="commentsLoading && recentComments.length === 0" size="md" />
      <div v-else-if="recentComments.length > 0" class="space-y-2">
        <div
          v-for="comment in recentComments"
          :key="comment.id"
          class="bg-white border border-gray-200 rounded-lg p-3"
        >
          <div class="flex items-center gap-2 text-xs text-gray-500 mb-1">
            <span>{{ comment.score }} points</span>
            <span>&middot;</span>
            <span>{{ comment.createdAt }}</span>
          </div>
          <p class="text-sm text-gray-800 line-clamp-3">{{ comment.body }}</p>
        </div>
      </div>
      <div v-else-if="!commentsLoading" class="bg-white rounded-lg border border-gray-200 py-8 text-center">
        <p class="text-sm text-gray-500">No comments yet.</p>
      </div>
    </template>
  </div>
</template>
