<script setup lang="ts">
import { useGraphQL, useGraphQLMutation } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'

definePageMeta({ middleware: 'guards' })

const route = useRoute()
const userId = route.params.id as string
const authStore = useAuthStore()

interface PrivateMessage {
  id: string
  creatorId: string
  body: string
  isRead: boolean
  createdAt: string
}

const CONVERSATION_QUERY = `
  query GetConversation($userId: ID!, $limit: Int, $offset: Int) {
    getConversation(userId: $userId, limit: $limit, offset: $offset) {
      id
      creatorId
      body
      isRead
      createdAt
    }
  }
`

const SEND_MESSAGE_MUTATION = `
  mutation SendMessage($input: SendMessageInput!) {
    sendMessage(input: $input) {
      message {
        id
        creatorId
        body
        isRead
        createdAt
      }
    }
  }
`

interface ConversationResponse {
  getConversation: PrivateMessage[]
}

interface SendMessageResponse {
  sendMessage: {
    message: PrivateMessage
  }
}

const { execute, loading, error } = useGraphQL<ConversationResponse>()
const { execute: executeSend, loading: sending } = useGraphQLMutation<SendMessageResponse>()

const messages = ref<PrivateMessage[]>([])
const page = ref(1)
const limit = 50
const hasMore = ref(false)
const threadContainer = ref<HTMLElement | null>(null)

useHead({ title: `Conversation` })

function formatTimestamp (dateString: string): string {
  const date = new Date(dateString)
  return date.toLocaleString(undefined, {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })
}

function isOwnMessage (message: PrivateMessage): boolean {
  return message.creatorId === authStore.user?.id
}

function scrollToBottom (): void {
  nextTick(() => {
    if (threadContainer.value) {
      threadContainer.value.scrollTop = threadContainer.value.scrollHeight
    }
  })
}

async function fetchMessages (): Promise<void> {
  const result = await execute(CONVERSATION_QUERY, {
    variables: {
      userId,
      limit: limit + 1,
      offset: 0,
    },
  })
  if (result?.getConversation) {
    hasMore.value = result.getConversation.length > limit
    messages.value = result.getConversation.slice(0, limit)
    scrollToBottom()
  }
}

async function loadOlder (): Promise<void> {
  page.value++
  const result = await execute(CONVERSATION_QUERY, {
    variables: {
      userId,
      limit: limit + 1,
      offset: (page.value - 1) * limit,
    },
  })
  if (result?.getConversation) {
    hasMore.value = result.getConversation.length > limit
    messages.value = [...result.getConversation.slice(0, limit), ...messages.value]
  }
}

async function handleSend (body: string): Promise<void> {
  const result = await executeSend(SEND_MESSAGE_MUTATION, {
    variables: {
      input: {
        recipientId: userId,
        body,
      },
    },
  })
  if (result?.sendMessage?.message) {
    messages.value.push(result.sendMessage.message)
    scrollToBottom()
  }
}

await fetchMessages()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4 flex flex-col" style="height: calc(100vh - 8rem);">
    <div class="flex items-center gap-2 mb-4">
      <NuxtLink to="/inbox/messages" class="button button-sm white">
        Back
      </NuxtLink>
      <h1 class="text-lg font-semibold text-gray-900">
        Conversation
      </h1>
    </div>

    <CommonErrorDisplay v-if="error" :message="error.message" @retry="fetchMessages" />
    <CommonLoadingSpinner v-else-if="loading && messages.length === 0" size="lg" />

    <template v-else>
      <div
        ref="threadContainer"
        class="flex-1 overflow-y-auto bg-white border border-gray-200 rounded-t p-4 space-y-3"
      >
        <div v-if="hasMore" class="text-center pb-2">
          <button class="button button-sm white" :disabled="loading" @click="loadOlder">
            Load older messages
          </button>
        </div>

        <p v-if="messages.length === 0" class="text-sm text-gray-500 text-center py-8">
          No messages yet. Start the conversation below.
        </p>

        <div
          v-for="message in messages"
          :key="message.id"
          class="flex"
          :class="isOwnMessage(message) ? 'justify-end' : 'justify-start'"
        >
          <div
            class="max-w-[75%] rounded-lg px-3 py-2"
            :class="isOwnMessage(message)
              ? 'bg-primary text-white'
              : 'bg-gray-100 text-gray-900'"
          >
            <p class="text-sm whitespace-pre-wrap break-words">
              {{ message.body }}
            </p>
            <p
              class="text-[11px] mt-1"
              :class="isOwnMessage(message) ? 'text-white/70' : 'text-gray-400'"
            >
              {{ formatTimestamp(message.createdAt) }}
            </p>
          </div>
        </div>
      </div>

      <div class="bg-white border border-t-0 border-gray-200 rounded-b p-3">
        <MessagesMessageComposer :disabled="sending" @send="handleSend" />
      </div>
    </template>
  </div>
</template>
