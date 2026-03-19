<script setup lang="ts">
defineEmits<{
  select: [emoji: string]
}>()
</script>

<template>
  <ClientOnly>
    <div class="bg-white border border-gray-200 rounded shadow-lg p-3">
      <p class="text-sm text-gray-500">
        Emoji picker.
      </p>
    </div>
    <template #fallback>
      <CommonLoadingSpinner size="sm" />
    </template>
  </ClientOnly>
</template>
