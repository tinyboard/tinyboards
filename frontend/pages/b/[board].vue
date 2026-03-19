<script setup lang="ts">
import { useBoard } from '~/composables/useBoard'
import { useGraphQL } from '~/composables/useGraphQL'
import { useAuthStore } from '~/stores/auth'

const route = useRoute()
const boardName = route.params.board as string
const authStore = useAuthStore()

const { board, isSubscribed, loading, error, fetchBoard, subscribe, unsubscribe } = useBoard()

await fetchBoard(boardName)

useHead({
  title: board.value?.title ?? boardName,
  link: [{ rel: 'canonical', href: `/b/${boardName}` }],
})
useSeoMeta({
  title: computed(() => board.value?.title ?? boardName),
  ogTitle: computed(() => board.value?.title ?? boardName),
  description: computed(() => board.value?.description || `Posts in +${boardName}`),
  ogDescription: computed(() => board.value?.description || `Posts in +${boardName}`),
  ogImage: computed(() => board.value?.icon || undefined),
  ogType: 'website',
})

const isMod = ref(false)

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

onMounted(async () => {
  if (!board.value || !authStore.isLoggedIn) return

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: board.value.id } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  }
})
</script>

<template>
  <div>
    <CommonLoadingSpinner v-if="loading && !board" size="lg" />
    <CommonErrorDisplay v-else-if="error" :message="error.message" @retry="fetchBoard(boardName)" />

    <template v-else-if="board">
      <BoardHeader :board="board" :is-subscribed="isSubscribed" @subscribe="subscribe" @unsubscribe="unsubscribe" />
      <BoardTabs
        :board-name="board.name"
        :is-mod="isMod"
        :section-config="board.sectionConfig ?? 3"
        :wiki-enabled="board.wikiEnabled ?? false"
      />

      <div class="max-w-5xl mx-auto px-4 py-4">
        <NuxtPage />
      </div>
    </template>
  </div>
</template>
