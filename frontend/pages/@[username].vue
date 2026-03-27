<script setup lang="ts">
import { useUser } from '~/composables/useUser'
import { useAuthStore } from '~/stores/auth'

const route = useRoute()
const username = computed(() => route.params.username as string)
const authStore = useAuthStore()

const { user, loading, error, fetchUser } = useUser()

const isOwnProfile = computed(() => authStore.user?.name === username.value)

useHead({ title: computed(() => user.value?.displayName ?? username.value) })
useSeoMeta({
  title: computed(() => `${user.value?.displayName ?? username.value} (@${username.value})`),
  ogTitle: computed(() => `${user.value?.displayName ?? username.value} (@${username.value})`),
  description: computed(() => user.value?.bio ? user.value.bio.substring(0, 160) : `Profile of @${username.value}`),
  ogDescription: computed(() => user.value?.bio ? user.value.bio.substring(0, 160) : `Profile of @${username.value}`),
  ogImage: computed(() => user.value?.avatar || undefined),
  ogType: 'profile',
})

watch(username, (name) => { fetchUser(name) })
await fetchUser(username.value)

// Provide user data to child pages
provide('profileUser', user)
provide('profileLoading', loading)
provide('profileIsOwnProfile', isOwnProfile)
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="loading && !user" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="fetchUser(username)" />

    <template v-else-if="user">
      <UserProfile :user="user" :is-own-profile="isOwnProfile" />
      <UserProfileTabs :username="username" :is-own-profile="isOwnProfile" />

      <div class="max-w-5xl mx-auto px-4 py-4">
        <UserActions v-if="!isOwnProfile" :user="user" class="mb-4" />
        <UserAdminActions
          v-if="!isOwnProfile && authStore.user?.isAdmin"
          :user="user"
          class="mb-4"
          @updated="fetchUser(username)"
        />
        <NuxtPage />
      </div>
    </template>
  </div>
</template>
