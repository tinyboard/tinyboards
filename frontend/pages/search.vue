<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { Post, Comment, User, Board } from '~/types/generated'
import { sanitizeHtml } from '~/utils/sanitize'

useHead({ title: 'Search' })

const SEARCH_QUERY = `
  query SearchContent($q: String!, $searchType: SearchType, $page: Int, $limit: Int) {
    searchContent(q: $q, searchType: $searchType, page: $page, limit: $limit) {
      posts {
        id
        title
        body
        bodyHTML
        url
        createdAt
        slug
        score
        commentCount
        board { id name title icon }
        creator { id name displayName avatar }
      }
      comments {
        id
        body
        bodyHTML
        createdAt
        score
        postId
        post { slug board { name } }
        creator { id name displayName avatar }
      }
      users {
        id
        name
        displayName
        avatar
        postScore
        commentScore
      }
      boards {
        id
        name
        title
        description
        icon
        subscribers
      }
    }
  }
`

interface SearchResult {
  posts: Post[]
  comments: Comment[]
  users: User[]
  boards: Board[]
}

interface SearchResponse {
  searchContent: SearchResult
}

const { execute, loading, error } = useGraphQL<SearchResponse>()

const route = useRoute()
const router = useRouter()

const query = ref((route.query.q as string) ?? '')
const searchType = ref<string>((route.query.type as string) ?? 'all')
const page = ref(Number(route.query.page) || 1)
const limit = 20
const results = ref<SearchResult | null>(null)
const hasSearched = ref(false)
const hasMore = ref(false)

const tabs = [
  { value: 'all', label: 'All' },
  { value: 'posts', label: 'Posts' },
  { value: 'comments', label: 'Comments' },
  { value: 'users', label: 'Users' },
  { value: 'boards', label: 'Boards' },
]

async function search (): Promise<void> {
  if (query.value.trim().length < 2) { return }

  hasSearched.value = true
  router.replace({ query: { q: query.value, type: searchType.value, page: page.value > 1 ? page.value : undefined } })

  const result = await execute(SEARCH_QUERY, {
    variables: {
      q: query.value,
      searchType: searchType.value,
      page: page.value,
      limit: limit + 1,
    },
  })
  if (result?.searchContent) {
    const content = result.searchContent
    // Check if any result type exceeds limit (indicating more results)
    const totalResults = content.posts.length + content.comments.length + content.users.length + content.boards.length
    hasMore.value = totalResults > limit
    // Trim to limit
    results.value = {
      posts: content.posts.slice(0, limit),
      comments: content.comments.slice(0, limit),
      users: content.users.slice(0, limit),
      boards: content.boards.slice(0, limit),
    }
  }
}

async function nextPage (): Promise<void> {
  if (hasMore.value) {
    page.value++
    await search()
  }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) {
    page.value--
    await search()
  }
}

async function setTab (tab: string): Promise<void> {
  searchType.value = tab
  page.value = 1
  if (hasSearched.value) {
    await search()
  }
}

// Auto-search if query param exists
if (query.value) {
  await search()
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-4">
      Search
    </h1>

    <form @submit.prevent="search" class="mb-4 flex gap-2">
      <input
        v-model="query"
        type="search"
        class="form-input flex-1"
        placeholder="Search posts, comments, users, boards..."
      >
      <button type="submit" class="button button-sm primary">
        Search
      </button>
    </form>

    <div v-if="hasSearched" class="mb-4 flex gap-1 border-b border-gray-200">
      <button
        v-for="tab in tabs"
        :key="tab.value"
        class="px-3 py-2 text-sm font-medium border-b-2 -mb-px transition-colors"
        :class="searchType === tab.value
          ? 'border-primary text-primary'
          : 'border-transparent text-gray-500 hover:text-gray-700'"
        @click="setTab(tab.value)"
      >
        {{ tab.label }}
      </button>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="search" />
    <CommonLoadingSpinner v-else-if="loading" size="lg" />

    <template v-else-if="results">
      <!-- Posts -->
      <div v-if="results.posts.length > 0 && (searchType === 'all' || searchType === 'posts')" class="mb-6">
        <h2 v-if="searchType === 'all'" class="text-sm font-semibold text-gray-700 mb-2">Posts</h2>
        <PostList :posts="results.posts" :loading="false" />
      </div>

      <!-- Comments -->
      <div v-if="results.comments.length > 0 && (searchType === 'all' || searchType === 'comments')" class="mb-6">
        <h2 v-if="searchType === 'all'" class="text-sm font-semibold text-gray-700 mb-2">Comments</h2>
        <div class="space-y-2">
          <NuxtLink
            v-for="comment in results.comments"
            :key="comment.id"
            :to="comment.post?.board?.name && comment.post?.slug
              ? `/b/${comment.post.board.name}/feed/${comment.postId}/${comment.post.slug}`
              : '#'"
            class="block bg-white border border-gray-200 rounded p-3 hover:border-gray-300 transition-colors"
          >
            <div class="flex items-center gap-2 text-xs text-gray-500 mb-1">
              <span v-if="comment.creator" class="font-medium text-gray-700">
                {{ comment.creator.displayName ?? comment.creator.name }}
              </span>
              <span>&middot; {{ comment.score }} points</span>
            </div>
            <div v-if="comment.bodyHTML" class="text-sm text-gray-800 line-clamp-3 prose prose-sm max-w-none [&>*]:m-0" v-html="sanitizeHtml(comment.bodyHTML)" />
            <p v-else class="text-sm text-gray-800 line-clamp-3">{{ comment.body }}</p>
          </NuxtLink>
        </div>
      </div>

      <!-- Users -->
      <div v-if="results.users.length > 0 && (searchType === 'all' || searchType === 'users')" class="mb-6">
        <h2 v-if="searchType === 'all'" class="text-sm font-semibold text-gray-700 mb-2">Users</h2>
        <div class="grid gap-3 sm:grid-cols-2">
          <UserCard v-for="user in results.users" :key="user.id" :user="user" />
        </div>
      </div>

      <!-- Boards -->
      <div v-if="results.boards.length > 0 && (searchType === 'all' || searchType === 'boards')" class="mb-6">
        <h2 v-if="searchType === 'all'" class="text-sm font-semibold text-gray-700 mb-2">Boards</h2>
        <div class="grid gap-3 sm:grid-cols-2">
          <BoardCard v-for="board in results.boards" :key="board.id" :board="board" />
        </div>
      </div>

      <!-- No results -->
      <p
        v-if="results.posts.length === 0 && results.comments.length === 0 && results.users.length === 0 && results.boards.length === 0"
        class="text-sm text-gray-500 text-center py-8"
      >
        No results found for "{{ query }}".
      </p>

      <CommonPagination
        v-if="results.posts.length > 0 || results.comments.length > 0 || results.users.length > 0 || results.boards.length > 0"
        :page="page"
        :has-more="hasMore"
        @prev="prevPage"
        @next="nextPage"
      />
    </template>

    <p v-else-if="!hasSearched" class="text-sm text-gray-500">
      Enter a search term to find content.
    </p>
  </div>
</template>
