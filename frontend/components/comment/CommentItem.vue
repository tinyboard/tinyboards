<script setup lang="ts">
import { ref } from 'vue'
import type { Comment } from '~/types/generated'
import { timeAgo } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'
import { useAuthStore } from '~/stores/auth'

type CommentNode = Comment & { children?: CommentNode[] }

const props = defineProps<{
  comment: CommentNode
  postId: string
  depth?: number
  isModerator?: boolean
  boardId?: string
}>()

const emit = defineEmits<{
  reply: [parentId: string, body: string]
}>()

const authStore = useAuthStore()
const collapsed = ref(false)
const showReply = ref(false)
const replyBody = ref('')

const maxDepth = 8
const currentDepth = props.depth ?? 0

function submitReply (): void {
  if (!replyBody.value.trim()) { return }
  emit('reply', props.comment.id, replyBody.value)
  replyBody.value = ''
  showReply.value = false
}
</script>

<template>
  <div
    class="border-l-2 sm:pl-3 mt-2"
    :class="[
      collapsed ? 'border-gray-200' : 'border-gray-300',
      currentDepth >= 4 ? 'pl-1' : 'pl-2'
    ]"
  >
    <!-- Comment header -->
    <div class="flex items-center gap-1 text-xs text-gray-500">
      <button
        class="text-gray-400 hover:text-gray-600 mr-1 p-2 sm:p-1 -m-1 min-w-[36px] sm:min-w-[28px] text-center"
        @click="collapsed = !collapsed"
      >
        [{{ collapsed ? '+' : '−' }}]
      </button>
      <NuxtLink
        v-if="comment.creator"
        :to="`/@${comment.creator.name}`"
        class="font-medium text-gray-700 no-underline hover:underline"
      >
        {{ comment.creator.displayName ?? comment.creator.name }}
      </NuxtLink>
      <span v-else class="text-gray-400 italic">[deleted]</span>
      <span v-if="comment.distinguishedAs === 'admin'" class="inline-flex items-center px-1 py-0 rounded text-[10px] font-medium bg-green-100 text-green-700 border border-green-200 leading-4">
        Admin
      </span>
      <span v-else-if="comment.distinguishedAs === 'mod'" class="inline-flex items-center px-1 py-0 rounded text-[10px] font-medium bg-blue-100 text-blue-700 border border-blue-200 leading-4">
        Mod
      </span>
      <span>&middot;</span>
      <time :datetime="comment.createdAt">{{ timeAgo(comment.createdAt) }}</time>
      <span v-if="comment.isPinned" class="inline-flex items-center rounded-full bg-amber-100 px-1.5 py-0.5 text-xs font-medium text-amber-800 ml-1">
        pinned
      </span>
    </div>

    <!-- Comment body -->
    <div v-if="!collapsed">
      <div v-if="comment.isDeleted" class="text-sm text-gray-400 italic mt-1">
        [deleted]
      </div>
      <div v-else-if="comment.isRemoved" class="text-sm text-red-400 italic mt-1">
        [removed]
      </div>
      <!-- eslint-disable-next-line vue/no-v-html -->
      <div v-else-if="comment.bodyHTML" class="prose prose-sm mt-1 max-w-none" v-html="sanitizeHtml(comment.bodyHTML)" />
      <div v-else class="prose prose-sm mt-1 max-w-none whitespace-pre-wrap">
        {{ comment.body }}
      </div>

      <!-- Signature -->
      <div
        v-if="comment.creator?.signature && !comment.isDeleted && !comment.isRemoved"
        class="mt-2 pt-2 border-t border-gray-100"
      >
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div class="prose prose-xs text-gray-400 max-w-none text-xs italic" v-html="sanitizeHtml(comment.creator.signature)" />
      </div>

      <!-- Actions -->
      <CommentActions
        :comment="comment"
        :is-moderator="isModerator"
        @toggle-reply="showReply = !showReply"
      />

      <!-- Reply form -->
      <div v-if="showReply && authStore.isLoggedIn" class="mt-2">
        <EditorMarkdownEditor
          v-model="replyBody"
          :board-id="boardId"
          placeholder="Write a reply..."
          min-height="80px"
        />
        <div class="flex gap-2 mt-1">
          <button class="button button-sm primary" @click="submitReply">
            Reply
          </button>
          <button class="button button-sm gray" @click="showReply = false">
            Cancel
          </button>
        </div>
      </div>

      <!-- Child comments (recursive) -->
      <div v-if="comment.children?.length && currentDepth < maxDepth">
        <CommentItem
          v-for="child in comment.children"
          :key="child.id"
          :comment="child"
          :post-id="postId"
          :depth="currentDepth + 1"
          :is-moderator="isModerator"
          :board-id="boardId"
          @reply="(parentId, body) => emit('reply', parentId, body)"
        />
      </div>
    </div>
  </div>
</template>
