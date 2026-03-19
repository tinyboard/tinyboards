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
</script>

<template>
  <article class="bg-white border border-gray-200 rounded p-4">
    <div class="flex gap-3">
      <PostActions :post="post" layout="vertical" />

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

        <!-- Image display -->
        <div v-if="post.image" class="mt-3">
          <img
            :src="post.image"
            :alt="post.altText || post.title"
            class="max-w-full max-h-[600px] rounded-lg border border-gray-200 object-contain"
          />
        </div>

        <!-- Embed preview -->
        <div v-if="post.embedTitle || post.embedDescription" class="mt-3 border border-gray-200 rounded-lg p-3 bg-gray-50">
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

        <!-- Meta -->
        <div class="flex items-center gap-1 mt-2 text-xs text-gray-500 flex-wrap">
          <NuxtLink v-if="post.board" :to="`/b/${post.board.name}`" class="font-medium text-gray-700 no-underline hover:underline">
            b/{{ post.board.name }}
          </NuxtLink>
          <span>&middot;</span>
          <NuxtLink v-if="post.creator" :to="`/@${post.creator.name}`" class="no-underline hover:underline text-gray-500">
            {{ post.creator.displayName ?? post.creator.name }}
          </NuxtLink>
          <span>&middot;</span>
          <time :datetime="post.createdAt">{{ timeAgo(post.createdAt) }}</time>
          <span v-if="post.isNSFW" class="badge badge-red ml-1">NSFW</span>
          <span v-if="post.isLocked" class="inline-flex items-center gap-0.5 text-yellow-500 ml-1">
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
              <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
            </svg>
            Locked
          </span>
          <span v-if="post.isFeaturedBoard" class="text-green-500 ml-1">Pinned</span>
          <span v-if="post.isRemoved" class="text-red-500 ml-1">Removed</span>
        </div>

        <!-- Body -->
        <!-- eslint-disable-next-line vue/no-v-html -->
        <div v-if="post.bodyHTML" class="prose prose-sm mt-4 max-w-none" v-html="sanitizeHtml(post.bodyHTML)" />
        <div v-else-if="post.body" class="prose prose-sm mt-4 max-w-none whitespace-pre-wrap">
          {{ post.body }}
        </div>

        <!-- User actions row -->
        <div class="flex items-center gap-1 mt-3 text-xs text-gray-500 flex-wrap">
          <span class="inline-flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            {{ post.commentCount }} comment{{ post.commentCount === 1 ? '' : 's' }}
          </span>

          <template v-if="authStore.isLoggedIn">
            <span class="text-gray-300">&middot;</span>
            <button class="hover:text-gray-700" @click="toggleSave">
              {{ saved ? 'Unsave' : 'Save' }}
            </button>

            <template v-if="!isOwnPost">
              <span class="text-gray-300">&middot;</span>
              <button class="hover:text-red-500" @click="showReport = true">
                Report
              </button>
            </template>

            <template v-if="isOwnPost">
              <span class="text-gray-300">&middot;</span>
              <button class="hover:text-red-500" @click="showDeleteConfirm = true">
                Delete
              </button>
            </template>
          </template>
        </div>

        <!-- Reactions -->
        <div class="mt-3">
          <CommonReactionBar target-type="post" :target-id="post.id" />
        </div>

        <!-- Mod actions -->
        <div v-if="canModerate" class="mt-2 pt-2 border-t border-gray-100">
          <PostModActions :post="post" @updated="emit('post-updated')" />
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
  </article>
</template>
