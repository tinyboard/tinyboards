<script setup lang="ts">
// Redirect handled by routeRules in nuxt.config.ts
// This page exists as a fallback
definePageMeta({
  layout: 'default',
})

navigateTo('/home', { redirectCode: 301 })
</script>

<template>
  <div />
</template>
