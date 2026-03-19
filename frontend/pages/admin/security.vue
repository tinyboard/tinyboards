<script setup lang="ts">
// Security settings have been consolidated into the Settings page.
definePageMeta({ layout: 'admin' })
navigateTo('/admin/settings', { replace: true, redirectCode: 301 })
</script>

<template>
  <div class="text-sm text-gray-500">
    Redirecting to settings...
  </div>
</template>
