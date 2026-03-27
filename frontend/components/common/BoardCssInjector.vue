<script setup lang="ts">
/**
 * Injects board-level custom CSS into the page when viewing a board.
 * The board CSS is placed AFTER site CSS in the cascade, so it overrides site styles.
 *
 * CSS cascade order:
 *   1. Built-in theme (via Tailwind + theme classes)
 *   2. Site custom CSS (injected by theme.client.ts plugin via #tb-site-custom-css)
 *   3. Board custom CSS (injected by this component via #tb-board-custom-css)
 */

const props = defineProps<{
  css: string | null | undefined
}>()

const styleId = 'tb-board-custom-css'

function applyBoardCss () {
  if (!import.meta.client) return

  let el = document.getElementById(styleId) as HTMLStyleElement | null
  if (props.css) {
    if (!el) {
      el = document.createElement('style')
      el.id = styleId
      el.setAttribute('data-source', 'board-mod')
      document.head.appendChild(el)
    }
    el.textContent = props.css
  } else if (el) {
    el.textContent = ''
  }
}

function removeBoardCss () {
  if (!import.meta.client) return
  const el = document.getElementById(styleId)
  if (el) {
    el.textContent = ''
  }
}

watch(() => props.css, applyBoardCss, { immediate: true })

onMounted(applyBoardCss)
onBeforeUnmount(removeBoardCss)
</script>

<template>
  <!-- Renderless component — CSS is injected directly into document.head -->
  <span v-if="false" />
</template>
