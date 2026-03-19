<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  src?: string
  name: string
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl'
}>(), {
  src: undefined,
  size: 'md',
})

const sizeClasses = {
  xs: 'w-5 h-5 text-[10px]',
  sm: 'w-7 h-7 text-xs',
  md: 'w-10 h-10 text-sm',
  lg: 'w-16 h-16 text-xl',
  xl: 'w-24 h-24 text-3xl',
}

const initials = computed(() => {
  return props.name.charAt(0).toUpperCase()
})
</script>

<template>
  <div
    class="rounded-full overflow-hidden shrink-0 bg-primary/10 flex items-center justify-center"
    :class="sizeClasses[size]"
  >
    <img
      v-if="src"
      :src="src"
      :alt="`${name} avatar`"
      class="w-full h-full object-cover"
    >
    <span v-else class="font-semibold text-primary select-none">
      {{ initials }}
    </span>
  </div>
</template>
