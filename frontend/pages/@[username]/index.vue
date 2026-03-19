<script setup lang="ts">
import { useUser } from '~/composables/useUser'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post } from '~/types/generated'

const route = useRoute()
const username = computed(() => route.params.username as string)
const authStore = useAuthStore()

const { user, loading, error, fetchUser } = useUser()

useHead({ title: computed(() => user.value?.displayName ?? username.value) })
useSeoMeta({
  title: computed(() => `${user.value?.displayName ?? username.value} (@${username.value})`),
  ogTitle: computed(() => `${user.value?.displayName ?? username.value} (@${username.value})`),
  description: computed(() => user.value?.bio ? user.value.bio.substring(0, 160) : `Profile of @${username.value}`),
  ogDescription: computed(() => user.value?.bio ? user.value.bio.substring(0, 160) : `Profile of @${username.value}`),
  ogImage: computed(() => user.value?.avatar || undefined),
  ogType: 'profile',
})

const isOwnProfile = computed(() => authStore.user?.name === username.value)

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
      board { id name title icon }
      creator { id name displayName avatar }
    }
  }
`

interface RecentPostsResponse {
  listPosts: Post[]
}

const { execute: execPosts, loading: postsLoading } = useGraphQL<RecentPostsResponse>()
const recentPosts = ref<Post[]>([])

async function loadProfile (name: string): Promise<void> {
  await fetchUser(name)
  if (user.value) {
    const postsResult = await execPosts(RECENT_POSTS_QUERY, {
      variables: { userName: name, limit: 10 },
    })
    recentPosts.value = postsResult?.listPosts ?? []
  }
}

watch(username, (name) => { loadProfile(name) })
await loadProfile(username.value)
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="loading && !user" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="loadProfile(username)" />

    <template v-else-if="user">
      <UserProfile :user="user" :is-own-profile="isOwnProfile" />

      <div class="max-w-5xl mx-auto px-4 py-4">
        <UserActions v-if="!isOwnProfile" :user="user" class="mb-4" />

        <div class="bg-white rounded-lg border border-gray-200 px-4 py-3 mb-4">
          <h3 class="text-sm font-semibold text-gray-700">Recent Posts</h3>
        </div>
        <PostList :posts="recentPosts" :loading="postsLoading" />
        <div v-if="!postsLoading && recentPosts.length === 0" class="bg-white rounded-lg border border-gray-200 py-12 text-center">
          <p class="text-sm text-gray-500">No posts yet.</p>
        </div>
      </div>
    </template>
  </div>
</template>
