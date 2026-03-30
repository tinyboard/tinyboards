<script setup lang="ts">
import { ref } from 'vue'
import { useAuthStore } from '~/stores/auth'
import { useSiteStore } from '~/stores/site'
import { useUIStore } from '~/stores/ui'
import { useAuth } from '~/composables/useAuth'
import { useNotificationPolling } from '~/composables/useNotificationPolling'

const authStore = useAuthStore()
const siteStore = useSiteStore()
const uiStore = useUIStore()
const { logout } = useAuth()

// Start notification polling (handles SSR safety internally)
useNotificationPolling()

const userMenuOpen = ref(false)

function toggleUserMenu () {
  userMenuOpen.value = !userMenuOpen.value
}

function closeUserMenu () {
  userMenuOpen.value = false
}

async function handleLogout () {
  closeUserMenu()
  await logout()
}
</script>

<template>
  <header class="bg-primary sticky top-0 z-50 shadow-md">
    <div class="max-w-8xl mx-auto px-3 sm:px-6 h-14 flex items-center justify-between">
      <!-- Left: Logo + Nav -->
      <div class="flex items-center gap-4 min-w-0 flex-1">
        <button
          class="lg:hidden p-1.5 text-white/70 hover:text-white hover:bg-white/10 rounded"
          aria-label="Toggle sidebar"
          @click="uiStore.toggleSidebar()"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
        <NuxtLink
          to="/home"
          class="flex items-center gap-2 no-underline hover:no-underline"
        >
          <img
            v-if="siteStore.icon"
            :src="siteStore.icon"
            class="w-7 h-7 sm:w-8 sm:h-8 flex-shrink-0"
            :alt="siteStore.name"
          >
          <div v-else class="w-7 h-7 sm:w-8 sm:h-8 rounded-lg bg-white/20 flex items-center justify-center flex-shrink-0">
            <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
          </div>
          <span class="font-bold text-base sm:text-lg text-white truncate">{{ siteStore.name }}</span>
        </NuxtLink>
        <LayoutAppNav />
      </div>

      <!-- Right: Auth actions -->
      <div class="flex items-center gap-1.5 sm:gap-2">
        <!-- Search (always visible) -->
        <NuxtLink
          to="/search"
          class="relative flex items-center justify-center w-10 h-10 sm:w-9 sm:h-9 text-white/70 rounded hover:bg-white/10 hover:text-white transition-colors no-underline"
          aria-label="Search"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </NuxtLink>

        <template v-if="authStore.isLoggedIn">
          <NuxtLink
            to="/submit"
            class="relative flex items-center justify-center w-10 h-10 sm:w-9 sm:h-9 text-white rounded hover:bg-white/10 transition-colors no-underline"
            aria-label="Submit post"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
          </NuxtLink>
          <NuxtLink
            v-if="authStore.isAdmin"
            to="/admin"
            class="relative flex items-center justify-center w-9 h-9 text-white/70 rounded hover:bg-white/10 hover:text-white transition-colors no-underline hidden sm:flex"
            aria-label="Admin panel"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </NuxtLink>
          <NuxtLink
            to="/inbox"
            class="relative flex items-center justify-center w-10 h-10 sm:w-9 sm:h-9 text-white/70 rounded hover:bg-white/10 hover:text-white transition-colors no-underline"
            aria-label="Inbox"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
            </svg>
            <span
              v-if="authStore.unreadNotificationCount > 0"
              class="absolute -top-0.5 -right-0.5 flex items-center justify-center min-w-[16px] h-4 px-1 rounded-full bg-red-500 text-white text-[10px] font-bold leading-none"
            >
              {{ authStore.unreadNotificationCount > 99 ? '99+' : authStore.unreadNotificationCount }}
            </span>
          </NuxtLink>
          <!-- User menu dropdown -->
          <div class="relative">
            <button
              class="flex items-center gap-2 text-white hover:bg-white/10 rounded-lg px-2 py-1.5 transition-colors cursor-pointer"
              aria-label="User menu"
              @click="toggleUserMenu"
              @keydown.escape="closeUserMenu"
            >
              <CommonAvatar
                :src="authStore.user?.avatar ?? undefined"
                :name="authStore.user?.name ?? ''"
                size="sm"
              />
              <span class="hidden sm:inline text-sm font-medium text-white">{{ authStore.user?.name }}</span>
              <svg class="w-3.5 h-3.5 text-white/60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </button>

            <!-- Backdrop (click to close) -->
            <div
              v-if="userMenuOpen"
              class="fixed inset-0 z-[60]"
              @click="closeUserMenu"
            />

            <!-- Dropdown -->
            <Transition
              enter-active-class="transition ease-out duration-100"
              enter-from-class="transform opacity-0 scale-95"
              enter-to-class="transform opacity-100 scale-100"
              leave-active-class="transition ease-in duration-75"
              leave-from-class="transform opacity-100 scale-100"
              leave-to-class="transform opacity-0 scale-95"
            >
              <div
                v-if="userMenuOpen"
                class="absolute right-0 mt-1 w-48 max-w-[calc(100vw-2rem)] bg-white rounded-lg shadow-lg ring-1 ring-black/5 z-[70] py-1"
              >
                <div class="px-3 py-2 border-b border-gray-100">
                  <p class="text-sm font-medium text-gray-900 truncate">{{ authStore.user?.displayName || authStore.user?.name }}</p>
                  <p class="text-xs text-gray-500 truncate">@{{ authStore.user?.name }}</p>
                </div>

                <NuxtLink
                  :to="`/@${authStore.user?.name}`"
                  class="flex items-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 no-underline transition-colors"
                  @click="closeUserMenu"
                >
                  <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                  </svg>
                  Profile
                </NuxtLink>

                <NuxtLink
                  to="/settings"
                  class="flex items-center gap-2 px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 no-underline transition-colors"
                  @click="closeUserMenu"
                >
                  <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  Settings
                </NuxtLink>

                <div class="border-t border-gray-100 mt-1 pt-1">
                  <button
                    class="flex items-center gap-2 w-full px-3 py-2 text-sm text-red-600 hover:bg-red-50 transition-colors cursor-pointer"
                    @click="handleLogout"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                    </svg>
                    Log out
                  </button>
                </div>
              </div>
            </Transition>
          </div>
        </template>
        <template v-else>
          <NuxtLink
            to="/login"
            class="button button-sm gray no-underline"
          >
            Log in
          </NuxtLink>
          <NuxtLink
            to="/register"
            class="button button-sm primary no-underline hidden sm:inline-flex"
          >
            Sign up
          </NuxtLink>
        </template>
      </div>
    </div>
  </header>
</template>
