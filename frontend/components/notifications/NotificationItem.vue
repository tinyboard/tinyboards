<script setup lang="ts">
const props = defineProps<{
  type: string
  message: string
  createdAt: string
  read: boolean
  postId?: string | null
  commentId?: string | null
  messageId?: string | null
}>()

const iconConfig = computed(() => {
  switch (props.type) {
    case 'comment_reply':
      return { color: 'text-blue-500 bg-blue-50', label: 'Reply' }
    case 'post_reply':
      return { color: 'text-indigo-500 bg-indigo-50', label: 'Reply' }
    case 'mention':
      return { color: 'text-amber-500 bg-amber-50', label: 'Mention' }
    case 'private_message':
    case 'message':
      return { color: 'text-green-500 bg-green-50', label: 'Message' }
    case 'follow':
      return { color: 'text-pink-500 bg-pink-50', label: 'Follow' }
    case 'board_invite':
      return { color: 'text-purple-500 bg-purple-50', label: 'Invite' }
    case 'moderation_action':
    case 'mod_action':
      return { color: 'text-red-500 bg-red-50', label: 'Mod' }
    case 'system':
      return { color: 'text-gray-500 bg-gray-100', label: 'System' }
    default:
      return { color: 'text-gray-400 bg-gray-50', label: 'Notification' }
  }
})

const href = computed(() => {
  if (props.commentId && props.postId) {
    return `/post/${props.postId}#comment-${props.commentId}`
  }
  if (props.postId) {
    return `/post/${props.postId}`
  }
  if (props.messageId) {
    return '/inbox?tab=messages'
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
</script>

<template>
  <component
    :is="href ? resolveComponent('NuxtLink') : 'div'"
    :to="href ?? undefined"
    class="flex items-start gap-3 px-4 py-3 transition-colors"
    :class="[
      read ? 'bg-white' : 'bg-blue-50',
      href ? 'hover:bg-gray-50 cursor-pointer' : ''
    ]"
  >
    <!-- Type-specific icon -->
    <div
      class="flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center mt-0.5"
      :class="iconConfig.color"
    >
      <!-- comment_reply / post_reply -->
      <svg v-if="type === 'comment_reply' || type === 'post_reply'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M3 10h10a8 8 0 018 8v2M3 10l6 6m-6-6l6-6" />
      </svg>
      <!-- mention -->
      <svg v-else-if="type === 'mention'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9" />
      </svg>
      <!-- private_message / message -->
      <svg v-else-if="type === 'private_message' || type === 'message'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
      </svg>
      <!-- follow -->
      <svg v-else-if="type === 'follow'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M19 7.5v3m0 0v3m0-3h3m-3 0h-3m-2.25-4.125a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zM4 19.235v-.11a6.375 6.375 0 0112.75 0v.109A12.318 12.318 0 0110.374 21c-2.331 0-4.512-.645-6.374-1.766z" />
      </svg>
      <!-- board_invite -->
      <svg v-else-if="type === 'board_invite'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M18 18.72a9.094 9.094 0 003.741-.479 3 3 0 00-4.682-2.72m.94 3.198l.001.031c0 .225-.012.447-.037.666A11.944 11.944 0 0112 21c-2.17 0-4.207-.576-5.963-1.584A6.062 6.062 0 016 18.719m12 0a5.971 5.971 0 00-.941-3.197m0 0A5.995 5.995 0 0012 12.75a5.995 5.995 0 00-5.058 2.772m0 0a3 3 0 00-4.681 2.72 8.986 8.986 0 003.74.477m.94-3.197a5.971 5.971 0 00-.94 3.197M15 6.75a3 3 0 11-6 0 3 3 0 016 0zm6 3a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0zm-13.5 0a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0z" />
      </svg>
      <!-- moderation_action / mod_action -->
      <svg v-else-if="type === 'moderation_action' || type === 'mod_action'" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75m-3-7.036A11.959 11.959 0 013.598 6 11.99 11.99 0 003 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285z" />
      </svg>
      <!-- system / default -->
      <svg v-else class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0" />
      </svg>
    </div>

    <div class="flex-1 min-w-0">
      <p class="text-sm text-gray-900">
        {{ message }}
      </p>
      <div class="flex items-center gap-2 mt-1">
        <span
          class="inline-flex items-center rounded px-1.5 py-0.5 text-xs font-medium"
          :class="iconConfig.color"
        >
          {{ iconConfig.label }}
        </span>
        <span class="text-xs text-gray-500">{{ formattedDate }}</span>
      </div>
    </div>

    <!-- Unread indicator dot -->
    <div v-if="!read" class="flex-shrink-0 mt-2">
      <div class="w-2 h-2 rounded-full bg-blue-500" />
    </div>
  </component>
</template>
