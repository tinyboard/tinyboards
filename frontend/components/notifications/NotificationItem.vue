<script setup lang="ts">
import type { NotificationActor, NotificationPostContext, NotificationCommentContext, NotificationMessageContext } from '~/composables/useNotifications'

const props = defineProps<{
  id: string
  type: string
  createdAt: string
  read: boolean
  postId?: string | null
  commentId?: string | null
  messageId?: string | null
  actor?: NotificationActor | null
  post?: NotificationPostContext | null
  comment?: NotificationCommentContext | null
  message?: NotificationMessageContext | null
}>()

const emit = defineEmits<{
  markRead: [id: string]
  delete: [id: string]
}>()

const actorName = computed(() => {
  if (!props.actor) return null
  return props.actor.displayName || props.actor.name
})

const description = computed(() => {
  switch (props.type) {
    case 'comment_reply': return 'replied to your comment'
    case 'post_reply': return 'commented on your post'
    case 'mention': return 'mentioned you'
    case 'private_message': return 'sent you a message'
    case 'mod_action': return 'moderator action'
    case 'system': return 'system notification'
    default: return 'notification'
  }
})

const contextText = computed(() => {
  if (props.comment) {
    return props.comment.body
  }
  if (props.message) {
    return props.message.body
  }
  return null
})

const contextLabel = computed(() => {
  if (props.comment) {
    return `in ${props.comment.boardName} / ${props.comment.postTitle}`
  }
  if (props.post) {
    return `in ${props.post.boardName} / ${props.post.title}`
  }
  return null
})

const iconConfig = computed(() => {
  switch (props.type) {
    case 'comment_reply':
      return { color: 'text-blue-500 bg-blue-50', icon: 'reply' }
    case 'post_reply':
      return { color: 'text-indigo-500 bg-indigo-50', icon: 'reply' }
    case 'mention':
      return { color: 'text-amber-500 bg-amber-50', icon: 'mention' }
    case 'private_message':
      return { color: 'text-green-500 bg-green-50', icon: 'message' }
    case 'mod_action':
      return { color: 'text-red-500 bg-red-50', icon: 'mod' }
    case 'system':
      return { color: 'text-gray-500 bg-gray-100', icon: 'system' }
    default:
      return { color: 'text-gray-400 bg-gray-50', icon: 'system' }
  }
})

const href = computed(() => {
  // For comment replies and mentions with comment context, link to the comment
  if (props.comment) {
    return `/post/${props.comment.postId}#comment-${props.comment.id}`
  }
  if (props.commentId && props.postId) {
    return `/post/${props.postId}#comment-${props.commentId}`
  }
  if (props.postId) {
    return `/post/${props.postId}`
  }
  if (props.post) {
    return `/post/${props.post.id}`
  }
  if (props.messageId && props.actor) {
    return `/inbox/messages/${props.actor.id}`
  }
  if (props.messageId) {
    return '/inbox/messages'
  }
  return null
})

const formattedDate = computed(() => {
  try {
    const date = new Date(props.createdAt)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    const diffDays = Math.floor(diffMs / 86400000)

    if (diffMins < 1) return 'just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`
    return date.toLocaleDateString()
  } catch {
    return props.createdAt
  }
})

function handleClick () {
  if (!props.read) {
    emit('markRead', props.id)
  }
}
</script>

<template>
  <div
    class="relative flex items-start gap-3 px-4 py-3 transition-colors group"
    :class="[
      read ? 'bg-white' : 'bg-blue-50/60',
      href ? 'hover:bg-gray-50' : ''
    ]"
  >
    <!-- Actor avatar or type icon -->
    <component
      :is="actor ? resolveComponent('NuxtLink') : 'div'"
      :to="actor ? `/@${actor.name}` : undefined"
      class="flex-shrink-0 mt-0.5"
    >
      <CommonAvatar
        v-if="actor"
        :src="actor.avatar ?? undefined"
        :name="actorName ?? ''"
        size="sm"
      />
      <div
        v-else
        class="w-8 h-8 rounded-full flex items-center justify-center"
        :class="iconConfig.color"
      >
        <svg v-if="iconConfig.icon === 'system'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0" />
        </svg>
        <svg v-else-if="iconConfig.icon === 'mod'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
        </svg>
      </div>
    </component>

    <!-- Content -->
    <component
      :is="href ? resolveComponent('NuxtLink') : 'div'"
      :to="href ?? undefined"
      class="flex-1 min-w-0 no-underline"
      @click="handleClick"
    >
      <!-- Main line: "username replied to your comment" -->
      <p class="text-sm text-gray-900 leading-snug">
        <span v-if="actorName" class="font-semibold">{{ actorName }}</span>
        <span :class="actorName ? 'ml-1' : ''">{{ description }}</span>
      </p>

      <!-- Context: board / post title -->
      <p v-if="contextLabel" class="text-xs text-gray-500 mt-0.5 truncate">
        {{ contextLabel }}
      </p>

      <!-- Snippet: comment body or message body -->
      <p v-if="contextText" class="text-xs text-gray-600 mt-1 line-clamp-2 leading-relaxed">
        {{ contextText }}
      </p>

      <!-- Timestamp -->
      <span class="text-xs text-gray-400 mt-1 inline-block">{{ formattedDate }}</span>
    </component>

    <!-- Right side: unread dot + actions -->
    <div class="flex items-center gap-1.5 flex-shrink-0 mt-1">
      <!-- Mark read button (only if unread) -->
      <button
        v-if="!read"
        class="p-1 rounded hover:bg-blue-100 text-blue-400 hover:text-blue-600 transition-colors opacity-0 group-hover:opacity-100"
        title="Mark as read"
        @click.prevent.stop="$emit('markRead', id)"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
        </svg>
      </button>

      <!-- Delete button -->
      <button
        class="p-1 rounded hover:bg-red-100 text-gray-300 hover:text-red-500 transition-colors opacity-0 group-hover:opacity-100"
        title="Delete notification"
        @click.prevent.stop="$emit('delete', id)"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>

      <!-- Unread indicator dot -->
      <div v-if="!read" class="w-2 h-2 rounded-full bg-blue-500 ml-1" />
    </div>
  </div>
</template>
