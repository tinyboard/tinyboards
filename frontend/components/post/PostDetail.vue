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
}>()

const authStore = useAuthStore()
const toast = useToast()

const { removePost: doRemovePost, restorePost: doRestorePost, lockPost: doLockPost, unlockPost: doUnlockPost, featurePost: doFeaturePost, distinguishPost: doDistinguishPost, markNsfwPost: doMarkNsfwPost, unmarkNsfwPost: doUnmarkNsfwPost } = useModeration()

const SAVE_MUTATION = `mutation SavePost($postId: ID!) { savePost(postId: $postId) { id isSaved } }`
const UNSAVE_MUTATION = `mutation UnsavePost($postId: ID!) { unsavePost(postId: $postId) { id isSaved } }`
const DELETE_MUTATION = `mutation DeletePost($postId: ID!) { deletePost(postId: $postId) { id } }`
const REPORT_MUTATION = `mutation ReportPost($postId: ID!, $reason: String!) { reportPost(postId: $postId, reason: $reason) { success } }`

const saved = ref(props.post.isSaved ?? false)
const showReport = ref(false)
const reportReason = ref('')
const showDeleteConfirm = ref(false)
const showMoreMenu = ref(false)
const showRemoveDialog = ref(false)
const removeReason = ref('')
const acting = ref(false)

watch(() => props.post.isSaved, (v) => { saved.value = v ?? false })

