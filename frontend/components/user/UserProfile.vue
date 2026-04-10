<script setup lang="ts">
import type { User } from '~/types/generated'
import { formatDate } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'

defineProps<{
  user: User
  isOwnProfile?: boolean
}>()
</script>

<template>
  <div class="bg-white border-b border-gray-200">
    <!-- Banner / Profile background -->
    <div
      class="h-32 sm:h-40 overflow-hidden relative"
      :class="!user.profileBackground && !user.banner ? 'bg-gradient-to-br from-primary to-primary-hover' : ''"
    >
      <img
        v-if="user.profileBackground"
        :src="user.profileBackground"
        :alt="`${user.name} profile background`"
        class="w-full h-full object-cover"
      >
      <img
        v-else-if="user.banner"
        :src="user.banner"
        :alt="`${user.name} banner`"
        class="w-full h-full object-cover"
      >
      <!-- Darkening overlay for readability when using profile background -->
      <div
        v-if="user.profileBackground"
        class="absolute inset-0 bg-gradient-to-t from-black/40 to-transparent"
      />
    </div>

    <!-- Profile info bar -->
    <div class="max-w-5xl mx-auto px-4">
      <div class="flex items-center gap-4 py-3">
        <!-- Avatar with optional frame -->
        <div class="relative shrink-0">
          <CommonAvatar
            :src="user.avatar ?? undefined"
            :name="user.name"
            size="lg"
            class="border-2 border-white shadow"
          />
          <!-- Avatar frame overlay -->
          <img
            v-if="user.avatarFrame"
            :src="user.avatarFrame"
            :alt="''"
            class="absolute inset-0 w-full h-full pointer-events-none"
            aria-hidden="true"
          >
        </div>

        <!-- Name + handle -->
        <div class="flex-1 min-w-0">
          <h1 class="text-lg font-bold text-gray-900 truncate">
            {{ user.displayName ?? user.name }}
          </h1>
          <p class="text-sm text-gray-500">@{{ user.name }}</p>
        </div>

        <!-- Edit profile button -->
        <div class="shrink-0">
          <NuxtLink
            v-if="isOwnProfile"
            to="/settings/profile"
            class="button button-sm white no-underline"
          >
            Edit Profile
          </NuxtLink>
        </div>
      </div>

      <!-- Bio -->
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-if="user.bioHTML" class="pb-3 text-sm text-gray-600 prose prose-sm max-w-none" v-html="sanitizeHtml(user.bioHTML)" />
      <p v-else-if="user.bio" class="pb-3 text-sm text-gray-600">{{ user.bio }}</p>

      <!-- Stats -->
      <div class="pb-3 flex flex-wrap gap-x-5 gap-y-2 text-sm text-gray-500">
        <span><strong class="text-gray-900">{{ user.postCount }}</strong> posts</span>
        <span><strong class="text-gray-900">{{ user.commentCount }}</strong> comments</span>
        <span title="Post Karma">
          <svg class="w-3.5 h-3.5 inline-block text-orange-400" fill="currentColor" viewBox="0 0 20 20">
            <path d="M2 10.5a1.5 1.5 0 113 0v6a1.5 1.5 0 01-3 0v-6zM6 10.333v5.43a2 2 0 001.106 1.79l.05.025A4 4 0 008.943 18h5.416a2 2 0 001.962-1.608l1.2-6A2 2 0 0015.56 8H12V4a2 2 0 00-2-2 1 1 0 00-1 1v.667a4 4 0 01-.8 2.4L6.8 7.933a4 4 0 00-.8 2.4z" />
          </svg>
          <strong class="text-gray-900">{{ user.postScore ?? 0 }}</strong> Post Karma
        </span>
        <span title="Comment Karma">
          <svg class="w-3.5 h-3.5 inline-block text-blue-400" fill="currentColor" viewBox="0 0 20 20">
            <path d="M2 10.5a1.5 1.5 0 113 0v6a1.5 1.5 0 01-3 0v-6zM6 10.333v5.43a2 2 0 001.106 1.79l.05.025A4 4 0 008.943 18h5.416a2 2 0 001.962-1.608l1.2-6A2 2 0 0015.56 8H12V4a2 2 0 00-2-2 1 1 0 00-1 1v.667a4 4 0 01-.8 2.4L6.8 7.933a4 4 0 00-.8 2.4z" />
          </svg>
          <strong class="text-gray-900">{{ user.commentScore ?? 0 }}</strong> Comment Karma
        </span>
        <span class="flex items-center gap-1">
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
          Joined {{ formatDate(user.createdAt) }}
        </span>
      </div>
    </div>
  </div>
</template>
