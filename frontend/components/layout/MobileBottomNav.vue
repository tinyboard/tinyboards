<script setup lang="ts">
import { useAuthStore } from '~/stores/auth'

const route = useRoute()
const authStore = useAuthStore()

function isActive (path: string): boolean {
  if (path === '/home') return route.path === '/home' || route.path === '/'
  return route.path.startsWith(path)
}
</script>

<template>
  <nav class="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 z-40">
    <div class="flex items-stretch justify-around h-14">
      <NuxtLink
        to="/home"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/home') ? 'text-primary' : 'text-gray-400'"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
        </svg>
        <span class="text-[10px] font-medium leading-none">Home</span>
      </NuxtLink>

      <NuxtLink
        to="/all"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/all') ? 'text-primary' : 'text-gray-400'"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-[10px] font-medium leading-none">All</span>
      </NuxtLink>

      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/submit"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/submit') ? 'text-primary' : 'text-gray-400'"
      >
        <div class="w-8 h-8 rounded-full bg-primary flex items-center justify-center -mt-1">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
        </div>
        <span class="text-[10px] font-medium leading-none">Post</span>
      </NuxtLink>
      <NuxtLink
        v-else
        to="/boards"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/boards') ? 'text-primary' : 'text-gray-400'"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
        </svg>
        <span class="text-[10px] font-medium leading-none">Boards</span>
      </NuxtLink>

      <NuxtLink
        to="/search"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/search') ? 'text-primary' : 'text-gray-400'"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <span class="text-[10px] font-medium leading-none">Search</span>
      </NuxtLink>

      <NuxtLink
        v-if="authStore.isLoggedIn"
        to="/inbox"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors relative"
        :class="isActive('/inbox') ? 'text-primary' : 'text-gray-400'"
      >
        <div class="relative">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
          </svg>
          <span
            v-if="authStore.unreadNotificationCount > 0"
            class="absolute -top-1 -right-1.5 flex items-center justify-center min-w-[18px] h-[18px] px-0.5 rounded-full bg-red-500 text-white text-[10px] font-bold leading-none pointer-events-none"
          >
            {{ authStore.unreadNotificationCount > 99 ? '99+' : authStore.unreadNotificationCount }}
          </span>
        </div>
        <span class="text-[10px] font-medium leading-none">Inbox</span>
      </NuxtLink>
      <NuxtLink
        v-else
        to="/login"
        class="flex flex-col items-center justify-center flex-1 gap-0.5 no-underline transition-colors"
        :class="isActive('/login') ? 'text-primary' : 'text-gray-400'"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
        <span class="text-[10px] font-medium leading-none">Log in</span>
      </NuxtLink>
    </div>
    <!-- iOS safe area padding -->
    <div class="h-[env(safe-area-inset-bottom,0px)] bg-white" />
  </nav>
</template>
