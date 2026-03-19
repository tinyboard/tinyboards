import { ref, computed } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'

interface PrivateMessage {
  id: string
  creatorId: string
  recipientId: string | null
  subject: string | null
  body: string
  isRead: boolean
  createdAt: string
  updatedAt: string
}

interface Conversation {
  otherUser: {
    id: string
    name: string
    displayName: string | null
    avatar: string | null
  }
  lastMessage: PrivateMessage
  unreadCount: number
  lastActivity: string
}

const CONVERSATIONS_QUERY = `
  query ListConversations {
    listConversations {
      otherUser { id name displayName avatar }
      lastMessage { id body createdAt isRead }
      unreadCount
      lastActivity
    }
  }
`

const CONVERSATION_QUERY = `
  query GetConversation($userId: ID!, $limit: Int, $offset: Int) {
    getConversation(userId: $userId, limit: $limit, offset: $offset) {
      id creatorId body isRead createdAt
    }
  }
`

const SEND_MESSAGE_MUTATION = `
  mutation SendMessage($input: SendMessageInput!) {
    sendMessage(input: $input) {
      message { id creatorId body createdAt }
    }
  }
`

const UNREAD_COUNT_QUERY = `
  query GetUnreadMessageCount {
    getUnreadMessageCount
  }
`

export function useMessages () {
  const { execute, loading, error } = useGraphQL<{ listConversations: Conversation[] }>()
  const conversations = ref<Conversation[]>([])
  const messages = ref<PrivateMessage[]>([])
  const unreadCount = ref(0)

  async function fetchConversations (): Promise<void> {
    const result = await execute(CONVERSATIONS_QUERY)
    if (result?.listConversations) {
      conversations.value = result.listConversations
    }
  }

  async function fetchConversation (userId: string, limit = 50, offset = 0): Promise<void> {
    const { execute: exec, error: fetchError } = useGraphQL<{ getConversation: PrivateMessage[] }>()
    const result = await exec(CONVERSATION_QUERY, {
      variables: { userId, limit, offset },
    })
    if (result?.getConversation) {
      messages.value = result.getConversation
    }
  }

  async function sendMessage (recipientId: string, body: string): Promise<PrivateMessage | null> {
    const { execute: exec, error: sendError } = useGraphQL<{ sendMessage: { message: PrivateMessage } }>()
    const result = await exec(SEND_MESSAGE_MUTATION, {
      variables: { input: { recipientId, body } },
    })
    if (result?.sendMessage?.message) {
      messages.value.push(result.sendMessage.message)
      return result.sendMessage.message
    }
    return null
  }

  async function fetchUnreadCount (): Promise<void> {
    const { execute: exec } = useGraphQL<{ getUnreadMessageCount: number }>()
    const result = await exec(UNREAD_COUNT_QUERY)
    if (result) {
      unreadCount.value = result.getUnreadMessageCount
    }
  }

  return {
    conversations,
    messages,
    unreadCount,
    loading,
    error,
    fetchConversations,
    fetchConversation,
    sendMessage,
    fetchUnreadCount,
  }
}
