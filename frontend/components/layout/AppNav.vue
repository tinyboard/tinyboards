<script setup lang="ts">
const route = useRoute()

const navItems = [
  { label: 'Home', to: '/home', icon: 'home' },
  { label: 'All', to: '/all', icon: 'globe' },
  { label: 'Boards', to: '/boards', icon: 'boards' },
]

function isActive (path: string): boolean {
  return route.path.startsWith(path)
}
</script>

<template>
  <nav class="hidden md:flex items-center gap-0.5 ml-4">
    <NuxtLink
      v-for="item in navItems"
      :key="item.to"
      :to="item.to"
      class="flex items-center gap-1.5 px-4 py-2 text-sm rounded no-underline transition-colors font-bold"
      :class="isActive(item.to)
        ? 'text-white bg-black/10 shadow-inner-xs'
        : 'text-white/70 hover:text-white'"
    >
      <!-- Home icon -->
      <svg v-if="item.icon === 'home'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
      </svg>
      <!-- Globe icon -->
      <svg v-else-if="item.icon === 'globe'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <!-- Boards icon -->
      <svg v-else-if="item.icon === 'boards'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
      </svg>
      {{ item.label }}
    </NuxtLink>
  </nav>
</template>
