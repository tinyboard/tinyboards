<script setup lang="ts">
import { ref, watch } from 'vue'
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'

definePageMeta({ layout: 'admin' })
useHead({ title: 'Admin - Bans' })

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

interface BannedUsersResponse {
  listBannedUsers: {
    users: User[]
    totalCount: number
  }
}

interface UnbanUserResponse {
  unbanUserFromSite: { success: boolean; message: string }
}

const page = ref(1)
const limit = 20

const { execute, data, loading, error } = useGraphQL<BannedUsersResponse>()
const { execute: executeUnban, loading: unbanLoading } = useGraphQLMutation<UnbanUserResponse>()

const LIST_BANNED_QUERY = `
  query ListBannedUsers($page: Int, $limit: Int) {
    listBannedUsers(page: $page, limit: $limit) {
      users {
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
      totalCount
    }
  }
`

async function fetchBannedUsers () {
  await execute(LIST_BANNED_QUERY, {
    variables: { page: page.value, limit },
  })
}

async function unbanUser (userId: string) {
  if (!confirm('Are you sure you want to unban this user?')) return

  await executeUnban(`
    mutation UnbanUserFromSite($userId: ID!) {
      unbanUserFromSite(userId: $userId) { success message }
    }
  `, { variables: { userId } })

  await fetchBannedUsers()
}

watch(page, fetchBannedUsers)

fetchBannedUsers()

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString()
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h2 class="text-lg font-semibold text-gray-900">
        Ban Management
      </h2>
      <span
        v-if="data?.listBannedUsers"
        class="text-sm text-gray-500"
      >
        {{ data.listBannedUsers.totalCount }} banned user{{ data.listBannedUsers.totalCount === 1 ? '' : 's' }}
      </span>
    </div>

    <CommonLoadingSpinner v-if="loading" size="lg" />

    <CommonErrorDisplay
      v-else-if="error"
      :message="error.message"
      @retry="fetchBannedUsers"
    />

    <div v-else-if="data?.listBannedUsers?.users?.length">
      <div class="space-y-3">
        <div
          v-for="user in data.listBannedUsers.users"
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
            <button
              class="button button-sm primary"
              :disabled="unbanLoading"
              @click="unbanUser(user.id)"
            >
              Unban
            </button>
          </div>
        </div>
      </div>

      <CommonPagination
        :page="page"
        :has-more="data.listBannedUsers.users.length >= limit"
        @prev="page--"
        @next="page++"
      />
    </div>

    <div v-else class="py-12 text-center text-sm text-gray-500">
      No banned users.
    </div>
  </div>
</template>
