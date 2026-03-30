<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'

const route = useRoute()
const authStore = useAuthStore()

const baseItems = [
  { label: 'Home', to: '/home', icon: 'home' },
  { label: 'All', to: '/all', icon: 'globe' },
  { label: 'Boards', to: '/boards', icon: 'boards' },
]

// Derive dynamic nav from subscribed board modes (Step 10)
const hasMixedModes = computed(() => {
  if (!authStore.isLoggedIn || authStore.subscribedBoards.length === 0) return false
  const modes = new Set(authStore.subscribedBoards.map(b => b.mode).filter(Boolean))
  return modes.has('feed') && modes.has('forum')
})

const navItems = computed(() => {
  if (!hasMixedModes.value) return baseItems
  return [
    ...baseItems,
    { label: 'Feed Posts', to: '/home?mode=feed', icon: 'feed' },
    { label: 'Discussions', to: '/home?mode=forum', icon: 'threads' },
  ]
})

function isActive (path: string): boolean {
  if (path.includes('?')) {
    const [basePath, query] = path.split('?')
    return route.path.startsWith(basePath) && route.fullPath.includes(query)
  }
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
      <!-- Feed icon -->
      <svg v-else-if="item.icon === 'feed'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
      </svg>
      <!-- Threads icon -->
      <svg v-else-if="item.icon === 'threads'" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
      </svg>
      {{ item.label }}
    </NuxtLink>
  </nav>
</template>
