<script setup lang="ts">
defineProps<{
  modelValue: string
  question?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <div>
    <label for="application-text" class="block text-sm font-medium text-gray-700 mb-1">
      Application
    </label>
    <p v-if="question" class="text-sm text-gray-600 mb-2">
      {{ question }}
    </p>
    <textarea
      id="application-text"
      :value="modelValue"
      class="form-input min-h-[100px]"
      required
      placeholder="Tell us why you'd like to join..."
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    />
  </div>
</template>
