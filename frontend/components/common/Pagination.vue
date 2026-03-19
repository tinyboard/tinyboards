<script setup lang="ts">
defineProps<{
  page: number
  hasMore: boolean
}>()

const emit = defineEmits<{
  prev: []
  next: []
}>()

function handlePrev () {
  emit('prev')
  if (import.meta.client) window.scrollTo({ top: 0, behavior: 'smooth' })
}

function handleNext () {
  emit('next')
  if (import.meta.client) window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>

<template>
  <div class="flex items-center justify-between py-4">
    <button
      class="button white button-sm"
      :disabled="page <= 1"
      @click="handlePrev"
    >
      Previous
    </button>

    <span class="text-sm text-gray-500">Page {{ page }}</span>

    <button
      class="button white button-sm"
      :disabled="!hasMore"
      @click="handleNext"
    >
      Next
    </button>
  </div>
</template>
