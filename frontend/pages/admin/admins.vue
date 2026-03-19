<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
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
const limit = 20

const { execute, data, loading, error } = useGraphQL<ListUsersResponse>()

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

const adminUsers = computed(() => {
  if (!data.value?.listUsers) return []
  return data.value.listUsers.filter(u => u.isAdmin)
})

async function fetchUsers () {
  await execute(LIST_USERS_QUERY, {
    variables: {
      page: page.value,
      limit: 100,
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
              <div class="font-medium text-gray-900">
                {{ user.displayName || user.name }}
              </div>
              <div class="text-xs text-gray-500">
                @{{ user.name }} &middot; Joined {{ formatDate(user.createdAt) }}
              </div>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-500">
              {{ user.postCount }} posts &middot; {{ user.commentCount }} comments
            </span>
            <span
              class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800"
            >
              {{ levelLabel(user.adminLevel) }}
            </span>
          </div>
        </div>
      </div>

      <CommonPagination
        :page="page"
        :has-more="(data?.listUsers?.length ?? 0) >= 100"
        @prev="page--"
        @next="page++"
      />
    </div>

    <div v-else class="py-12 text-center text-sm text-gray-500">
      No admin users found.
    </div>
  </div>
</template>
