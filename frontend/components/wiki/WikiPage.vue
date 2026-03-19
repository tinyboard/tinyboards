<script setup lang="ts">
import { sanitizeHtml } from '~/utils/sanitize'

defineProps<{
  content: string
}>()
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -->
  <div class="prose prose-sm max-w-none" v-html="sanitizeHtml(content)" />
</template>
