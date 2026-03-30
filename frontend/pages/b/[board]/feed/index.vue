<script setup lang="ts">
// The /b/[board]/feed/ route is deprecated. All content is now shown at /b/[board]/.
const route = useRoute()
const boardName = route.params.board as string

await navigateTo(`/b/${boardName}`, { replace: true, redirectCode: 301 })
</script>

<template>
  <div />
</template>
