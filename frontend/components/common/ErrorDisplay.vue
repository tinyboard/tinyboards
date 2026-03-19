<script setup lang="ts">
withDefaults(defineProps<{
  title?: string
  message?: string
  retryable?: boolean
}>(), {
  title: 'Something went wrong',
  message: 'An unexpected error occurred. Please try again.',
  retryable: true,
})

const emit = defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="flex flex-col items-center justify-center py-12 px-4 text-center">
    <svg class="w-12 h-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z" />
    </svg>
    <h2 class="text-lg font-semibold text-gray-900 mb-1">
      {{ title }}
    </h2>
    <p class="text-sm text-gray-500 mb-4 max-w-md">
      {{ message }}
    </p>
    <button
      v-if="retryable"
      class="button primary"
      @click="emit('retry')"
    >
      Try again
    </button>
  </div>
</template>
