<script setup lang="ts">
defineProps<{
  name: string
  description?: string
  slug?: string
  creatorName?: string
  followerCount?: number | null
  boardCount?: number | null
  isFollowing?: boolean | null
  color?: string | null
}>()

defineEmits<{
  follow: []
  unfollow: []
}>()
</script>

<template>
  <div class="bg-white border border-gray-200 rounded-lg p-4 hover:border-gray-300 transition-colors">
    <div class="flex items-start justify-between">
      <div class="flex-1 min-w-0">
        <NuxtLink
          :to="creatorName && slug ? `/streams/@${creatorName}/${slug}` : '#'"
          class="text-sm font-semibold text-gray-900 hover:text-primary transition-colors no-underline"
        >
          {{ name }}
        </NuxtLink>
        <p v-if="creatorName" class="text-xs text-gray-500 mt-0.5">by @{{ creatorName }}</p>
        <p v-if="description" class="text-xs text-gray-500 mt-1 line-clamp-2">{{ description }}</p>
        <div class="flex items-center gap-3 mt-2 text-xs text-gray-400">
          <span v-if="followerCount != null">{{ followerCount }} followers</span>
          <span v-if="boardCount != null">{{ boardCount }} boards</span>
        </div>
      </div>
      <button
        v-if="isFollowing != null"
        class="button button-sm shrink-0 ml-3"
        :class="isFollowing ? 'white' : 'primary'"
        @click="isFollowing ? $emit('unfollow') : $emit('follow')"
      >
        {{ isFollowing ? 'Unfollow' : 'Follow' }}
      </button>
    </div>
  </div>
</template>
