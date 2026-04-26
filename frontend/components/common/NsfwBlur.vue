<script setup lang="ts">
const props = withDefaults(defineProps<{
  isNsfw: boolean
  // For media that has no intrinsic width (videos, iframes, link previews
  // with `w-full` children), pass `fluid` so the wrapper stays block-level
  // and lets the slot fill it. Default sizes the wrapper to its content so
  // the icon overlays the actual media instead of empty space beside it.
  fluid?: boolean
}>(), {
  fluid: false
})

const revealed = ref(false)

function reveal () {
  revealed.value = true
}
</script>

<template>
  <div
    v-if="isNsfw && !revealed"
    class="relative cursor-pointer overflow-hidden rounded-lg group max-w-full"
    :class="fluid ? 'block' : 'inline-block align-top'"
    title="NSFW – click to reveal"
    @click.stop="reveal"
  >
    <div class="blur-lg pointer-events-none select-none">
      <slot />
    </div>
    <div class="absolute inset-0 flex items-center justify-center">
      <div class="flex items-center justify-center w-10 h-10 rounded-full bg-black/30 backdrop-blur-sm text-white/90 transition-opacity group-hover:opacity-100 opacity-90">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="w-5 h-5"
          aria-hidden="true"
        >
          <path d="M9.88 9.88a3 3 0 1 0 4.24 4.24" />
          <path d="M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68" />
          <path d="M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61" />
          <line x1="2" y1="2" x2="22" y2="22" />
        </svg>
      </div>
    </div>
  </div>
  <slot v-else />
</template>
