<script setup lang="ts">
import { useStreams } from '~/composables/useStreams'
import { useStreamFeed } from '~/composables/useStreamFeed'
import { useAuthStore } from '~/stores/auth'
import { useToast } from '~/composables/useToast'

const route = useRoute()
const username = route.params.username as string
const slug = route.params.slug as string
const authStore = useAuthStore()
const toast = useToast()

const {
  stream,
  loading,
  error,
  fetchStream,
  followStream,
  unfollowStream,
  updateNavbarSettings,
  mutating,
} = useStreams()

const { posts, loading: loadingFeed, fetchFeed, hasMore, nextPage: loadMore } = useStreamFeed()
const notFound = ref(false)

const isOwner = computed(() => {
  if (!stream.value?.creatorId || !authStore.user) return false
  return stream.value.creatorId === authStore.user.id
})

const isFollowing = computed(() => stream.value?.isFollowing ?? false)

const boardNames = computed(() => {
  if (!stream.value?.boardSubscriptions) return []
  return stream.value.boardSubscriptions
    .map(s => s.board?.name)
    .filter((n): n is string => !!n)
})

async function loadData () {
  const result = await fetchStream(undefined, slug)
  if (result) {
    if (result.creator?.name !== username) {
      notFound.value = true
      return
    }
    if (boardNames.value.length > 0) {
      await fetchFeed(boardNames.value)
    }
  } else {
    notFound.value = true
  }
}

async function handleFollow () {
  if (!stream.value) return
  if (isFollowing.value) {
    await unfollowStream(stream.value.id)
    toast.success('Unfollowed stream')
  } else {
    await followStream(stream.value.id)
    toast.success('Following stream')
  }
  await fetchStream(undefined, slug)
}

async function handleAddToNavbar () {
  if (!stream.value) return
  await updateNavbarSettings(stream.value.id, true)
  toast.success('Added to sidebar')
}

async function loadNextPage () {
  if (boardNames.value.length > 0) {
    await loadMore(boardNames.value)
  }
}

await loadData()

useHead({ title: computed(() => stream.value ? `${stream.value.name} by @${username}` : `${slug} by @${username}`) })
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="loadData" />

    <div v-else-if="notFound" class="text-center py-12">
      <p class="text-sm text-gray-500 mb-4">Stream not found.</p>
      <NuxtLink to="/streams" class="text-sm text-primary hover:underline">
        Browse streams
      </NuxtLink>
    </div>

    <template v-else-if="stream">
      <div class="flex items-start justify-between mb-6">
        <div class="flex-1 min-w-0">
          <h1 class="text-lg font-semibold text-gray-900">{{ stream.name }}</h1>
          <p class="text-sm text-gray-500 mt-0.5">
            by <NuxtLink :to="`/@${username}`" class="hover:underline">@{{ username }}</NuxtLink>
          </p>
          <p v-if="stream.description" class="text-sm text-gray-600 mt-1">{{ stream.description }}</p>

          <div v-if="stream.boardSubscriptions && stream.boardSubscriptions.length > 0" class="flex flex-wrap gap-1.5 mt-2">
            <NuxtLink
              v-for="sub in stream.boardSubscriptions"
              :key="sub.boardId"
              :to="sub.board ? `/b/${sub.board.name}` : '#'"
              class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-2.5 py-0.5 text-xs text-gray-700 hover:bg-gray-200 no-underline"
            >
              +{{ sub.board?.name ?? 'unknown' }}
            </NuxtLink>
          </div>

          <div class="flex items-center gap-4 mt-2 text-xs text-gray-400">
            <span v-if="stream.followerCount != null">{{ stream.followerCount }} followers</span>
            <span v-if="stream.boardSubscriptionCount != null">{{ stream.boardSubscriptionCount }} boards</span>
          </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <button
            v-if="authStore.isLoggedIn && !isOwner"
            class="button button-sm"
            :class="isFollowing ? 'white' : 'primary'"
            :disabled="mutating"
            @click="handleFollow"
          >
            {{ isFollowing ? 'Unfollow' : 'Follow' }}
          </button>

          <button
            v-if="authStore.isLoggedIn && isFollowing"
            class="button white button-sm"
            title="Add to sidebar navigation"
            @click="handleAddToNavbar"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" />
            </svg>
          </button>

          <NuxtLink
            v-if="isOwner"
            :to="`/streams/${stream.id}/edit`"
            class="button white button-sm"
          >
            Edit
          </NuxtLink>
        </div>
      </div>

      <CommonLoadingSpinner v-if="loadingFeed" />

      <div v-else-if="posts.length === 0" class="text-center py-12">
        <p class="text-sm text-gray-500">No posts in this stream yet.</p>
      </div>

      <div v-else class="space-y-3">
        <PostCard
          v-for="post in posts"
          :key="post.id"
          :post="post"
        />
        <div v-if="hasMore" class="text-center py-4">
          <button class="button white button-sm" @click="loadNextPage">
            Load more
          </button>
        </div>
      </div>
    </template>
  </div>
</template>
