<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '~/stores/auth'

defineProps<{
  placeholder?: string
  boardId?: string
}>()

const emit = defineEmits<{
  submit: [body: string]
}>()

const authStore = useAuthStore()
const body = ref('')

function handleSubmit (): void {
  if (!body.value.trim()) { return }
  emit('submit', body.value)
  body.value = ''
}
</script>

<template>
  <div v-if="authStore.isLoggedIn" class="mt-4">
    <EditorMarkdownEditor
      v-model="body"
      :board-id="boardId"
      :placeholder="placeholder ?? 'Write a comment...'"
      min-height="80px"
    />
    <button
      class="button button-sm primary mt-2"
      :disabled="!body.trim()"
      @click="handleSubmit"
    >
      Comment
    </button>
  </div>
  <div v-else class="mt-4 text-sm text-gray-500">
    <NuxtLink to="/login" class="text-primary">
      Log in
    </NuxtLink> to comment.
  </div>
</template>
