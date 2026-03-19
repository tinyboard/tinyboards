<script setup lang="ts">
defineProps<{
  activeFlair?: string
}>()

defineEmits<{
  select: [flairName: string | null]
}>()
</script>

<template>
  <div class="flex items-center gap-1 overflow-x-auto py-1">
    <slot />
  </div>
</template>
