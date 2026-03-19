<script setup lang="ts">
import { usePosts } from '~/composables/usePosts'
import { useBoard } from '~/composables/useBoard'

const route = useRoute()
const boardName = route.params.board as string

// Access the board data from the parent layout to check section config
const { board } = useBoard()

// If only one section is enabled, redirect to it
onMounted(() => {
  if (!board.value) return
  const config = board.value.sectionConfig ?? 3
  const hasFeed = (config & 1) === 1
  const hasThreads = (config & 2) === 2

  if (hasThreads && !hasFeed) {
    navigateTo(`/b/${boardName}/threads`, { replace: true })
  } else if (hasFeed && !hasThreads) {
    navigateTo(`/b/${boardName}/feed`, { replace: true })
  }
})

const { posts, loading, error, page, sort, hasMore, fetchPosts, nextPage, prevPage, setSort } = usePosts({
  boardName,
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
