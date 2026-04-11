<script setup lang="ts">
const props = defineProps<{
  isNsfw: boolean
}>()

const revealed = ref(false)

function reveal () {
  revealed.value = true
}
</script>

<template>
  <div v-if="isNsfw && !revealed" class="relative cursor-pointer overflow-hidden rounded-lg" @click.stop="reveal">
    <div class="blur-lg pointer-events-none select-none">
      <slot />
    </div>
    <div class="absolute inset-0 flex items-center justify-center bg-black/40 rounded-lg">
      <span class="px-3 py-1.5 bg-red-600 text-white text-sm font-medium rounded shadow">
        NSFW &mdash; Click to reveal
      </span>
    </div>
  </div>
  <slot v-else />
</template>
