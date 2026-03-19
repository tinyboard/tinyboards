<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  submit: [data: { reason: string; expires: string; removeContent: boolean }]
}>()

const reason = ref('')
const expires = ref('')
const removeContent = ref(false)

function handleSubmit (): void {
  emit('submit', {
    reason: reason.value,
    expires: expires.value,
    removeContent: removeContent.value,
  })
}
</script>

<template>
  <form class="space-y-4" @submit.prevent="handleSubmit">
    <div>
      <label for="ban-reason" class="block text-sm font-medium text-gray-700 mb-1">Reason</label>
      <textarea id="ban-reason" v-model="reason" class="form-input min-h-[60px]" />
    </div>
    <div>
      <label for="ban-expires" class="block text-sm font-medium text-gray-700 mb-1">Expires</label>
      <input id="ban-expires" v-model="expires" type="datetime-local" class="form-input">
    </div>
    <div class="flex items-center gap-2">
      <input id="ban-remove" v-model="removeContent" type="checkbox">
      <label for="ban-remove" class="text-sm text-gray-700">Remove all content</label>
    </div>
    <button type="submit" class="button red">
      Ban User
    </button>
  </form>
</template>
