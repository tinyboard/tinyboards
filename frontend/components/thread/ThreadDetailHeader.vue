<script setup lang="ts">
import type { Post } from '~/types/generated'
import { timeAgo } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import { useModeration } from '~/composables/useModeration'

const props = defineProps<{
  post: Post
  isModerator?: boolean
}>()

const emit = defineEmits<{
  'post-updated': []
  deleted: []
  quote: [author: string, body: string]
}>()

const authStore = useAuthStore()
const { removePost: doRemovePost } = useModeration()

const SAVE_MUTATION = `
  mutation SavePost($postId: ID!) { savePost(postId: $postId) { id isSaved } }
`
const UNSAVE_MUTATION = `
  mutation UnsavePost($postId: ID!) { unsavePost(postId: $postId) { id isSaved } }
`
const DELETE_MUTATION = `
  mutation DeletePost($postId: ID!) { deletePost(postId: $postId) { id } }
`
const REPORT_MUTATION = `
  mutation ReportPost($postId: ID!, $reason: String!) { reportPost(postId: $postId, reason: $reason) { success } }
`

const saved = ref(props.post.isSaved ?? false)
const showReport = ref(false)
const reportReason = ref('')
const showDeleteConfirm = ref(false)

watch(() => props.post.isSaved, (v) => { saved.value = v ?? false })

const isOwnPost = computed(() => authStore.user?.id === props.post.creator?.id)
const canModerate = computed(() => props.isModerator || authStore.isAdmin)

const displayName = computed(() => {
  if (!props.post.creator) return '[deleted]'
  return props.post.creator.displayName ?? props.post.creator.name
})

const avatarInitial = computed(() => {
  return displayName.value.charAt(0).toUpperCase()
})

async function toggleSave (): Promise<void> {
  const { execute } = useGraphQL()
  const mutation = saved.value ? UNSAVE_MUTATION : SAVE_MUTATION
  const result = await execute(mutation, { variables: { postId: props.post.id } })
  if (result) saved.value = !saved.value
}

async function submitReport (): Promise<void> {
  if (!reportReason.value.trim()) return
  const { execute } = useGraphQL()
  await execute(REPORT_MUTATION, { variables: { postId: props.post.id, reason: reportReason.value } })
  showReport.value = false
  reportReason.value = ''
}

async function deletePost (): Promise<void> {
  const { execute } = useGraphQL()
  await execute(DELETE_MUTATION, { variables: { postId: props.post.id } })
  showDeleteConfirm.value = false
  emit('deleted')
}

function handleQuoteOP (): void {
  const body = props.post.body ?? ''
  emit('quote', displayName.value, body)
}
</script>

