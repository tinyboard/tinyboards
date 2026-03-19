<script setup lang="ts">
const adminNav = [
  { label: 'Dashboard', to: '/admin' },
  { label: 'Settings', to: '/admin/settings' },
  { label: 'Appearance', to: '/admin/appearance' },
  { label: 'Users', to: '/admin/users' },
  { label: 'Bans', to: '/admin/bans' },
  { label: 'Board Settings', to: '/admin/board_settings' },
  { label: 'Content', to: '/admin/content' },
  { label: 'Filtering', to: '/admin/filtering' },
  { label: 'Applications', to: '/admin/applications' },
  { label: 'Invites', to: '/admin/invites' },
  { label: 'Mod Queue', to: '/admin/queue' },
  { label: 'Reports', to: '/admin/reports/posts' },
  { label: 'Removed', to: '/admin/removed/posts' },
  { label: 'Emojis', to: '/admin/emojis' },
  { label: 'Flairs', to: '/admin/flairs' },
  { label: 'Admins', to: '/admin/admins' },
  { label: 'Custom CSS', to: '/admin/css' },
]

const mobileNavOpen = ref(false)
</script>

<template>
  <div class="min-h-screen flex flex-col bg-gray-100">
    <LayoutAppHeader />

    <!-- Mobile admin nav toggle -->
    <div class="md:hidden bg-white border-b border-gray-200 px-4 py-2">
      <button
        class="flex items-center gap-2 text-sm font-medium text-gray-700"
        @click="mobileNavOpen = !mobileNavOpen"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
        Admin Menu
      </button>
      <nav v-if="mobileNavOpen" class="mt-2 pb-1">
        <NuxtLink
          v-for="item in adminNav"
          :key="item.to"
          :to="item.to"
          class="block px-3 py-2 text-sm rounded no-underline text-gray-700 hover:bg-gray-50"
          active-class="bg-primary/5 text-primary font-medium"
          @click="mobileNavOpen = false"
        >
          {{ item.label }}
        </NuxtLink>
      </nav>
    </div>

    <div class="flex-1 max-w-6xl mx-auto w-full px-4 py-6">
      <div class="flex gap-6">
        <!-- Admin sidebar nav (desktop) -->
        <nav class="w-52 shrink-0 hidden md:block">
          <div class="bg-white rounded-lg border border-gray-200 overflow-hidden sticky top-20">
            <div class="px-4 py-3 bg-primary text-white">
              <h2 class="font-semibold text-sm">Admin Panel</h2>
            </div>
            <ul class="p-2 space-y-0.5 max-h-[calc(100vh-8rem)] overflow-y-auto">
              <li v-for="item in adminNav" :key="item.to">
                <NuxtLink
                  :to="item.to"
                  class="block px-3 py-1.5 text-sm rounded no-underline text-gray-700 hover:bg-gray-50 transition-colors"
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
          <div class="bg-white rounded-lg border border-gray-200 p-6">
            <slot />
          </div>
        </main>
      </div>
    </div>

    <LayoutAppFooter />
  </div>
</template>
