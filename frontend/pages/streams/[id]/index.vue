<script setup lang="ts">
import { useStreams, type StreamData } from '~/composables/useStreams'
import { useStreamFeed } from '~/composables/useStreamFeed'
import { useAuthStore } from '~/stores/auth'
import { useToast } from '~/composables/useToast'

const route = useRoute()
const streamId = route.params.id as string
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
  regenerateShareToken,
  mutating,
} = useStreams()

const { posts, loading: loadingFeed, fetchFeed, hasMore, nextPage: loadMore } = useStreamFeed()

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

const showShareModal = ref(false)
const shareUrl = computed(() => {
  if (!stream.value?.creator) return ''
  return `${window.location.origin}/streams/@${stream.value.creator.name}/${stream.value.slug}`
})

async function loadData () {
  await fetchStream(streamId)
  if (boardNames.value.length > 0) {
    await fetchFeed(boardNames.value)
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
  await fetchStream(streamId)
}

async function handleToggleNavbar () {
  if (!stream.value) return
  // For simplicity, toggle navbar with no position (appends to end)
  await updateNavbarSettings(stream.value.id, true)
  toast.success('Added to sidebar')
}

async function handleRegenerateToken () {
  if (!stream.value) return
  const token = await regenerateShareToken(stream.value.id)
  if (token) {
    await fetchStream(streamId)
    toast.success('Share link generated')
    showShareModal.value = true
  }
}

function copyShareUrl () {
  if (import.meta.client) {
    navigator.clipboard.writeText(shareUrl.value)
    toast.success('Link copied')
  }
}

async function loadNextPage () {
  if (boardNames.value.length > 0) {
    await loadMore(boardNames.value)
  }
}

await loadData()

useHead({ title: computed(() => stream.value?.name ?? 'Stream') })
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <CommonLoadingSpinner v-if="loading" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="loadData" />

    <template v-else-if="stream">
      <!-- Stream header -->
      <div class="flex items-start justify-between mb-6">
        <div class="flex-1 min-w-0">
          <h1 class="text-lg font-semibold text-gray-900">{{ stream.name }}</h1>
          <p v-if="stream.creator" class="text-sm text-gray-500 mt-0.5">
            by
            <NuxtLink :to="`/@${stream.creator.name}`" class="hover:underline">
              @{{ stream.creator.name }}
            </NuxtLink>
          </p>
          <p v-if="stream.description" class="text-sm text-gray-600 mt-1">{{ stream.description }}</p>

          <!-- Board subscription pills -->
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

          <!-- Stats row -->
          <div class="flex items-center gap-4 mt-2 text-xs text-gray-400">
            <span v-if="stream.followerCount != null">{{ stream.followerCount }} followers</span>
            <span v-if="stream.boardSubscriptionCount != null">{{ stream.boardSubscriptionCount }} boards</span>
          </div>
        </div>

        <!-- Action buttons -->
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
            @click="handleToggleNavbar"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" />
            </svg>
          </button>

          <ClientOnly>
            <button
              v-if="isOwner"
              class="button white button-sm"
              title="Share stream"
              @click="showShareModal ? (showShareModal = false) : handleRegenerateToken()"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
              </svg>
            </button>
          </ClientOnly>

          <NuxtLink
            v-if="isOwner"
            :to="`/streams/${streamId}/edit`"
            class="button white button-sm"
          >
            Edit
          </NuxtLink>
        </div>
      </div>

      <!-- Share modal -->
      <ClientOnly>
        <div v-if="showShareModal && stream.shareToken" class="bg-gray-50 border rounded-lg p-4 mb-4">
          <div class="flex items-center justify-between mb-2">
            <h3 class="text-sm font-medium text-gray-700">Share Link</h3>
            <button class="text-xs text-gray-400 hover:text-gray-600" @click="showShareModal = false">Close</button>
          </div>
          <div class="flex gap-2">
            <input
              :value="shareUrl"
              readonly
              class="form-input flex-1 text-sm"
              @focus="($event.target as HTMLInputElement).select()"
            />
            <button class="button button-sm primary" @click="copyShareUrl">Copy</button>
          </div>
        </div>
      </ClientOnly>

      <!-- Feed -->
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