const isOwnPost = computed(() => authStore.user?.id === props.post.creator?.id)
const canModerate = computed(() => props.isModerator || authStore.isAdmin)
const isImageVideo = computed(() => {
  const url = props.post.image
  return !!url && /\.(mp4|webm|ogg|mov)$/i.test(url)
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

async function handleLockToggle (): Promise<void> {
  acting.value = true
  showMoreMenu.value = false
  const success = props.post.isLocked
    ? await doUnlockPost(props.post.id)
    : await doLockPost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

async function handleFeatureToggle (): Promise<void> {
  acting.value = true
  showMoreMenu.value = false
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
  showMoreMenu.value = false
  const success = await doRestorePost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

async function handleDistinguish (): Promise<void> {
  acting.value = true
  showMoreMenu.value = false
  const success = await doDistinguishPost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

async function handleNsfwToggle (): Promise<void> {
  acting.value = true
  showMoreMenu.value = false
  const success = props.post.isNSFW
    ? await doUnmarkNsfwPost(props.post.id)
    : await doMarkNsfwPost(props.post.id)
  acting.value = false
  if (success) emit('post-updated')
}

function openRemoveDialog (): void {
  showMoreMenu.value = false
  showRemoveDialog.value = true
}

// Close menu on outside click
function onClickOutsideMenu (e: Event): void {
  showMoreMenu.value = false
}
</script>

<template>
  <article class="bg-white border border-gray-200 rounded p-3 sm:p-4">
    <div class="flex flex-col sm:flex-row gap-3">
      <!-- Vote column: hidden on mobile, shown on sm+ -->
      <PostActions :post="post" layout="vertical" class="hidden sm:flex" />

      <div class="flex-1 min-w-0">
        <h1 class="text-xl font-bold text-gray-900 leading-snug">
          {{ post.title }}
        </h1>

        <!-- External link -->
        <a
          v-if="post.url"
          :href="post.url"
          class="text-sm text-primary hover:underline mt-1 inline-block"
          target="_blank"
          rel="noopener noreferrer"
        >
          {{ post.url }}
        </a>

        <!-- Video display (uploaded video file) -->
        <CommonNsfwBlur v-if="post.image && isImageVideo" fluid :is-nsfw="post.isNSFW" class="mt-3 max-w-full">
          <video
            :src="post.image"
            class="max-w-full max-h-[400px] sm:max-h-[600px] rounded-lg border border-gray-200"
            controls
            preload="metadata"
          />
        </CommonNsfwBlur>

        <!-- Image display -->
        <CommonNsfwBlur v-if="post.image && !isImageVideo" :is-nsfw="post.isNSFW" class="mt-3">
          <img
            :src="post.image"
            :alt="post.altText || post.title"
            class="max-w-full max-h-[400px] sm:max-h-[600px] rounded-lg border border-gray-200 object-contain"
          />
        </CommonNsfwBlur>

        <!-- Embed preview -->
        <CommonNsfwBlur v-if="post.embedTitle || post.embedDescription" fluid :is-nsfw="post.isNSFW" class="mt-3">
          <div class="border border-gray-200 rounded-lg p-3 bg-gray-50">
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
        </CommonNsfwBlur>

        <!-- Meta -->
        <div class="flex items-center gap-1.5 sm:gap-1 mt-2 text-xs text-gray-500 flex-wrap">
          <NuxtLink v-if="post.board" :to="`/b/${post.board.name}`" class="font-medium text-gray-700 no-underline hover:underline">
            b/{{ post.board.name }}
          </NuxtLink>
          <span>&middot;</span>
          <NuxtLink v-if="post.creator" :to="`/@${post.creator.name}`" class="no-underline hover:underline text-gray-500">
            {{ post.creator.displayName ?? post.creator.name }}
          </NuxtLink>
          <span v-if="post.distinguishedAs === 'admin'" class="inline-flex items-center px-1 py-0 rounded text-[10px] font-medium bg-green-100 text-green-700 border border-green-200 leading-4">
            Admin
          </span>
          <span v-else-if="post.distinguishedAs === 'mod'" class="inline-flex items-center px-1 py-0 rounded text-[10px] font-medium bg-blue-100 text-blue-700 border border-blue-200 leading-4">
            Mod
          </span>
          <span>&middot;</span>
          <time :datetime="post.createdAt">{{ timeAgo(post.createdAt) }}</time>
          <span v-if="post.isNSFW" class="badge badge-red ml-1">NSFW</span>
          <span v-if="post.isLocked" class="inline-flex items-center gap-0.5 text-yellow-500 ml-1">
            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
            </svg>
            Locked
          </span>
          <span v-if="post.isFeaturedBoard" class="inline-flex items-center gap-0.5 text-green-500 ml-1">
            <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
              <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
            </svg>
            Pinned
          </span>
          <span v-if="post.isRemoved" class="text-red-500 ml-1">Removed</span>
        </div>

        <!-- Body -->
        <CommonNsfwBlur v-if="post.bodyHTML" fluid :is-nsfw="post.isNSFW" class="block mt-4">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div class="prose prose-sm max-w-none" v-html="sanitizeHtml(post.bodyHTML)" />
        </CommonNsfwBlur>
        <CommonNsfwBlur v-else-if="post.body" fluid :is-nsfw="post.isNSFW" class="block mt-4">
          <div class="prose prose-sm max-w-none whitespace-pre-wrap">
            {{ post.body }}
          </div>
        </CommonNsfwBlur>

        <!-- Mobile inline votes -->
        <div class="sm:hidden mt-3">
          <PostActions :post="post" layout="horizontal" />
        </div>

        <!-- Action bar: unified user + mod actions -->
        <div class="flex items-center gap-1.5 sm:gap-1 mt-3 text-xs text-gray-500 flex-wrap">
          <!-- Comments count -->
          <span class="inline-flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            {{ post.commentCount }} comment{{ post.commentCount === 1 ? '' : 's' }}
          </span>

          <!-- Save/Unsave -->
          <template v-if="authStore.isLoggedIn">
            <span class="text-gray-300">&middot;</span>
            <button
              class="inline-flex items-center gap-1 transition-colors"
              :class="saved ? 'text-primary font-medium' : 'hover:text-gray-700'"
              @click="toggleSave"
            >
              <svg class="w-3.5 h-3.5" :fill="saved ? 'currentColor' : 'none'" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
              </svg>
              {{ saved ? 'Saved' : 'Save' }}
            </button>

            <!-- Report (non-own) -->
            <template v-if="!isOwnPost">
              <span class="text-gray-300">&middot;</span>
              <button class="inline-flex items-center gap-1 hover:text-red-500 transition-colors" @click="showReport = true">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 21v-4m0 0V5a2 2 0 012-2h6.5l1 1H21l-3 6 3 6h-8.5l-1-1H5a2 2 0 00-2 2zm9-13.5V9" />
                </svg>
                Report
              </button>
            </template>

            <!-- Delete (own post) -->
            <template v-if="isOwnPost">
              <span class="text-gray-300">&middot;</span>
              <button class="inline-flex items-center gap-1 hover:text-red-500 transition-colors" @click="showDeleteConfirm = true">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
                Delete
              </button>
            </template>

            <!-- Mod/Admin actions menu -->
            <template v-if="canModerate">
              <span class="text-gray-300">&middot;</span>
              <div class="relative">
                <button
                  class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded hover:bg-gray-100 transition-colors"
                  :class="showMoreMenu ? 'bg-gray-100 text-gray-700' : 'text-gray-500 hover:text-gray-700'"
                  @click="showMoreMenu = !showMoreMenu"
                >
                  <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  Mod
                </button>

                <!-- Dropdown menu -->
                <Teleport to="body">
                  <div
                    v-if="showMoreMenu"
                    class="fixed inset-0 z-40"
                    @click="showMoreMenu = false"
                  />
                </Teleport>
                <div
                  v-if="showMoreMenu"
                  class="absolute left-0 top-full mt-1 w-48 max-w-[calc(100vw-2rem)] bg-white rounded-lg border border-gray-200 shadow-lg z-50 py-1"
                >
                  <div class="px-3 py-1.5 text-[10px] font-semibold text-gray-400 uppercase tracking-wider">
                    Moderation
                  </div>

                  <!-- Lock/Unlock -->
                  <button
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-50 transition-colors"
                    :disabled="acting"
                    @click="handleLockToggle"
                  >
                    <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path v-if="post.isLocked" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
                      <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                    </svg>
                    {{ post.isLocked ? 'Unlock post' : 'Lock post' }}
                  </button>

                  <!-- Pin/Unpin -->
                  <button
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
                    :class="post.isFeaturedBoard ? 'text-green-700 hover:bg-green-50' : 'text-gray-700 hover:bg-gray-50'"
                    :disabled="acting"
                    @click="handleFeatureToggle"
                  >
                    <svg class="w-4 h-4" :class="post.isFeaturedBoard ? 'text-green-500' : 'text-gray-400'" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                    </svg>
                    {{ post.isFeaturedBoard ? 'Unpin post' : 'Pin post' }}
                  </button>

                  <!-- NSFW Toggle -->
                  <button
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
                    :class="post.isNSFW ? 'text-red-700 hover:bg-red-50' : 'text-gray-700 hover:bg-gray-50'"
                    :disabled="acting"
                    @click="handleNsfwToggle"
                  >
                    <svg class="w-4 h-4" :class="post.isNSFW ? 'text-red-500' : 'text-gray-400'" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path v-if="post.isNSFW" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
                      <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
                    </svg>
                    {{ post.isNSFW ? 'Unmark NSFW' : 'Mark NSFW' }}
                  </button>

                  <!-- Distinguish (own posts only) -->
                  <button
                    v-if="isOwnPost"
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors"
                    :class="post.distinguishedAs ? 'text-green-700 hover:bg-green-50' : 'text-gray-700 hover:bg-gray-50'"
                    :disabled="acting"
                    @click="handleDistinguish"
                  >
                    <svg class="w-4 h-4" :class="post.distinguishedAs ? 'text-green-500' : 'text-gray-400'" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                    </svg>
                    {{ post.distinguishedAs ? 'Undistinguish' : 'Distinguish' }}
                  </button>

                  <div class="border-t border-gray-100 my-1" />

                  <!-- Remove/Restore -->
                  <button
                    v-if="!post.isRemoved"
                    class="w-full flex items-center gap-2 px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors"
                    :disabled="acting"
                    @click="openRemoveDialog"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 18.364A9 9 0 005.636 5.636m12.728 12.728A9 9 0 015.636 5.636m12.728 12.728L5.636 5.636" />
                    </svg>
                    Remove post
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
                    Restore post
                  </button>
                </div>
              </div>
            </template>
          </template>
        </div>

        <!-- Reactions (threads only) -->
        <div v-if="post.isThread" class="mt-3">
          <CommonReactionBar target-type="post" :target-id="post.id" :board-id="post.board?.id" />
        </div>
      </div>
    </div>

    <!-- Report dialog -->
    <CommonModal v-if="showReport" @close="showReport = false">
      <template #title>Report Post</template>
      <template #default>
        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
            <textarea
              v-model="reportReason"
              class="form-input"
              rows="3"
              placeholder="Why are you reporting this post?"
            />
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
            <input
              v-model="removeReason"
              type="text"
              class="form-input"
              placeholder="Reason for removal..."
            />
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
