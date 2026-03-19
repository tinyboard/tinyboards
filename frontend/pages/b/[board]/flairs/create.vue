<script setup lang="ts">
// Flair creation is handled inline on the flairs index page.
// Redirect there with the create form pre-opened via query param.
const route = useRoute()
const boardName = route.params.board as string

definePageMeta({ middleware: 'guards' })

onMounted(async () => {
  await navigateTo(`/b/${boardName}/flairs`)
})
</script>

<template>
  <div class="p-4">
    <p class="text-sm text-gray-500">Redirecting to flair management...</p>
  </div>
</template>
