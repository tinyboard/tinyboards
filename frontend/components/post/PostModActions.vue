<script setup lang="ts">
import { ref } from 'vue'
import { useModeration } from '~/composables/useModeration'
import { useAuthStore } from '~/stores/auth'
import type { Post } from '~/types/generated'

const props = defineProps<{
  post: Post
}>()

const emit = defineEmits<{
  updated: []
}>()

const { lockPost, unlockPost, featurePost, removePost, restorePost, distinguishPost, markNsfwPost, unmarkNsfwPost } = useModeration()

const authStore = useAuthStore()
const showRemoveDialog = ref(false)
const removeReason = ref('')
const acting = ref(false)

const isOwnPost = computed(() => authStore.user?.id === props.post.creatorId)

async function handleDistinguish (): Promise<void> {
  acting.value = true
  const success = await distinguishPost(props.post.id)
  acting.value = false
  if (success) emit('updated')
}

async function handleNsfwToggle (): Promise<void> {
  acting.value = true
  const success = props.post.isNSFW
    ? await unmarkNsfwPost(props.post.id)
    : await markNsfwPost(props.post.id)
  acting.value = false
  if (success) emit('updated')
}

async function handleLockToggle (): Promise<void> {
  acting.value = true
  const success = props.post.isLocked
    ? await unlockPost(props.post.id)
    : await lockPost(props.post.id)
  acting.value = false
  if (success) emit('updated')
}

async function handleFeatureToggle (): Promise<void> {
  acting.value = true
  const success = await featurePost(props.post.id, !props.post.isFeaturedBoard, 'board')
  acting.value = false
  if (success) emit('updated')
}

async function handleRemove (): Promise<void> {
  acting.value = true
  const success = await removePost(props.post.id, removeReason.value || undefined)
  showRemoveDialog.value = false
  removeReason.value = ''
  acting.value = false
  if (success) emit('updated')
}

async function handleRestore (): Promise<void> {
  acting.value = true
  const success = await restorePost(props.post.id)
  acting.value = false
  if (success) emit('updated')
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-1 text-xs">
    <!-- Lock / Unlock -->
    <button
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-100 text-gray-500 hover:text-gray-700"
      :disabled="acting"
      @click="handleLockToggle"
    >
      <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
        <path v-if="post.isLocked" fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd" />
        <path v-else d="M10 2a5 5 0 00-5 5v2a2 2 0 00-2 2v5a2 2 0 002 2h10a2 2 0 002-2v-5a2 2 0 00-2-2h-1V7a4 4 0 00-7.92-.8.75.75 0 01-1.46-.36A5.5 5.5 0 0115.5 7v2h.5a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2V7a5 5 0 015-5z" />
      </svg>
      {{ post.isLocked ? 'Unlock' : 'Lock' }}
    </button>

    <!-- Feature / Unfeature (Pin) -->
    <button
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-100"
      :class="post.isFeaturedBoard ? 'text-green-600 hover:text-green-700' : 'text-gray-500 hover:text-gray-700'"
      :disabled="acting"
      @click="handleFeatureToggle"
    >
      <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
        <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
      </svg>
      {{ post.isFeaturedBoard ? 'Unpin' : 'Pin' }}
    </button>

    <!-- Distinguish (only for own posts) -->
    <button
      v-if="isOwnPost"
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-100"
      :class="post.distinguishedAs ? 'text-green-600 hover:text-green-700' : 'text-gray-500 hover:text-gray-700'"
      :disabled="acting"
      @click="handleDistinguish"
    >
      <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 20 20">
        <path d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
      </svg>
      {{ post.distinguishedAs ? 'Undistinguish' : 'Distinguish' }}
    </button>

    <!-- NSFW Toggle -->
    <button
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-100"
      :class="post.isNSFW ? 'text-red-600 hover:text-red-700' : 'text-gray-500 hover:text-gray-700'"
      :disabled="acting"
      @click="handleNsfwToggle"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path v-if="post.isNSFW" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.878 9.878L3 3m6.878 6.878L21 21" />
        <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
      {{ post.isNSFW ? 'Unmark NSFW' : 'Mark NSFW' }}
    </button>

    <!-- Remove / Restore -->
    <button
      v-if="!post.isRemoved"
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-red-50 text-gray-500 hover:text-red-600"
      :disabled="acting"
      @click="showRemoveDialog = true"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
      Remove
    </button>
    <button
      v-else
      class="inline-flex items-center gap-1 px-2 py-1 rounded hover:bg-green-50 text-red-500 hover:text-green-600"
      :disabled="acting"
      @click="handleRestore"
    >
      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
      </svg>
      Restore
    </button>

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
  </div>
</template>
