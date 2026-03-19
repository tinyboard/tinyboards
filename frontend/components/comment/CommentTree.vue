<script setup lang="ts">
import { computed } from 'vue'
import type { Comment } from '~/types/generated'

const props = defineProps<{
  comments: Comment[]
  postId: string
  loading: boolean
  isModerator?: boolean
}>()

const emit = defineEmits<{
  reply: [parentId: string, body: string]
}>()

// Build a tree from flat comments using parentId and level
const rootComments = computed(() => {
  const byId = new Map<string, Comment & { children: Comment[] }>()
  const roots: Array<Comment & { children: Comment[] }> = []

  // First pass: create entries with children arrays
  for (const c of props.comments) {
    byId.set(c.id, { ...c, children: [] })
  }

  // Second pass: build parent-child relationships
  for (const c of props.comments) {
    const node = byId.get(c.id)!
    if (c.parentId && byId.has(c.parentId)) {
      byId.get(c.parentId)!.children.push(node)
    } else {
      roots.push(node)
    }
  }

  return roots
})
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="loading && comments.length === 0" />

    <div v-else-if="comments.length === 0" class="py-6 text-center text-sm text-gray-500">
      No comments yet. Be the first to comment.
    </div>

    <div v-else class="space-y-1">
      <CommentItem
        v-for="comment in rootComments"
        :key="comment.id"
        :comment="comment"
        :post-id="postId"
        :is-moderator="isModerator"
        @reply="(parentId, body) => emit('reply', parentId, body)"
      />
    </div>
  </div>
</template>
