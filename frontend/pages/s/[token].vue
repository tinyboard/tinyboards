<script setup lang="ts">
const route = useRoute()
const token = route.params.token as string

useHead({ title: 'Shared Stream' })
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <h1 class="text-lg font-semibold text-gray-900 mb-4">
      Shared Stream
    </h1>
    <p class="text-sm text-gray-500">
      Viewing shared stream ({{ token }}).
    </p>
  </div>
</template>
