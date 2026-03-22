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
    class="w-[290px] shrink-0 overflow-y-auto"
    :class="{ 'hidden lg:block': !uiStore.sidebarOpen }"
  >
    <div class="p-4">
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
