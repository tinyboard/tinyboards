<script setup lang="ts">
defineProps<{
  code: string
  url?: string
}>()
</script>

<template>
  <img
    v-if="url"
    :src="url"
    :alt="code"
    class="emoji"
  >
  <span v-else>{{ code }}</span>
</template>
