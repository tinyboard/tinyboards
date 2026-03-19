<script setup lang="ts">
defineProps<{
  label: string
  color?: string
  backgroundColor?: string
}>()
</script>

<template>
  <span
    class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
    :style="{
      color: color ?? '#374151',
      backgroundColor: backgroundColor ?? '#f3f4f6',
    }"
  >
    {{ label }}
  </span>
</template>
