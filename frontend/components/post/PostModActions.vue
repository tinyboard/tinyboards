<script setup lang="ts">
import { ref } from 'vue'
import { useModeration } from '~/composables/useModeration'
import type { Post } from '~/types/generated'

const props = defineProps<{
  post: Post
}>()

const emit = defineEmits<{
  updated: []
}>()

const { lockPost, unlockPost, featurePost, removePost, restorePost } = useModeration()

const showRemoveDialog = ref(false)
const removeReason = ref('')
const acting = ref(false)

async function handleLockToggle (): Promise<void> {
  acting.value = true
  if (props.post.isLocked) {
    await unlockPost(props.post.id)
  } else {
    await lockPost(props.post.id)
  }
  acting.value = false
  emit('updated')
}

async function handleFeatureToggle (): Promise<void> {
  acting.value = true
  await featurePost(props.post.id, !props.post.isFeaturedBoard, 'board')
  acting.value = false
  emit('updated')
}

async function handleRemove (): Promise<void> {
  acting.value = true
  await removePost(props.post.id, removeReason.value || undefined)
  showRemoveDialog.value = false
  removeReason.value = ''
  acting.value = false
  emit('updated')
}

async function handleRestore (): Promise<void> {
  acting.value = true
  await restorePost(props.post.id)
  acting.value = false
  emit('updated')
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
