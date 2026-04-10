<script setup lang="ts">
import { useSiteStore } from '~/stores/site'

const siteStore = useSiteStore()
</script>

<template>
  <footer class="bg-white/50 backdrop-blur-sm border-t border-gray-200 px-2.5 sm:px-6 2xl:px-12 py-5 mt-auto">
    <div class="max-w-8xl mx-auto flex flex-col sm:flex-row items-center justify-center sm:justify-start gap-2 text-xs text-gray-500">
      <div class="flex items-center gap-2 -mx-1 flex-wrap">
        <NuxtLink to="/help" class="px-2 py-1 hover:text-primary no-underline text-gray-400 transition-colors flex-shrink-0">
          Help
        </NuxtLink>
        <NuxtLink to="/help/about" class="px-2 py-1 hover:text-primary no-underline text-gray-400 transition-colors flex-shrink-0">
          About
        </NuxtLink>
        <NuxtLink to="/help/terms" class="px-2 py-1 hover:text-primary no-underline text-gray-400 transition-colors flex-shrink-0">
          Terms
        </NuxtLink>
        <NuxtLink to="/help/privacy" class="px-2 py-1 hover:text-primary no-underline text-gray-400 transition-colors flex-shrink-0">
          Privacy
        </NuxtLink>
      </div>
      <div class="flex items-center gap-1.5 text-gray-400 mt-2 sm:mt-0 sm:ml-auto text-center sm:text-left">
        <span>Powered by</span>
        <span class="text-primary/50 hover:text-primary font-semibold">{{ siteStore.name || 'TinyBoards' }}</span>
      </div>
    </div>
  </footer>
</template>
