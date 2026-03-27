<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useToast } from '~/composables/useToast'

definePageMeta({ layout: 'admin' })
const toast = useToast()
useHead({ title: 'Admin - Admins' })

interface User {
  id: string
  name: string
  displayName: string
  avatar: string | null
  isBanned: boolean
  isAdmin: boolean
  adminLevel: number
  createdAt: string
  postCount: number
  commentCount: number
}

interface ListUsersResponse {
  listUsers: User[]
}

const page = ref(1)
const limit = 50
const searchTerm = ref('')

const { execute, data, loading, error } = useGraphQL<ListUsersResponse>()
const { execute: executeMutation, loading: mutationLoading } = useGraphQL()

const LIST_USERS_QUERY = `
  query ListUsers($searchTerm: String, $page: Int, $limit: Int) {
    listUsers(searchTerm: $searchTerm, page: $page, limit: $limit) {
      id
      name
      displayName
      avatar
      isBanned
      isAdmin
      adminLevel
      createdAt
      postCount
      commentCount
    }
  }
`

const SET_ADMIN_LEVEL_MUTATION = `
  mutation SetUserAdminLevel($userId: ID!, $adminLevel: Int!) {
    setUserAdminLevel(userId: $userId, adminLevel: $adminLevel) {
      id
      name
      isAdmin
      adminLevel
    }
  }
`

const REMOVE_ADMIN_MUTATION = `
  mutation SetUserAdminLevel($userId: ID!, $adminLevel: Int!) {
    setUserAdminLevel(userId: $userId, adminLevel: $adminLevel) {
      id
      name
      isAdmin
      adminLevel
    }
  }
`

const adminUsers = computed(() => {
  if (!data.value?.listUsers) return []
  return data.value.listUsers.filter(u => u.isAdmin)
})

// Modal state for adding a new admin
const showAddAdmin = ref(false)
const newAdminSearch = ref('')
const selectedUser = ref<User | null>(null)
const selectedLevel = ref(1)
const { execute: executeSearch, data: searchData } = useGraphQL<ListUsersResponse>()

async function searchUsers () {
  if (!newAdminSearch.value.trim()) return
  await executeSearch(LIST_USERS_QUERY, {
    variables: {
      searchTerm: newAdminSearch.value,
      page: 1,
      limit: 10,
    },
  })
}

let searchTimeout: ReturnType<typeof setTimeout> | null = null
function onSearchInput () {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(searchUsers, 300)
}

const searchResults = computed(() => {
  if (!searchData.value?.listUsers) return []
  return searchData.value.listUsers.filter(u => !u.isAdmin)
})

async function addAdmin () {
  if (!selectedUser.value) return
  const result = await executeMutation(SET_ADMIN_LEVEL_MUTATION, {
    variables: { userId: selectedUser.value.id, adminLevel: selectedLevel.value },
  })
  if (result) {
    toast.success(`${selectedUser.value.displayName || selectedUser.value.name} is now an admin`)
    showAddAdmin.value = false
    selectedUser.value = null
    newAdminSearch.value = ''
    selectedLevel.value = 1
    await fetchUsers()
  }
}

async function removeAdmin (user: User) {
  if (!confirm(`Remove admin privileges from ${user.displayName || user.name}?`)) return
  const result = await executeMutation(REMOVE_ADMIN_MUTATION, {
    variables: { userId: user.id, adminLevel: 0 },
  })
  if (result) {
    toast.success(`Admin privileges removed from ${user.displayName || user.name}`)
    await fetchUsers()
  }
}

async function changeLevel (user: User, newLevel: number) {
  const result = await executeMutation(SET_ADMIN_LEVEL_MUTATION, {
    variables: { userId: user.id, adminLevel: newLevel },
  })
  if (result) {
    toast.success(`Admin level updated to ${newLevel}`)
    await fetchUsers()
  }
}

async function fetchUsers () {
  await execute(LIST_USERS_QUERY, {
    variables: {
      page: page.value,
      limit,
    },
  })
}

watch(page, fetchUsers)

fetchUsers()

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}

