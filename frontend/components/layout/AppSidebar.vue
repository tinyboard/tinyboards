<script setup lang="ts">
import { useUIStore } from '~/stores/ui'

const uiStore = useUIStore()
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
</script>

<template>
  <aside
    class="w-80 shrink-0"
    :class="{ 'hidden lg:block': !uiStore.sidebarOpen }"
  >
    <div class="sticky top-4 space-y-0 max-h-[calc(100vh-5rem)] overflow-y-auto py-4 pr-1">
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
</template>
