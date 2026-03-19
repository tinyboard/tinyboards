<script setup lang="ts">
import { useWiki } from '~/composables/useWiki'
import { useGraphQL } from '~/composables/useGraphQL'

const route = useRoute()
const boardName = route.params.board as string

useHead({ title: `Wiki - b/${boardName}` })

const { pages, loadingPages, fetchPages } = useWiki()
const boardId = ref<string | null>(null)
const isMod = ref(false)

const BOARD_QUERY = `
  query GetBoard($name: String!) {
    board(name: $name) { id }
  }
`

const BOARD_SETTINGS_QUERY = `
  query GetBoardSettings($boardId: ID!) {
    getBoardSettings(boardId: $boardId) {
      moderatorPermissions
    }
  }
`

onMounted(async () => {
  const { execute: execBoard } = useGraphQL<{ board: { id: string } }>()
  const boardResult = await execBoard(BOARD_QUERY, { variables: { name: boardName } })
  if (!boardResult?.board) return

  boardId.value = boardResult.board.id

  const { execute: execSettings } = useGraphQL<{ getBoardSettings: { moderatorPermissions: number | null } }>()
  const settingsResult = await execSettings(BOARD_SETTINGS_QUERY, { variables: { boardId: boardId.value } })
  if (settingsResult?.getBoardSettings?.moderatorPermissions != null) {
    isMod.value = true
  }

  await fetchPages(boardName)
})

function formatDate (dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-4">
    <div class="flex items-center justify-between mb-6">
      <div>
        <nav class="text-sm text-gray-500 mb-1">
          <NuxtLink :to="`/b/${boardName}`" class="hover:text-gray-700">b/{{ boardName }}</NuxtLink>
          <span class="mx-1">/</span>
          <span>Wiki</span>
        </nav>
        <h1 class="text-lg font-semibold text-gray-900">Wiki</h1>
      </div>
      <NuxtLink
        v-if="isMod"
        :to="`/b/${boardName}/wiki/new`"
        class="button primary button-sm"
      >
        Create page
      </NuxtLink>
    </div>

    <CommonLoadingSpinner v-if="loadingPages" />

    <template v-else-if="pages.length > 0">
      <div class="space-y-2">
        <NuxtLink
          v-for="page in pages"
          :key="page.id"
          :to="`/b/${boardName}/wiki/${page.slug}`"
          class="block bg-white rounded-lg border border-gray-200 p-4 hover:border-gray-300 transition-colors"
        >
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-medium text-gray-900">{{ page.title }}</h3>
            <span class="text-xs text-gray-500">{{ formatDate(page.updatedAt) }}</span>
          </div>
        </NuxtLink>
      </div>
    </template>

    <div v-else class="text-center py-12">
      <template v-if="isMod">
        <p class="text-sm text-gray-500 mb-4">No wiki pages yet.</p>
        <NuxtLink :to="`/b/${boardName}/wiki/new`" class="button primary">
          Create the first wiki page
        </NuxtLink>
      </template>
      <p v-else class="text-sm text-gray-500">
        This board doesn't have a wiki yet.
      </p>
    </div>
  </div>
</template>