function levelLabel (level: number): string {
  switch (level) {
    case 1: return 'Moderator'
    case 2: return 'Admin'
    case 3: return 'Super Admin'
    default: return `Level ${level}`
  }
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-lg font-semibold text-gray-900">
        Admin Management
      </h2>
      <button class="button primary button-sm" @click="showAddAdmin = true">
        Add Admin
      </button>
    </div>

    <CommonLoadingSpinner v-if="loading" size="lg" />

    <CommonErrorDisplay
      v-else-if="error"
      :message="error.message"
      @retry="fetchUsers"
    />

    <div v-else-if="adminUsers.length">
      <div class="space-y-3">
        <div
          v-for="user in adminUsers"
          :key="user.id"
          class="flex items-center justify-between p-4 bg-white border rounded-lg"
        >
          <div class="flex items-center gap-3">
            <CommonAvatar
              :src="user.avatar ?? undefined"
              :name="user.displayName || user.name"
              size="sm"
            />
            <div>
              <NuxtLink :to="`/@${user.name}`" class="font-medium text-gray-900 hover:underline">
                {{ user.displayName || user.name }}
              </NuxtLink>
              <div class="text-xs text-gray-500">
                @{{ user.name }} &middot; Joined {{ formatDate(user.createdAt) }}
              </div>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-500">
              {{ user.postCount }} posts &middot; {{ user.commentCount }} comments
            </span>

            <select
              :value="user.adminLevel"
              class="form-select text-xs py-1 px-2 rounded border-gray-300"
              :disabled="mutationLoading"
              @change="changeLevel(user, parseInt(($event.target as HTMLSelectElement).value))"
            >
              <option :value="1">Level 1 - Appearance</option>
              <option :value="2">Level 2 - Config</option>
              <option :value="3">Level 3 - Content</option>
              <option :value="4">Level 4 - Users</option>
              <option :value="5">Level 5 - Boards</option>
              <option :value="6">Level 6 - Full</option>
              <option :value="7">Level 7 - Owner</option>
            </select>

            <button
              class="button button-sm text-red-600 hover:text-red-800 hover:bg-red-50"
              :disabled="mutationLoading"
              @click="removeAdmin(user)"
            >
              Remove
            </button>
          </div>
        </div>
      </div>

      <CommonPagination
        :page="page"
        :has-more="(data?.listUsers?.length ?? 0) >= limit"
        @prev="page--"
        @next="page++"
      />
    </div>

    <div v-else class="py-12 text-center text-sm text-gray-500">
      No admin users found.
    </div>

    <!-- Add Admin Modal -->
    <div v-if="showAddAdmin" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showAddAdmin = false">
      <div class="bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <h3 class="text-lg font-semibold mb-4">Add Administrator</h3>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Search User</label>
          <input
            v-model="newAdminSearch"
            type="text"
            class="form-input w-full"
            placeholder="Search by username..."
            @input="onSearchInput"
          >
        </div>

        <div v-if="searchResults.length" class="mb-4 max-h-40 overflow-y-auto border rounded">
          <button
            v-for="u in searchResults"
            :key="u.id"
            class="flex items-center gap-2 w-full px-3 py-2 text-left hover:bg-gray-50 text-sm"
            :class="selectedUser?.id === u.id ? 'bg-primary/10' : ''"
            @click="selectedUser = u"
          >
            <CommonAvatar :src="u.avatar ?? undefined" :name="u.name" size="xs" />
            <span>{{ u.displayName || u.name }}</span>
            <span class="text-gray-400">@{{ u.name }}</span>
          </button>
        </div>

        <div v-if="selectedUser" class="mb-4 p-3 bg-gray-50 rounded text-sm">
          Selected: <strong>{{ selectedUser.displayName || selectedUser.name }}</strong> (@{{ selectedUser.name }})
        </div>

        <div class="mb-4">
          <label class="block text-sm font-medium text-gray-700 mb-1">Admin Level</label>
          <select v-model="selectedLevel" class="form-select w-full">
            <option :value="1">Level 1 - Appearance</option>
            <option :value="2">Level 2 - Config</option>
            <option :value="3">Level 3 - Content</option>
            <option :value="4">Level 4 - Users</option>
            <option :value="5">Level 5 - Boards</option>
            <option :value="6">Level 6 - Full</option>
            <option :value="7">Level 7 - Owner</option>
          </select>
        </div>

        <div class="flex justify-end gap-2">
          <button class="button gray button-sm" @click="showAddAdmin = false">Cancel</button>
          <button
            class="button primary button-sm"
            :disabled="!selectedUser || mutationLoading"
            @click="addAdmin"
          >
            Grant Admin
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
