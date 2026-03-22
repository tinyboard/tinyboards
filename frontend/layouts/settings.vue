<script setup lang="ts">
const settingsNav = [
  { label: 'Account', to: '/settings/account', icon: 'account' },
  { label: 'Profile', to: '/settings/profile', icon: 'profile' },
  { label: 'Appearance', to: '/settings/appearance', icon: 'appearance' },
  { label: 'Notifications', to: '/settings/notifications', icon: 'notifications' },
  { label: 'Privacy', to: '/settings/privacy', icon: 'privacy' },
  { label: 'Security', to: '/settings/security', icon: 'security' },
]

const mobileNavOpen = ref(false)
</script>

<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <!-- Mobile settings nav toggle -->
    <div class="lg:hidden bg-white border-b border-gray-200 px-4 py-2">
      <button
        class="flex items-center gap-2 text-sm font-medium text-gray-700 w-full"
        @click="mobileNavOpen = !mobileNavOpen"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
        Settings Menu
        <svg
          class="w-4 h-4 ml-auto transition-transform"
          :class="{ 'rotate-180': mobileNavOpen }"
          fill="none" stroke="currentColor" viewBox="0 0 24 24"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
      <nav v-if="mobileNavOpen" class="mt-2 pb-1">
        <NuxtLink
          v-for="item in settingsNav"
          :key="item.to"
          :to="item.to"
          class="block px-3 py-2.5 text-sm rounded no-underline text-gray-700 hover:bg-gray-50"
          active-class="bg-primary/5 text-primary font-medium"
          @click="mobileNavOpen = false"
        >
          {{ item.label }}
        </NuxtLink>
      </nav>
    </div>

    <div class="flex-1 max-w-5xl mx-auto w-full px-3 sm:px-4 py-4 sm:py-6 pb-20 lg:pb-6">
      <div class="flex gap-6">
        <!-- Settings sidebar nav (desktop) -->
        <nav class="w-52 shrink-0 hidden lg:block">
          <div class="bg-white rounded-lg border border-gray-200 overflow-hidden sticky top-20">
            <div class="px-4 py-3 bg-primary text-white">
              <h2 class="font-semibold text-sm">Settings</h2>
            </div>
            <ul class="p-2 space-y-0.5">
              <li v-for="item in settingsNav" :key="item.to">
                <NuxtLink
                  :to="item.to"
                  class="block px-3 py-2 text-sm rounded no-underline text-gray-700 hover:bg-gray-50 transition-colors"
                  active-class="bg-primary/5 text-primary font-medium"
                >
                  {{ item.label }}
                </NuxtLink>
              </li>
            </ul>
          </div>
        </nav>

        <!-- Content area -->
        <main class="flex-1 min-w-0">
          <div class="bg-white rounded-lg border border-gray-200 p-4 sm:p-6">
            <slot />
          </div>
        </main>
      </div>
    </div>

    <LayoutAppFooter class="hidden lg:block" />
    <LayoutMobileBottomNav class="lg:hidden" />
  </div>
</template>
