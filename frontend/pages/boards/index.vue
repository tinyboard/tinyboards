<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'
import type { Board } from '~/types/generated'

const authStore = useAuthStore()
const route = useRoute()
const router = useRouter()

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
      mode
    }
  }
`

interface ListBoardsResponse {
  listBoards: (Board & { mode?: string })[]
}

const { execute, loading, error } = useGraphQL<ListBoardsResponse>()

const boards = ref<(Board & { mode?: string })[]>([])
const page = ref(1)
const searchTerm = ref('')
const sort = ref('hot')
const modeFilter = ref<string>((route.query.mode as string) || 'all')
const limit = 25
const hasMore = ref(false)

const filteredBoards = computed(() => {
  if (modeFilter.value === 'all') return boards.value
  return boards.value.filter(b => b.mode === modeFilter.value)
})

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

function setModeFilter (mode: string) {
  modeFilter.value = mode
  page.value = 1
  // Persist filter in URL
  const query = { ...route.query }
  if (mode === 'all') {
    delete query.mode
  } else {
    query.mode = mode
  }
  router.replace({ query })
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

    <!-- Mode filter -->
    <div class="mb-4 flex gap-1">
      <button
        type="button"
        class="px-3 py-1 rounded-full text-xs font-medium transition-colors"
        :class="modeFilter === 'all' ? 'bg-gray-900 text-white' : 'bg-gray-100 text-gray-600 hover:bg-gray-200'"
        @click="setModeFilter('all')"
      >
        All
      </button>
      <button
        type="button"
        class="px-3 py-1 rounded-full text-xs font-medium transition-colors"
        :class="modeFilter === 'feed' ? 'bg-blue-600 text-white' : 'bg-blue-50 text-blue-700 hover:bg-blue-100'"
        @click="setModeFilter('feed')"
      >
        📰 Feed
      </button>
      <button
        type="button"
        class="px-3 py-1 rounded-full text-xs font-medium transition-colors"
        :class="modeFilter === 'forum' ? 'bg-purple-600 text-white' : 'bg-purple-50 text-purple-700 hover:bg-purple-100'"
        @click="setModeFilter('forum')"
      >
        💬 Forum
      </button>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchBoards" />
    <CommonLoadingSpinner v-else-if="loading && boards.length === 0" size="lg" />

    <div v-else-if="filteredBoards.length > 0" class="grid gap-3 sm:grid-cols-2">
      <BoardCard v-for="board in filteredBoards" :key="board.id" :board="board" />
    </div>
    <p v-else class="text-sm text-gray-500 text-center py-8">
      No boards found.
    </p>

    <CommonPagination
      v-if="filteredBoards.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
