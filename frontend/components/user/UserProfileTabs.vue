<script setup lang="ts">
const props = defineProps<{
  username: string
  isOwnProfile?: boolean
}>()

const route = useRoute()

interface TabItem {
  label: string
  to: string
  icon: string
  ownOnly?: boolean
}

const tabs = computed<TabItem[]>(() => {
  const base = `/@${props.username}`
  const items: TabItem[] = [
    { label: 'Overview', to: base, icon: 'overview' },
    { label: 'Posts', to: `${base}/posts`, icon: 'posts' },
    { label: 'Comments', to: `${base}/comments`, icon: 'comments' },
  ]

  if (props.isOwnProfile) {
    items.push(
      { label: 'Saved', to: `${base}/saved`, icon: 'saved', ownOnly: true },
      { label: 'Following', to: `${base}/following`, icon: 'following', ownOnly: true },
    )
  }

  return items
})

function isActive (tab: TabItem): boolean {
  if (tab.to === `/@${props.username}`) {
    return route.path === tab.to || route.path === `${tab.to}/`
  }
  return route.path.startsWith(tab.to)
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
        <!-- Overview icon -->
        <svg v-if="tab.icon === 'overview'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
        <!-- Posts icon -->
        <svg v-if="tab.icon === 'posts'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
        </svg>
        <!-- Comments icon -->
        <svg v-if="tab.icon === 'comments'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 8h10M7 12h4m1 8l-4-4H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-3l-4 4z" />
        </svg>
        <!-- Saved icon -->
        <svg v-if="tab.icon === 'saved'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
        </svg>
        <!-- Following icon -->
        <svg v-if="tab.icon === 'following'" class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
        </svg>
        {{ tab.label }}
      </NuxtLink>
    </div>
  </nav>
</template>
