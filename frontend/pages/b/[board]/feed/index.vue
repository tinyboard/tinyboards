<script setup lang="ts">
import { usePosts } from '~/composables/usePosts'

const route = useRoute()
const boardName = route.params.board as string

const { posts, loading, error, page, sort, hasMore, fetchPosts, nextPage, prevPage, setSort } = usePosts({
  boardName,
  postType: 'feed',
})

await fetchPosts()
</script>

<template>
  <div>
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-2 flex items-center justify-between mb-4">
      <CommonSortSelector v-model="sort" @update:model-value="setSort" />
      <CommonViewToggle />
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchPosts" />
    <PostList :posts="posts" :loading="loading" />
    <CommonPagination v-if="posts.length > 0" :page="page" :has-more="hasMore" @prev="prevPage" @next="nextPage" />
  </div>
</template>