<template>
  <article class="bg-white border border-gray-200 rounded-lg overflow-hidden">
    <!-- Thread title bar -->
    <div class="px-4 py-3 bg-primary/5 border-b border-primary/10">
      <div class="flex items-center gap-2 flex-wrap">
        <h1 class="text-lg font-bold text-gray-900 flex-1">
          {{ post.title }}
        </h1>
        <div class="flex items-center gap-2">
          <span v-if="post.isNSFW" class="badge badge-red">NSFW</span>
          <span v-if="post.isLocked" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-yellow-100 text-yellow-700 text-xs font-medium">
            <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
            </svg>
            Locked
          </span>
          <span v-if="post.isFeaturedBoard" class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-green-100 text-green-700 text-xs font-medium">
            Pinned
          </span>
        </div>
      </div>
    </div>

    <!-- OP post -->
    <div class="flex flex-col sm:flex-row">
      <!-- Author sidebar (desktop) -->
      <div class="hidden sm:flex flex-col items-center gap-1 px-4 py-4 bg-gray-50 border-r border-gray-100 min-w-[140px]">
        <div v-if="post.creator?.avatar" class="w-16 h-16 rounded-full overflow-hidden">
          <img :src="post.creator.avatar" :alt="displayName" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-16 h-16 rounded-full bg-primary/10 text-primary flex items-center justify-center text-2xl font-bold">
          {{ avatarInitial }}
        </div>
        <NuxtLink
          v-if="post.creator"
          :to="`/@${post.creator.name}`"
          class="text-sm font-semibold text-gray-900 no-underline hover:text-primary text-center mt-1"
        >
          {{ displayName }}
        </NuxtLink>
        <span v-else class="text-sm text-gray-400 italic">[deleted]</span>
        <span class="text-xs text-gray-400">Thread Starter</span>
      </div>

      <!-- Author bar (mobile) -->
      <div class="flex sm:hidden items-center gap-3 px-4 py-2 bg-gray-50 border-b border-gray-100">
        <div v-if="post.creator?.avatar" class="w-8 h-8 rounded-full overflow-hidden flex-shrink-0">
          <img :src="post.creator.avatar" :alt="displayName" class="w-full h-full object-cover" />
        </div>
        <div v-else class="w-8 h-8 rounded-full bg-primary/10 text-primary flex items-center justify-center text-sm font-semibold flex-shrink-0">
          {{ avatarInitial }}
        </div>
        <div>
          <NuxtLink
            v-if="post.creator"
            :to="`/@${post.creator.name}`"
            class="text-sm font-semibold text-gray-900 no-underline hover:text-primary"
          >
            {{ displayName }}
          </NuxtLink>
          <span class="text-xs text-gray-400 ml-2">Thread Starter</span>
        </div>
        <time :datetime="post.createdAt" class="text-xs text-gray-400 ml-auto">
          {{ timeAgo(post.createdAt) }}
        </time>
      </div>

      <!-- Post content -->
      <div class="flex-1 px-4 py-4">
        <div class="flex items-center justify-between mb-3">
          <time :datetime="post.createdAt" class="text-xs text-gray-400 hidden sm:block">
            {{ timeAgo(post.createdAt) }}
          </time>
          <span class="text-xs text-gray-400 font-mono">#1</span>
        </div>

        <!-- External link -->
        <a
          v-if="post.url"
          :href="post.url"
          class="inline-flex items-center gap-1 text-sm text-primary hover:underline mb-3"
          target="_blank"
          rel="noopener noreferrer"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
          {{ post.url }}
        </a>

        <!-- Image -->
        <div v-if="post.image" class="mb-3">
          <img
            :src="post.image"
            :alt="post.altText || post.title"
            class="max-w-full max-h-[600px] rounded-lg border border-gray-200 object-contain"
          />
        </div>

        <!-- Embed -->
        <div v-if="post.embedTitle || post.embedDescription" class="mb-3 border border-gray-200 rounded-lg p-3 bg-gray-50">
          <p v-if="post.embedTitle" class="text-sm font-medium text-gray-900">{{ post.embedTitle }}</p>
          <p v-if="post.embedDescription" class="text-xs text-gray-600 mt-1">{{ post.embedDescription }}</p>
          <div v-if="post.embedVideoUrl" class="mt-2">
            <iframe
              :src="post.embedVideoUrl"
              class="w-full aspect-video rounded"
              allowfullscreen
              loading="lazy"
            />
          </div>
        </div>

        <!-- Body -->
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div v-if="post.bodyHTML" class="prose prose-sm max-w-none" v-html="sanitizeHtml(post.bodyHTML)" />
        <div v-else-if="post.body" class="prose prose-sm max-w-none whitespace-pre-wrap">
          {{ post.body }}
        </div>
      </div>
    </div>

    <!-- Footer -->
    <div class="px-4 py-2 bg-gray-50 border-t border-gray-100 flex items-center justify-between flex-wrap gap-2">
      <!-- Left: reactions -->
      <CommonReactionBar target-type="post" :target-id="post.id" />

      <!-- Right: actions -->
      <div class="flex items-center gap-1 text-xs text-gray-500">
        <button
          v-if="authStore.isLoggedIn"
          class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-200 hover:text-gray-700 transition-colors"
          @click="handleQuoteOP"
          title="Quote this post"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
          </svg>
          Quote
        </button>

        <template v-if="authStore.isLoggedIn">
          <button
            class="px-2 py-1 rounded hover:bg-gray-200 hover:text-gray-700 transition-colors"
            @click="toggleSave"
          >
            {{ saved ? 'Unsave' : 'Save' }}
          </button>

          <button
            v-if="!isOwnPost"
            class="px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
            @click="showReport = true"
          >
            Report
          </button>

          <button
            v-if="isOwnPost"
            class="px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
            @click="showDeleteConfirm = true"
          >
            Delete
          </button>
        </template>

        <CommonReactionBar target-type="post" :target-id="post.id" />

        <template v-if="canModerate">
          <div class="w-px h-4 bg-gray-300 mx-1" />
          <PostModActions :post="post" @updated="emit('post-updated')" />
        </template>
      </div>
    </div>

    <!-- Report dialog -->
    <CommonModal v-if="showReport" @close="showReport = false">
      <template #title>Report Post</template>
      <template #default>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
            <textarea v-model="reportReason" class="form-input" rows="3" placeholder="Why are you reporting this post?" />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showReport = false">Cancel</button>
            <button class="button primary button-sm" @click="submitReport">Submit Report</button>
          </div>
        </div>
      </template>
    </CommonModal>

    <!-- Delete confirmation -->
    <CommonModal v-if="showDeleteConfirm" @close="showDeleteConfirm = false">
      <template #title>Delete Post</template>
      <template #default>
        <div class="space-y-3">
          <p class="text-sm text-gray-600">Are you sure you want to delete this post? This cannot be undone.</p>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showDeleteConfirm = false">Cancel</button>
            <button class="button button-sm bg-red-600 text-white hover:bg-red-700" @click="deletePost">Delete</button>
          </div>
        </div>
      </template>
    </CommonModal>
  </article>
</template>
