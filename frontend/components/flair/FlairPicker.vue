<script setup lang="ts">
defineEmits<{
  select: [flairId: number]
}>()
</script>

<template>
  <div class="bg-white border border-gray-200 rounded p-3">
    <h4 class="text-xs font-semibold text-gray-500 uppercase mb-2">
      Select Flair
    </h4>
    <slot />
  </div>
</template>
