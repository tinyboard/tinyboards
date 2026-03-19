<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'
import type { User } from '~/types/generated'

useHead({ title: 'Members' })

const LIST_USERS_QUERY = `
  query ListUsers($searchTerm: String, $page: Int, $limit: Int) {
    listUsers(searchTerm: $searchTerm, page: $page, limit: $limit) {
      id
      name
      displayName
      avatar
      bio
      createdAt
      postCount
      commentCount
      postScore
      commentScore
      isBanned
      isAdmin
    }
  }
`

interface ListUsersResponse {
  listUsers: User[]
}

const { execute, loading, error } = useGraphQL<ListUsersResponse>()

const users = ref<User[]>([])
const page = ref(1)
const searchTerm = ref('')
const limit = 25
const hasMore = ref(false)

async function fetchUsers (): Promise<void> {
  const result = await execute(LIST_USERS_QUERY, {
    variables: {
      searchTerm: searchTerm.value || undefined,
      page: page.value,
      limit: limit + 1,
    },
  })
  if (result?.listUsers) {
    hasMore.value = result.listUsers.length > limit
    users.value = result.listUsers.slice(0, limit)
  }
}

async function handleSearch (): Promise<void> {
  page.value = 1
  await fetchUsers()
}

async function nextPage (): Promise<void> {
  if (hasMore.value) {
    page.value++
    await fetchUsers()
  }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) {
    page.value--
    await fetchUsers()
  }
}

await fetchUsers()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-lg font-semibold text-gray-900">
        Members
      </h1>
    </div>

    <div class="mb-4">
      <form @submit.prevent="handleSearch" class="flex gap-2">
        <input
          v-model="searchTerm"
          type="search"
          class="form-input flex-1"
          placeholder="Search members..."
        >
        <button type="submit" class="button button-sm primary">
          Search
        </button>
      </form>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchUsers" />
    <CommonLoadingSpinner v-else-if="loading && users.length === 0" size="lg" />

    <div v-else-if="users.length > 0" class="grid gap-3 sm:grid-cols-2">
      <UserCard v-for="user in users" :key="user.id" :user="user" />
    </div>
    <p v-else class="text-sm text-gray-500 text-center py-8">
      No members found.
    </p>

    <CommonPagination
      v-if="users.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
