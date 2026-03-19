<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Inbox' })

interface Notification {
  id: string
  type: string
  isRead: boolean
  createdAt: string
  commentId: string | null
  postId: string | null
  messageId: string | null
}

const NOTIFICATIONS_QUERY = `
  query GetNotifications($unreadOnly: Boolean, $kindFilter: String, $page: Int, $limit: Int) {
    getNotifications(unreadOnly: $unreadOnly, kindFilter: $kindFilter, page: $page, limit: $limit) {
      id
      type
      isRead
      createdAt
      commentId
      postId
      messageId
    }
  }
`

const MARK_ALL_READ_MUTATION = `
  mutation MarkAllRead {
    markAllNotificationsAsRead {
      success
      markedCount
    }
  }
`

interface NotificationsResponse {
  getNotifications: Notification[]
}

const { execute, loading, error } = useGraphQL<NotificationsResponse>()
const { execute: executeMutation } = useGraphQL()

const notifications = ref<Notification[]>([])
const page = ref(1)
const unreadOnly = ref(false)
const kindFilter = ref<string | null>(null)
const limit = 25
const hasMore = ref(false)

const filters = [
  { value: null, label: 'All' },
  { value: 'replies', label: 'Replies' },
  { value: 'mention', label: 'Mentions' },
  { value: 'private_message', label: 'Messages' },
  { value: 'activity', label: 'Activity' },
]

function notificationLabel (type: string): string {
  switch (type) {
    case 'comment_reply': return 'replied to your comment'
    case 'post_reply': return 'replied to your post'
    case 'mention': return 'mentioned you'
    case 'private_message': return 'sent you a message'
    case 'mod_action': return 'moderator action'
    case 'system': return 'system notification'
    default: return 'notification'
  }
}

async function fetchNotifications (): Promise<void> {
  const result = await execute(NOTIFICATIONS_QUERY, {
    variables: {
      unreadOnly: unreadOnly.value,
      kindFilter: kindFilter.value,
      page: page.value,
      limit: limit + 1,
    },
  })
  if (result?.getNotifications) {
    hasMore.value = result.getNotifications.length > limit
    notifications.value = result.getNotifications.slice(0, limit)
  }
}

async function markAllRead (): Promise<void> {
  await executeMutation(MARK_ALL_READ_MUTATION)
  notifications.value = notifications.value.map(n => ({ ...n, isRead: true }))
}

async function setFilter (filter: string | null): Promise<void> {
  kindFilter.value = filter
  page.value = 1
  await fetchNotifications()
}

async function toggleUnread (): Promise<void> {
  unreadOnly.value = !unreadOnly.value
  page.value = 1
  await fetchNotifications()
}

async function nextPage (): Promise<void> {
  if (hasMore.value) {
    page.value++
    await fetchNotifications()
  }
}

async function prevPage (): Promise<void> {
  if (page.value > 1) {
    page.value--
    await fetchNotifications()
  }
}

await fetchNotifications()
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 py-4">
    <!-- Header card -->
    <div class="bg-white rounded-lg border border-gray-200 px-4 py-3 mb-4">
      <div class="flex items-center justify-between">
        <h1 class="text-lg font-semibold text-gray-900">
          Notifications
        </h1>
        <div class="flex items-center gap-2">
          <NuxtLink to="/inbox/messages" class="button button-sm white no-underline inline-flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            Messages
          </NuxtLink>
          <button
            class="button button-sm"
            :class="unreadOnly ? 'primary' : 'white'"
            @click="toggleUnread"
          >
            {{ unreadOnly ? 'Showing unread' : 'Unread only' }}
          </button>
          <button class="button button-sm white" @click="markAllRead">
            Mark all read
          </button>
        </div>
      </div>
    </div>

    <!-- Filter tabs in a card -->
    <div class="bg-white rounded-lg border border-gray-200 px-3 py-1 mb-4">
      <div class="flex gap-0.5">
        <button
          v-for="filter in filters"
          :key="filter.value ?? 'all'"
          class="px-3 py-2 text-sm font-medium border-b-2 -mb-px transition-colors"
          :class="kindFilter === filter.value
            ? 'border-primary text-primary'
            : 'border-transparent text-gray-500 hover:text-gray-700'"
          @click="setFilter(filter.value)"
        >
          {{ filter.label }}
        </button>
      </div>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchNotifications" />
    <CommonLoadingSpinner v-else-if="loading && notifications.length === 0" size="lg" />

    <div v-else-if="notifications.length > 0" class="bg-white border border-gray-200 rounded-lg divide-y divide-gray-100">
      <NotificationsNotificationItem
        v-for="notification in notifications"
        :key="notification.id"
        :type="notification.type"
        :message="notificationLabel(notification.type)"
        :created-at="notification.createdAt"
        :read="notification.isRead"
        :post-id="notification.postId"
        :comment-id="notification.commentId"
        :message-id="notification.messageId"
      />
    </div>
    <div v-else class="bg-white border border-gray-200 rounded-lg py-12 text-center">
      <p class="text-sm text-gray-500">No notifications.</p>
    </div>

    <CommonPagination
      v-if="notifications.length > 0"
      :page="page"
      :has-more="hasMore"
      @prev="prevPage"
      @next="nextPage"
    />
  </div>
</template>
