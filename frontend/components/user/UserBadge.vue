<script setup lang="ts">
defineProps<{
  label: string
  type?: 'admin' | 'mod' | 'banned' | 'default'
}>()

const typeClasses = {
  admin: 'badge badge-blue',
  mod: 'badge badge-blue',
  banned: 'badge badge-red',
  default: 'badge',
}
</script>

<template>
  <span :class="typeClasses[type ?? 'default']">
    {{ label }}
  </span>
</template>
