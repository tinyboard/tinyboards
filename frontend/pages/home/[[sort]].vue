<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import { usePosts } from '~/composables/usePosts'
import type { ListingType } from '~/types/generated'

const authStore = useAuthStore()
const siteStore = useSiteStore()

useHead({ title: 'Home' })
useSeoMeta({
  title: computed(() => `Home | ${siteStore.name || 'TinyBoards'}`),
  ogTitle: computed(() => `Home | ${siteStore.name || 'TinyBoards'}`),
  description: computed(() => siteStore.description || 'A community-driven discussion platform.'),
  ogDescription: computed(() => siteStore.description || 'A community-driven discussion platform.'),
  ogImage: computed(() => siteStore.icon || undefined),
  ogType: 'website',
})
const route = useRoute()

const { posts, loading, error, page, sort, hasMore, fetchPosts, nextPage, prevPage, setSort } = usePosts({
  listingType: (authStore.isLoggedIn ? 'subscribed' : 'all') as ListingType,
  basePath: '/home',
})

// Initialize sort from URL param
if (route.params.sort && typeof route.params.sort === 'string') {
  sort.value = route.params.sort
}

await fetchPosts()
</script>

<template>
  <div>
    <!-- Welcome banner for anonymous users -->
    <div v-if="!authStore.isLoggedIn" class="pt-4">
      <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div class="h-24 bg-gradient-to-br from-primary to-primary-hover" />
        <div class="px-6 py-4 -mt-6">
          <div class="w-12 h-12 rounded-xl bg-white shadow-md flex items-center justify-center border border-gray-100 mb-3">
            <img
              v-if="siteStore.icon"
              :src="siteStore.icon"
              class="w-8 h-8"
              :alt="siteStore.name"
            >
            <svg v-else class="w-7 h-7 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3.75 21h16.5M4.5 3h15M5.25 3v18m13.5-18v18M9 6.75h1.5m-1.5 3h1.5m-1.5 3h1.5m3-6H15m-1.5 3H15m-1.5 3H15M9 21v-3.375c0-.621.504-1.125 1.125-1.125h3.75c.621 0 1.125.504 1.125 1.125V21" />
            </svg>
          </div>
          <h1 class="text-xl font-bold text-gray-900 mb-1">
            Welcome to {{ siteStore.name || 'TinyBoards' }}
          </h1>
          <p class="text-sm text-gray-600 mb-3 max-w-lg">
            A community-driven platform for sharing ideas, discussions, and content.
            Join the conversation or browse what others are talking about.
          </p>
          <div class="flex items-center gap-2">
            <NuxtLink to="/register" class="button button-sm primary no-underline">
              Create Account
            </NuxtLink>
            <NuxtLink to="/boards" class="button button-sm white no-underline">
              Browse Boards
            </NuxtLink>
          </div>
        </div>
      </div>
    </div>

    <!-- Sort bar -->
    <div class="pt-4">
      <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
        <CommonSortSelector v-model="sort" @update:model-value="setSort" />
        <CommonViewToggle />
      </div>
    </div>

    <!-- Content area -->
    <div class="pb-4">
      <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />

      <PostList :posts="posts" :loading="loading" />

      <CommonPagination
        v-if="posts.length > 0"
        :page="page"
        :has-more="hasMore"
        @prev="prevPage"
        @next="nextPage"
      />
    </div>
  </div>
</template>
