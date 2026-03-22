<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import type { User } from '~/types/generated'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const username = computed(() => route.params.username as string)
const authStore = useAuthStore()
const profileUser = inject<Ref<User | null>>('profileUser')

const isOwnProfile = computed(() => authStore.user?.name === username.value)

const FOLLOWING_QUERY = `
  query UserFollowing($userId: ID!) {
    userFollowing(userId: $userId) {
      id
      name
      displayName
      avatar
      bio
      postCount
      commentCount
      createdAt
    }
  }
`

interface FollowingResponse {
  userFollowing: User[]
}

const { execute, loading, error } = useGraphQL<FollowingResponse>()
const following = ref<User[]>([])

async function fetchFollowing (): Promise<void> {
  const userId = profileUser?.value?.id
  if (!userId) return
  const result = await execute(FOLLOWING_QUERY, {
    variables: { userId },
  })
  following.value = result?.userFollowing ?? []
}

if (isOwnProfile.value && profileUser?.value?.id) {
  await fetchFollowing()
}
</script>

<template>
  <div>
    <template v-if="isOwnProfile">
      <div class="bg-white rounded-lg border border-gray-200 px-4 py-2.5 mb-4">
        <h2 class="text-sm font-semibold text-gray-900">Following</h2>
      </div>

      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchFollowing" />
      <CommonLoadingSpinner v-else-if="loading" size="lg" />

      <div v-else-if="following.length > 0" class="space-y-2">
        <NuxtLink
          v-for="user in following"
          :key="user.id"
          :to="`/@${user.name}`"
          class="flex items-center gap-3 bg-white border border-gray-200 rounded-lg p-3 hover:border-gray-300 transition-colors no-underline"
        >
          <CommonAvatar
            :src="user.avatar ?? undefined"
            :name="user.name"
            size="md"
          />
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-gray-900 truncate">
              {{ user.displayName ?? user.name }}
            </p>
            <p class="text-xs text-gray-500">@{{ user.name }}</p>
            <p v-if="user.bio" class="text-xs text-gray-400 truncate mt-0.5">{{ user.bio }}</p>
          </div>
          <div class="text-right text-xs text-gray-400 shrink-0">
            <span>{{ user.postCount ?? 0 }} posts</span>
          </div>
        </NuxtLink>
      </div>

      <div v-else class="bg-white rounded-lg border border-gray-200 py-12 text-center">
        <div class="inline-flex w-12 h-12 rounded-xl bg-gray-100 items-center justify-center mb-3">
          <svg class="w-6 h-6 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
          </svg>
        </div>
        <p class="text-sm font-medium text-gray-600 mb-1">Not following anyone yet</p>
        <p class="text-xs text-gray-400">Follow users to see their content on your home feed.</p>
      </div>
    </template>
    <CommonErrorDisplay
      v-else
      title="Not available"
      message="You can only view your own following list."
      :retryable="false"
    />
  </div>
</template>
