<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'
import type { Board } from '~/types/generated'

const authStore = useAuthStore()

useHead({ title: 'Boards' })

const LIST_BOARDS_QUERY = `
  query ListBoards($page: Int, $limit: Int, $sort: SortType, $searchTerm: String, $searchTitleAndDesc: Boolean) {
    listBoards(page: $page, limit: $limit, sort: $sort, searchTerm: $searchTerm, searchTitleAndDesc: $searchTitleAndDesc) {
      id
      name
      title
      description
      icon
      subscribers
      posts
      comments
      usersActiveDay
      usersActiveWeek
      isNSFW
      createdAt
    }
  }
`

interface ListBoardsResponse {
  listBoards: Board[]
}

const { execute, loading, error } = useGraphQL<ListBoardsResponse>()

const boards = ref<Board[]>([])
const page = ref(1)
const searchTerm = ref('')
const sort = ref('hot')
const limit = 25
const hasMore = ref(false)

async function fetchBoards (): Promise<void> {
  const result = await execute(LIST_BOARDS_QUERY, {
    variables: {
      page: page.value,
      limit: limit + 1,
      sort: sort.value,
      searchTerm: searchTerm.value || undefined,
      searchTitleAndDesc: searchTerm.value ? true : undefined,
    },
  })
  if (result?.listBoards) {
    hasMore.value = result.listBoards.length > limit
    boards.value = result.listBoards.slice(0, limit)
  }
}

async function handleSearch (): Promise<void> {
  page.value = 1
  await fetchBoards()
}

async function nextPage (): Promise<void> {
  if (hasMore.value) {
    page.value++
    await fetchBoards()
  }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) {
    page.value--
    await fetchBoards()
  }
}

await fetchBoards()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-lg font-semibold text-gray-900">
        Board Directory
      </h1>
      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/boards/create"
        class="button button-sm primary no-underline"
      >
        Create Board
      </NuxtLink>
    </div>

    <div class="mb-4 flex flex-col sm:flex-row gap-2">
      <form @submit.prevent="handleSearch" class="flex gap-2 flex-1">
        <input
          v-model="searchTerm"
          type="search"
          class="form-input flex-1"
          placeholder="Search boards..."
        >
        <button type="submit" class="button button-sm primary">
          Search
        </button>
      </form>
      <select
        :value="sort"
        class="form-input w-auto"
        @change="sort = ($event.target as HTMLSelectElement).value; page = 1; fetchBoards()"
      >
        <option value="hot">Hot</option>
        <option value="new">New</option>
        <option value="topDay">Top</option>
        <option value="active">Active</option>
        <option value="mostComments">Most Comments</option>
      </select>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchBoards" />
    <CommonLoadingSpinner v-else-if="loading && boards.length === 0" size="lg" />

    <div v-else-if="boards.length > 0" class="grid gap-3 sm:grid-cols-2">
      <BoardCard v-for="board in boards" :key="board.id" :board="board" />
    </div>
    <p v-else class="text-sm text-gray-500 text-center py-8">
      No boards found.
    </p>

    <CommonPagination
      v-if="boards.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
