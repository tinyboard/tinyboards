<script setup lang="ts">
import { useUIStore } from '~/stores/ui'
import { useAuthStore } from '~/stores/auth'

const uiStore = useUIStore()
const authStore = useAuthStore()
const route = useRoute()

type SidebarSection = 'home' | 'all' | 'board' | 'user' | 'boards' | 'search' | 'members' | 'default'

const currentSection = computed<SidebarSection>(() => {
  const path = route.path

  if (path.startsWith('/b/')) return 'board'
  if (path.startsWith('/@')) return 'user'
  if (path === '/home' || path.startsWith('/home/')) return 'home'
  if (path === '/all' || path.startsWith('/all/')) return 'all'
  if (path === '/boards' || path.startsWith('/boards/')) return 'boards'
  if (path === '/search') return 'search'
  if (path === '/members' || path.startsWith('/members/')) return 'members'

  return 'home'
})

// Close sidebar on route change (mobile)
watch(() => route.path, () => {
  if (uiStore.sidebarOpen) {
    uiStore.closeSidebar()
  }
})

// Lock body scroll when sidebar is open on mobile
watch(() => uiStore.sidebarOpen, (open) => {
  if (import.meta.client) {
    document.body.style.overflow = open ? 'hidden' : ''
  }
})
</script>

<template>
  <!-- Desktop: static sidebar -->
  <aside class="hidden lg:block w-80 xl:w-[22rem] shrink-0">
    <div class="sticky top-20 space-y-0 max-h-[calc(100vh-6rem)] overflow-y-auto py-4 pr-1">
      <SidebarHomeSidebar v-if="currentSection === 'home'" />
      <SidebarAllSidebar v-else-if="currentSection === 'all'" />
      <SidebarBoardPageSidebar v-else-if="currentSection === 'board'" />
      <SidebarUserProfileSidebar v-else-if="currentSection === 'user'" />
      <SidebarBoardsDirectorySidebar v-else-if="currentSection === 'boards'" />
      <SidebarSearchSidebar v-else-if="currentSection === 'search'" />
      <SidebarMembersSidebar v-else-if="currentSection === 'members'" />
      <SidebarHomeSidebar v-else />
    </div>
  </aside>

  <!-- Mobile: slide-out drawer -->
  <Teleport to="body">
    <Transition name="sidebar-overlay">
      <div
        v-if="uiStore.sidebarOpen"
        class="fixed inset-0 bg-black/40 z-50 lg:hidden"
        @click="uiStore.closeSidebar()"
      />
    </Transition>

    <Transition name="sidebar-drawer">
      <aside
        v-if="uiStore.sidebarOpen"
        class="fixed top-0 left-0 bottom-0 w-[min(320px,80vw)] bg-white z-50 lg:hidden shadow-xl overflow-y-auto overscroll-contain"
      >
        <!-- Drawer header -->
        <div class="sticky top-0 bg-primary text-white px-4 py-3 flex items-center justify-between z-10">
          <span class="font-semibold text-sm">Menu</span>
          <button
            class="p-1 rounded hover:bg-white/20 transition-colors"
            aria-label="Close menu"
            @click="uiStore.closeSidebar()"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- Mobile nav links (only items not in bottom nav) -->
        <nav class="px-3 py-3 border-b border-gray-200">
          <NuxtLink
            v-if="authStore.isLoggedIn"
            to="/boards"
            class="flex items-center gap-3 px-3 py-3 sm:py-2.5 text-sm font-medium rounded-lg no-underline transition-colors"
            :class="currentSection === 'boards' ? 'bg-primary/10 text-primary' : 'text-gray-700 hover:bg-gray-100'"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
            </svg>
            Boards
          </NuxtLink>
          <NuxtLink
            to="/members"
            class="flex items-center gap-3 px-3 py-3 sm:py-2.5 text-sm font-medium rounded-lg no-underline transition-colors"
            :class="currentSection === 'members' ? 'bg-primary/10 text-primary' : 'text-gray-700 hover:bg-gray-100'"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Members
          </NuxtLink>
          <NuxtLink
            v-if="authStore.isAdmin"
            to="/admin"
            class="flex items-center gap-3 px-3 py-3 sm:py-2.5 text-sm font-medium rounded-lg no-underline transition-colors"
            :class="route.path.startsWith('/admin') ? 'bg-primary/10 text-primary' : 'text-gray-700 hover:bg-gray-100'"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            Admin Panel
          </NuxtLink>
        </nav>

        <!-- Context sidebar content -->
        <div class="p-3">
          <SidebarHomeSidebar v-if="currentSection === 'home'" />
          <SidebarAllSidebar v-else-if="currentSection === 'all'" />
          <SidebarBoardPageSidebar v-else-if="currentSection === 'board'" />
          <SidebarUserProfileSidebar v-else-if="currentSection === 'user'" />
          <SidebarBoardsDirectorySidebar v-else-if="currentSection === 'boards'" />
          <SidebarSearchSidebar v-else-if="currentSection === 'search'" />
          <SidebarMembersSidebar v-else-if="currentSection === 'members'" />
          <SidebarHomeSidebar v-else />
        </div>
      </aside>
    </Transition>
  </Teleport>
</template>
