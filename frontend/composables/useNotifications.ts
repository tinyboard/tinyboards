import { ref } from 'vue'
import { useGraphQL } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'

export interface NotificationActor {
  id: string
  name: string
  displayName: string | null
  avatar: string | null
}

export interface NotificationPostContext {
  id: string
  title: string
  boardName: string
  boardId: string
}

export interface NotificationCommentContext {
  id: string
  body: string
  postId: string
  postTitle: string
  boardName: string
}

export interface NotificationMessageContext {
  id: string
  body: string
}

export interface Notification {
  id: string
  type: string
  isRead: boolean
  createdAt: string
  commentId: string | null
  postId: string | null
  messageId: string | null
  actor: NotificationActor | null
  post: NotificationPostContext | null
  comment: NotificationCommentContext | null
  message: NotificationMessageContext | null
}

interface UnreadCount {
  total: number
  replies: number
  mentions: number
  privateMessages: number
  activity: number
}

const NOTIFICATIONS_QUERY = `
  query GetNotifications($unreadOnly: Boolean, $kindFilter: String, $page: Int, $limit: Int) {
    getNotifications(unreadOnly: $unreadOnly, kindFilter: $kindFilter, page: $page, limit: $limit) {
      id type isRead createdAt commentId postId messageId
      actor { id name displayName avatar }
      post { id title boardName boardId }
      comment { id body postId postTitle boardName }
      message { id body }
    }
  }
`

const UNREAD_COUNT_QUERY = `
  query GetUnreadCount {
    getUnreadNotificationCount {
      total replies mentions privateMessages activity
    }
  }
`

const MARK_READ_MUTATION = `
  mutation MarkNotificationsRead($notificationIds: [ID!]!) {
    markNotificationsRead(notificationIds: $notificationIds) {
      success markedCount
    }
  }
`

const MARK_ALL_READ_MUTATION = `
  mutation MarkAllRead {
    markAllNotificationsAsRead {
      success markedCount
    }
  }
`

const DELETE_NOTIFICATION_MUTATION = `
  mutation DeleteNotification($notificationId: ID!) {
    deleteNotification(notificationId: $notificationId) {
      success
    }
  }
`

export function useNotifications () {
  const { execute, loading, error } = useGraphQL<{ getNotifications: Notification[] }>()
  const notifications = ref<Notification[]>([])
  const unreadCount = ref<UnreadCount | null>(null)
  const page = ref(1)
  const limit = 25

  const hasMore = ref(false)

  async function fetchNotifications (options?: { unreadOnly?: boolean; kindFilter?: string }): Promise<void> {
    const result = await execute(NOTIFICATIONS_QUERY, {
      variables: {
        unreadOnly: options?.unreadOnly ?? false,
        kindFilter: options?.kindFilter ?? undefined,
        page: page.value,
        limit: limit + 1,
      },
    })
    if (result?.getNotifications) {
      hasMore.value = result.getNotifications.length > limit
      notifications.value = result.getNotifications.slice(0, limit)
    }
  }

  async function fetchUnreadCount (): Promise<void> {
    const { execute: exec } = useGraphQL<{ getUnreadNotificationCount: UnreadCount }>()
    const result = await exec(UNREAD_COUNT_QUERY)
    if (result?.getUnreadNotificationCount) {
      unreadCount.value = result.getUnreadNotificationCount
    }
  }

  async function markRead (ids: string[]): Promise<void> {
    const { execute: exec } = useGraphQL()
    await exec(MARK_READ_MUTATION, { variables: { notificationIds: ids } })
    notifications.value = notifications.value.map(n =>
      ids.includes(n.id) ? { ...n, isRead: true } : n
    )
    // Sync the header badge count
    const authStore = useAuthStore()
    const unreadRemaining = notifications.value.filter(n => !n.isRead).length
    // Decrement by how many we just marked (approximate — full refresh on next poll)
    authStore.setUnreadNotificationCount(
      Math.max(0, authStore.unreadNotificationCount - ids.length)
    )
  }

  async function markAllRead (): Promise<void> {
    const { execute: exec } = useGraphQL()
    await exec(MARK_ALL_READ_MUTATION)
    notifications.value = notifications.value.map(n => ({ ...n, isRead: true }))
    const authStore = useAuthStore()
    authStore.setUnreadNotificationCount(0)
  }

  async function deleteNotification (id: string): Promise<void> {
    const { execute: exec } = useGraphQL()
    await exec(DELETE_NOTIFICATION_MUTATION, { variables: { notificationId: id } })
    const wasUnread = notifications.value.find(n => n.id === id && !n.isRead)
    notifications.value = notifications.value.filter(n => n.id !== id)
    if (wasUnread) {
      const authStore = useAuthStore()
      authStore.setUnreadNotificationCount(
        Math.max(0, authStore.unreadNotificationCount - 1)
      )
    }
  }

  return {
    notifications,
    unreadCount,
    loading,
    error,
    page,
    hasMore,
    fetchNotifications,
    fetchUnreadCount,
    markRead,
    markAllRead,
    deleteNotification,
  }
}
