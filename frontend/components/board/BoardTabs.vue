<script setup lang="ts">
const props = defineProps<{
  boardName: string
  isMod?: boolean
  mode?: string
  wikiEnabled?: boolean
}>()

const route = useRoute()

interface TabItem {
  label: string
  to: string
  icon?: string
}

const tabs = computed<TabItem[]>(() => {
  const base: TabItem[] = []
  const boardMode = props.mode ?? 'feed'

  if (boardMode === 'feed') {
    base.push({ label: 'Posts', to: `/b/${props.boardName}`, icon: 'feed' })
  } else {
    base.push({ label: 'Discussions', to: `/b/${props.boardName}`, icon: 'threads' })
  }

  if (props.wikiEnabled) {
    base.push({ label: 'Wiki', to: `/b/${props.boardName}/wiki`, icon: 'wiki' })
  }

  base.push({ label: 'Members', to: `/b/${props.boardName}/members`, icon: 'members' })

  if (props.isMod) {
    base.push(
      { label: 'Flairs', to: `/b/${props.boardName}/flairs` },
      { label: 'Settings', to: `/b/${props.boardName}/settings` },
      { label: 'Mod', to: `/b/${props.boardName}/mod/queue` },
    )
  }

  return base
})

function isActive (tab: TabItem): boolean {
  const to = tab.to
  if (to === `/b/${props.boardName}`) {
    return route.path === to || route.path === `${to}/`
  }
  return route.path.startsWith(to)
}
</script>

<template>
  <nav class="bg-white border-b border-gray-200">
    <div class="max-w-5xl mx-auto px-4 flex gap-0.5 overflow-x-auto scrollbar-hidden">
      <NuxtLink
        v-for="tab in tabs"
        :key="tab.to"
        :to="tab.to"
        class="px-3 py-2.5 text-sm whitespace-nowrap no-underline transition-colors border-b-2 flex items-center gap-1.5"
        :class="[
          isActive(tab)
            ? 'border-primary text-primary font-medium'
            : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
        ]"
      >
        <!-- Thread icon -->
        <svg v-if="tab.icon === 'threads'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
        </svg>
        <!-- Feed icon -->
        <svg v-if="tab.icon === 'feed'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
        </svg>
        <!-- Members icon -->
        <svg v-if="tab.icon === 'members'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
        <!-- Wiki icon -->
        <svg v-if="tab.icon === 'wiki'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
        {{ tab.label }}
      </NuxtLink>
    </div>
  </nav>
</template>
