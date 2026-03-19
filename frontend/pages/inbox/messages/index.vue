<script setup lang="ts">
import { useGraphQL } from '~/composables/useGraphQL'

definePageMeta({ middleware: 'guards' })
useHead({ title: 'Messages' })

interface OtherUser {
  id: string
  name: string
  displayName: string
  avatar: string | null
}

interface LastMessage {
  body: string
  createdAt: string
  isRead: boolean
}

interface Conversation {
  otherUser: OtherUser
  lastMessage: LastMessage
  unreadCount: number
  lastActivity: string
}

const CONVERSATIONS_QUERY = `
  query ListConversations {
    listConversations {
      otherUser {
        id
        name
        displayName
        avatar
      }
      lastMessage {
        body
        createdAt
        isRead
      }
      unreadCount
      lastActivity
    }
  }
`

interface ConversationsResponse {
  listConversations: Conversation[]
}

const { execute, loading, error } = useGraphQL<ConversationsResponse>()
const conversations = ref<Conversation[]>([])

function formatTime (dateString: string): string {
  const date = new Date(dateString)
  if (isNaN(date.getTime())) { return '' }
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMins / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffMins < 1) { return 'just now' }
  if (diffMins < 60) { return `${diffMins}m ago` }
  if (diffHours < 24) { return `${diffHours}h ago` }
  if (diffDays < 7) { return `${diffDays}d ago` }
  return date.toLocaleDateString()
}

function truncateBody (body: string, maxLength = 80): string {
  if (body.length <= maxLength) { return body }
  return body.slice(0, maxLength).trimEnd() + '...'
}

async function fetchConversations (): Promise<void> {
  const result = await execute(CONVERSATIONS_QUERY)
  if (result?.listConversations) {
    conversations.value = result.listConversations
  }
}

await fetchConversations()
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 py-4">
    <div class="bg-white rounded-lg border border-gray-200 px-4 py-3 mb-4 flex items-center justify-between">
      <h1 class="text-lg font-semibold text-gray-900">
        Messages
      </h1>
      <NuxtLink to="/inbox" class="button button-sm white no-underline">
        Back to Inbox
      </NuxtLink>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchConversations" />
    <CommonLoadingSpinner v-else-if="loading && conversations.length === 0" size="lg" />

    <div v-else-if="conversations.length > 0" class="bg-white border border-gray-200 rounded-lg divide-y divide-gray-100 overflow-hidden">
      <NuxtLink
        v-for="conversation in conversations"
        :key="conversation.otherUser.id"
        :to="`/inbox/messages/${conversation.otherUser.id}`"
        class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 transition-colors"
        :class="{ 'bg-primary/5': conversation.unreadCount > 0 }"
      >
        <CommonAvatar
          :src="conversation.otherUser.avatar ?? undefined"
          :name="conversation.otherUser.displayName || conversation.otherUser.name"
          size="md"
        />
        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between">
            <span
              class="text-sm truncate"
              :class="conversation.unreadCount > 0 ? 'font-semibold text-gray-900' : 'font-medium text-gray-700'"
            >
              {{ conversation.otherUser.displayName || conversation.otherUser.name }}
            </span>
            <span class="text-xs text-gray-400 shrink-0 ml-2">
              {{ formatTime(conversation.lastActivity) }}
            </span>
          </div>
          <div class="flex items-center justify-between mt-0.5">
            <p
              class="text-sm truncate"
              :class="conversation.unreadCount > 0 ? 'text-gray-700' : 'text-gray-500'"
            >
              {{ truncateBody(conversation.lastMessage.body) }}
            </p>
            <span
              v-if="conversation.unreadCount > 0"
              class="shrink-0 ml-2 inline-flex items-center justify-center w-5 h-5 text-xs font-semibold text-white bg-primary rounded-full"
            >
              {{ conversation.unreadCount }}
            </span>
          </div>
        </div>
      </NuxtLink>
    </div>

    <p v-else class="text-sm text-gray-500 text-center py-8">
      No conversations yet.
    </p>
  </div>
</template>
