<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  send: [body: string]
}>()

const body = ref('')

function handleSend (): void {
  if (!body.value.trim()) { return }
  emit('send', body.value)
  body.value = ''
}
</script>

<template>
  <div class="flex gap-2">
    <input
      v-model="body"
      type="text"
      class="form-input flex-1"
      placeholder="Type a message..."
      @keyup.enter="handleSend"
    >
    <button class="button primary" :disabled="!body.trim()" @click="handleSend">
      Send
    </button>
  </div>
</template>
