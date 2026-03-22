<script setup lang="ts">
import type { Post } from '~/types/generated'
import { timeAgo } from '~/utils/date'
import { sanitizeHtml } from '~/utils/sanitize'
import { useAuthStore } from '~/stores/auth'
import { useGraphQL } from '~/composables/useGraphQL'
import { useModeration } from '~/composables/useModeration'
import { useToast } from '~/composables/useToast'

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
const toast = useToast()
const { removePost: doRemovePost, restorePost: doRestorePost, lockPost: doLockPost, unlockPost: doUnlockPost, featurePost: doFeaturePost } = useModeration()

const SAVE_MUTATION = `mutation SavePost($postId: ID!) { savePost(postId: $postId) { id isSaved } }`
const UNSAVE_MUTATION = `mutation UnsavePost($postId: ID!) { unsavePost(postId: $postId) { id isSaved } }`
const DELETE_MUTATION = `mutation DeletePost($postId: ID!) { deletePost(postId: $postId) { id } }`
const REPORT_MUTATION = `mutation ReportPost($postId: ID!, $reason: String!) { reportPost(postId: $postId, reason: $reason) { success } }`

const saved = ref(props.post.isSaved ?? false)
const showReport = ref(false)
const reportReason = ref('')
const showDeleteConfirm = ref(false)
const showModMenu = ref(false)
const showRemoveDialog = ref(false)
const removeReason = ref('')
const acting = ref(false)

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
  if (result) {
    saved.value = !saved.value
    toast.success(saved.value ? 'Post saved' : 'Post unsaved')
  }
}

async function submitReport (): Promise<void> {
  if (!reportReason.value.trim()) return
  const { execute } = useGraphQL()
  const result = await execute(REPORT_MUTATION, { variables: { postId: props.post.id, reason: reportReason.value } })
  showReport.value = false
  reportReason.value = ''
  if (result) toast.success('Report submitted')
}

async function deletePost (): Promise<void> {
  const { execute } = useGraphQL()
  const result = await execute(DELETE_MUTATION, { variables: { postId: props.post.id } })
  showDeleteConfirm.value = false
  if (result) {
    toast.success('Post deleted')
    emit('deleted')
  }
}

function handleQuoteOP (): void {
  const body = props.post.body ?? ''
  emit('quote', displayName.value, body)
}

async function handleLockToggle (): Promise<void> {
  acting.value = true
  showModMenu.value = false
  const success = props.post.isLocked ? await doUnlockPost(props.post.id) : await doLockPost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

async function handleFeatureToggle (): Promise<void> {
  acting.value = true
  showModMenu.value = false
  const success = await doFeaturePost(props.post.id, !props.post.isFeaturedBoard, 'board')
  acting.value = false
  if (success) emit('post-updated')
}

async function handleRemove (): Promise<void> {
  acting.value = true
  const success = await doRemovePost(props.post.id, removeReason.value || undefined)
  showRemoveDialog.value = false
  removeReason.value = ''
  acting.value = false
  if (success) emit('post-updated')
}

async function handleRestore (): Promise<void> {
  acting.value = true
  showModMenu.value = false
  const success = await doRestorePost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

function openRemoveDialog (): void {
  showModMenu.value = false
  showRemoveDialog.value = true
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

    <!-- Footer: unified actions -->
    <div class="px-4 py-2 bg-gray-50 border-t border-gray-100 flex items-center justify-between flex-wrap gap-2">
      <!-- Left: reactions -->
      <CommonReactionBar target-type="post" :target-id="post.id" />

      <!-- Right: actions -->
      <div class="flex items-center gap-1 text-xs text-gray-500">
        <button
          v-if="authStore.isLoggedIn"
          class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-200 hover:text-gray-700 transition-colors"
          title="Quote this post"
          @click="handleQuoteOP"
        >
          <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
          </svg>
          Quote
        </button>

        <template v-if="authStore.isLoggedIn">
          <button
            class="inline-flex items-center gap-1 px-2 py-1 rounded transition-colors"
            :class="saved ? 'text-primary bg-primary/10' : 'hover:bg-gray-200 hover:text-gray-700'"
            @click="toggleSave"
          >
            <svg class="w-3.5 h-3.5" :fill="saved ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
            </svg>
            {{ saved ? 'Saved' : 'Save' }}
          </button>

          <button
            v-if="!isOwnPost"
            class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
            @click="showReport = true"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 21v-4m0 0V5a2 2 0 012-2h6.5l1 1H21l-3 6 3 6h-8.5l-1-1H5a2 2 0 00-2 2zm9-13.5V9" />
            </svg>
            Report
          </button>

          <button
            v-if="isOwnPost"
            class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-red-50 hover:text-red-500 transition-colors"
            @click="showDeleteConfirm = true"
          >
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
            Delete
          </button>
        </template>

        <!-- Mod actions menu -->
        <template v-if="canModerate">
          <div class="w-px h-4 bg-gray-300 mx-1" />
          <div class="relative">
            <button
              class="inline-flex items-center gap-1 px-2 py-1 rounded transition-colors"
              :class="showModMenu ? 'bg-gray-200 text-gray-700' : 'hover:bg-gray-200 hover:text-gray-700'"
              @click="showModMenu = !showModMenu"
            >
              <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
              Mod
            </button>

            <Teleport to="body">
              <div v-if="showModMenu" class="fixed inset-0 z-40" @click="showModMenu = false" />
            </Teleport>
            <div
              v-if="showModMenu"
              class="absolute right-0 top-full mt-1 w-48 bg-white rounded-lg border border-gray-200 shadow-lg z-50 py-1"
            >
              <div class="px-3 py-1.5 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                Moderation
              </div>
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                :disabled="acting"
                @click="handleLockToggle"
              >
                <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path v-if="post.isLocked" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
                  <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                {{ post.isLocked ? 'Unlock' : 'Lock' }}
              </button>
              <button
                class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
                :class="post.isFeaturedBoard ? 'text-green-700 hover:bg-green-50' : 'text-gray-700 hover:bg-gray-50'"
                :disabled="acting"
                @click="handleFeatureToggle"
              >
                <svg class="w-4 h-4" :class="post.isFeaturedBoard ? 'text-green-500' : 'text-gray-400'" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                </svg>
                {{ post.isFeaturedBoard ? 'Unpin' : 'Pin' }}
              </button>
              <div class="border-t border-gray-100 my-1" />
              <button
                v-if="!post.isRemoved"
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
                :disabled="acting"
                @click="openRemoveDialog"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                </svg>
                Remove
              </button>
              <button
                v-else
                class="w-full flex items-center gap-2 px-3 py-2 text-sm text-green-600 hover:bg-green-50 transition-colors"
                :disabled="acting"
                @click="handleRestore"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Restore
              </button>
            </div>
          </div>
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

    <!-- Remove reason dialog -->
    <CommonModal v-if="showRemoveDialog" @close="showRemoveDialog = false">
      <template #title>Remove Post</template>
      <template #default>
        <div class="space-y-3">
          <p class="text-sm text-gray-600">Are you sure you want to remove this post?</p>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason (optional)</label>
            <input v-model="removeReason" type="text" class="form-input" placeholder="Reason for removal..." />
          </div>
          <div class="flex gap-2 justify-end">
            <button class="button white button-sm" @click="showRemoveDialog = false">Cancel</button>
            <button class="button button-sm bg-red-600 text-white hover:bg-red-700" :disabled="acting" @click="handleRemove">
              {{ acting ? 'Removing...' : 'Remove Post' }}
            </button>
          </div>
        </div>
      </template>
    </CommonModal>
  </article>
</template>
