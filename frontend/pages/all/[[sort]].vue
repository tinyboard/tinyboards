<script setup lang="ts">
import { usePosts } from '~/composables/usePosts'
import { useSiteStore } from '~/stores/site'
import type { ListingType } from '~/types/generated'

const siteStore = useSiteStore()

useHead({ title: 'All' })
useSeoMeta({
  title: computed(() => `All Posts | ${siteStore.name || 'TinyBoards'}`),
  ogTitle: computed(() => `All Posts | ${siteStore.name || 'TinyBoards'}`),
  description: computed(() => siteStore.description || 'All posts across the community.'),
  ogDescription: computed(() => siteStore.description || 'All posts across the community.'),
  ogType: 'website',
})

const route = useRoute()

const { posts, loading, error, page, sort, hasMore, fetchPosts, nextPage, prevPage, setSort } = usePosts({
  listingType: 'all' as ListingType,
  basePath: '/all',
})

if (route.params.sort && typeof route.params.sort === 'string') {
  sort.value = route.params.sort
}

await fetchPosts()
</script>

<template>
  <div>
    <!-- Sort bar -->
    <div class="pt-4">
      <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
        <CommonSortSelector v-model="sort" @update:model-value="setSort" />
        <CommonViewToggle />
      </div>
    </div>

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
